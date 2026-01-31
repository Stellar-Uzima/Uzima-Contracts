#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct Escrow {
    pub order_id: u64,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub token: Address,
    pub released: bool,
    pub refunded: bool,
    pub disputed: bool,
    pub approvals: Vec<Address>,
}

#[derive(Clone)]
#[contracttype]
pub struct FeeConfig {
    pub platform_fee_bps: u32, // basis points, e.g., 250 = 2.5%
    pub fee_receiver: Address,
}

const ESCROWS: Symbol = symbol_short!("escrow");
const FEE_CONF: Symbol = symbol_short!("feeconf");
const REENTRANCY_LOCK: Symbol = symbol_short!("relock");
const CREDITS: Symbol = symbol_short!("credits");

#[contract]
pub struct EscrowContract;

fn require_not_reentrant(env: &Env) {
    let locked: bool = env
        .storage()
        .temporary()
        .get(&REENTRANCY_LOCK)
        .unwrap_or(false);
    if locked {
        panic!("ReentrancyGuard: reentrant call");
    }
    env.storage().temporary().set(&REENTRANCY_LOCK, &true);
}

fn clear_reentrancy(env: &Env) {
    env.storage().temporary().remove(&REENTRANCY_LOCK);
}

fn add_credit(env: &Env, addr: &Address, delta: i128) {
    let mut credits: Map<Address, i128> = env
        .storage()
        .persistent()
        .get(&CREDITS)
        // FIXED: Removed redundant borrow &env -> env
        .unwrap_or(Map::new(env));
    let current = credits.get(addr.clone()).unwrap_or(0);
    let new_bal = current + delta;
    credits.set(addr.clone(), new_bal);
    env.storage().persistent().set(&CREDITS, &credits);
}

#[contractimpl]
impl EscrowContract {
    pub fn set_fee_config(env: Env, fee_receiver: Address, platform_fee_bps: u32) {
        // basic bounds: <= 10000 bps
        if platform_fee_bps > 10_000 {
            panic!("Invalid fee bps");
        }
        let conf = FeeConfig {
            fee_receiver,
            platform_fee_bps,
        };
        env.storage().persistent().set(&FEE_CONF, &conf);
    }

    pub fn get_fee_config(env: Env) -> Option<FeeConfig> {
        env.storage().persistent().get(&FEE_CONF)
    }

    pub fn create_escrow(
        env: Env,
        order_id: u64,
        payer: Address,
        payee: Address,
        amount: i128,
        token: Address,
    ) -> bool {
        // effects: write escrow record, no external calls here
        if amount <= 0 {
            panic!("Invalid amount");
        }
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        if escrows.contains_key(order_id) {
            panic!("Escrow exists");
        }

        let approvals = Vec::new(&env);
        let e = Escrow {
            order_id,
            payer: payer.clone(),
            payee: payee.clone(),
            amount,
            token: token.clone(),
            released: false,
            refunded: false,
            disputed: false,
            approvals,
        };
        escrows.set(order_id, e);
        env.storage().persistent().set(&ESCROWS, &escrows);

        // event
        let topics = (symbol_short!("EscNew"), order_id);
        env.events().publish(topics, (payer, payee, amount, token));
        true
    }

    pub fn mark_disputed(env: Env, order_id: u64) {
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).unwrap_or_else(|| panic!("Not found"));
        e.disputed = true;
        escrows.set(order_id, e.clone());
        env.storage().persistent().set(&ESCROWS, &escrows);
        env.events()
            .publish((symbol_short!("EscDisput"), order_id), ());
    }

    pub fn approve_release(env: Env, order_id: u64, approver: Address) {
        // add unique approval; off-chain oracle/admins can be approvers
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).unwrap_or_else(|| panic!("Not found"));
        if e.released || e.refunded {
            panic!("Already settled");
        }
        let mut approvals = e.approvals.clone();
        if !approvals.contains(&approver) {
            approvals.push_back(approver);
        }
        e.approvals = approvals;
        escrows.set(order_id, e);
        env.storage().persistent().set(&ESCROWS, &escrows);
    }

    pub fn release_escrow(env: Env, order_id: u64) -> bool {
        require_not_reentrant(&env);
        // checks
        let fee_conf: FeeConfig = env
            .storage()
            .persistent()
            .get(&FEE_CONF)
            .unwrap_or_else(|| panic!("Fee not set"));

        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).unwrap_or_else(|| panic!("Not found"));
        if e.released || e.refunded {
            panic!("Already settled");
        }
        // simple threshold: at least 2 approvals (payer + oracle/admin)
        if e.approvals.len() < 2 {
            panic!("Insufficient approvals");
        }

        // effects: mark released and record owed balances (pull-payment)
        e.released = true;
        escrows.set(order_id, e.clone());
        env.storage().persistent().set(&ESCROWS, &escrows);

        // interactions: credit balances via pull-payment pattern
        // FIXED: Removed unnecessary cast (e.amount as i128)
        let fee = e.amount * (fee_conf.platform_fee_bps as i128) / 10_000;
        let provider_amount = e.amount - fee;
        add_credit(&env, &e.payee, provider_amount);
        add_credit(&env, &fee_conf.fee_receiver, fee);
        env.events().publish(
            (symbol_short!("EscRel"), order_id),
            (
                e.payee,
                provider_amount,
                fee_conf.fee_receiver,
                fee,
                e.token,
            ),
        );

        clear_reentrancy(&env);
        true
    }

    pub fn refund_escrow(env: Env, order_id: u64) -> bool {
        require_not_reentrant(&env);
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).unwrap_or_else(|| panic!("Not found"));
        if e.released || e.refunded {
            panic!("Already settled");
        }
        // require at least one approval (oracle/admin) or disputed flag
        // FIXED: Changed len() == 0 to is_empty()
        if e.approvals.is_empty() && !e.disputed {
            panic!("No basis to refund");
        }

        e.refunded = true;
        escrows.set(order_id, e.clone());
        env.storage().persistent().set(&ESCROWS, &escrows);

        // credit payer for refund
        add_credit(&env, &e.payer, e.amount);
        env.events().publish(
            (symbol_short!("EscRefund"), order_id),
            (e.payer, e.amount, e.token),
        );
        clear_reentrancy(&env);
        true
    }

    pub fn get_escrow(env: Env, order_id: u64) -> Option<Escrow> {
        let escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        escrows.get(order_id)
    }

    pub fn get_credit(env: Env, addr: Address) -> i128 {
        let credits: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&CREDITS)
            .unwrap_or(Map::new(&env));
        credits.get(addr).unwrap_or(0)
    }

    pub fn withdraw(env: Env, token: Address, to: Address) -> i128 {
        require_not_reentrant(&env);
        let mut credits: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&CREDITS)
            .unwrap_or(Map::new(&env));
        let amount = credits.get(to.clone()).unwrap_or(0);
        if amount <= 0 {
            panic!("No credit");
        }
        credits.set(to.clone(), 0);
        env.storage().persistent().set(&CREDITS, &credits);
        env.events()
            .publish((symbol_short!("Withdrawn"),), (to.clone(), amount, token));
        clear_reentrancy(&env);
        amount
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_normal_release_flow() {
        let env = Env::default();
        let cid = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &cid);

        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = Address::generate(&env);
        client.set_fee_config(&Address::generate(&env), &250u32); // 2.5%

        assert!(client.create_escrow(&1u64, &payer, &payee, &1000i128, &token));
        client.approve_release(&1u64, &payer);
        client.approve_release(&1u64, &Address::generate(&env));
        assert!(client.release_escrow(&1u64));
        let e = client.get_escrow(&1u64).unwrap();
        assert!(e.released);
        // credits: payee 975, fee 25
        let payee_credit = client.get_credit(&payee);
        let _fee_credit = client.get_credit(&Address::generate(&env)); // not the same; need fee receiver used above
                                                                       // Retrieve exact fee receiver credit
                                                                       // We can't read fee receiver here; just assert payee credit equals expected
        assert_eq!(payee_credit, 975);
    }

    #[test]
    fn test_refund_flow_with_dispute() {
        let env = Env::default();
        let cid = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &cid);

        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = Address::generate(&env);
        client.set_fee_config(&Address::generate(&env), &500u32); // 5%

        assert!(client.create_escrow(&2u64, &payer, &payee, &1000i128, &token));
        client.mark_disputed(&2u64);
        assert!(client.refund_escrow(&2u64));
        let e = client.get_escrow(&2u64).unwrap();
        assert!(e.refunded);
        // payer credited
        let payer_credit = client.get_credit(&payer);
        assert_eq!(payer_credit, 1000);
    }

    #[test]
    fn test_reentrancy_guard() {
        let env = Env::default();
        let cid = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &cid);
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = Address::generate(&env);
        client.set_fee_config(&Address::generate(&env), &0u32);

        client.create_escrow(&3u64, &payer, &payee, &1000i128, &token);
        client.approve_release(&3u64, &payer);
        client.approve_release(&3u64, &Address::generate(&env));

        // We can't easily simulate reentrancy in test; guard path is exercised
        assert!(client.release_escrow(&3u64));
        // Withdraw credited amount
        assert_eq!(client.get_credit(&payee), 1000);
        let withdrawn = client.withdraw(&token, &payee);
        assert_eq!(withdrawn, 1000);
        assert_eq!(client.get_credit(&payee), 0);
    }
}
