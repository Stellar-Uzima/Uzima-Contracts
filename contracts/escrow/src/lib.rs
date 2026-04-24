#![no_std]
#![allow(clippy::needless_borrow)]
#![allow(clippy::unnecessary_cast)]
#![allow(dead_code)]

pub mod errors;
pub use errors::Error;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Symbol,
    Vec,
};


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EscrowStatus {
    Pending = 0,
    Active = 1,
    Settled = 2,
    Refunded = 3,
    Disputed = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct Escrow {
    pub order_id: u64,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub token: Address,
    pub status: EscrowStatus,
    pub approvals: Vec<Address>,
    pub reason: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PlatformStats {
    pub total_volume: i128,
    pub total_escrows: u64,
    pub settled_count: u64,
    pub refunded_count: u64,
    pub disputed_count: u64,
    pub active_count: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct DailyStats {
    pub day_id: u64,
    pub volume: i128,
    pub count: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ExportMetadata {
    pub format: String,
    pub checksum: BytesN<32>,
    pub timestamp: u64,
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
const STATS: Symbol = symbol_short!("stats");
const ADMIN: Symbol = symbol_short!("admin");
const DAILY_STATS: Symbol = symbol_short!("dlystats");

#[contract]
pub struct EscrowContract;

fn require_not_reentrant(env: &Env) -> Result<(), Error> {
    let locked: bool = env
        .storage()
        .temporary()
        .get(&REENTRANCY_LOCK)
        .unwrap_or(false);
    if locked {
        return Err(Error::ReentrancyGuard);
    }
    env.storage().temporary().set(&REENTRANCY_LOCK, &true);
    Ok(())
}

fn clear_reentrancy(env: &Env) {
    env.storage().temporary().remove(&REENTRANCY_LOCK);
}

fn add_credit(env: &Env, addr: &Address, delta: i128) {
    let mut credits: Map<Address, i128> = env
        .storage()
        .persistent()
        .get(&CREDITS)
        .unwrap_or(Map::new(&env));
    let current = credits.get(addr.clone()).unwrap_or(0);
    let new_bal = current.saturating_add(delta);
    credits.set(addr.clone(), new_bal);
    env.storage().persistent().set(&CREDITS, &credits);
}

fn update_stats(
    env: &Env,
    volume: i128,
    is_new: bool,
    settled: bool,
    refunded: bool,
    disputed: bool,
    active_delta: i32,
) {
    let mut stats: PlatformStats = env
        .storage()
        .instance()
        .get(&STATS)
        .unwrap_or(PlatformStats {
            total_volume: 0,
            total_escrows: 0,
            settled_count: 0,
            refunded_count: 0,
            disputed_count: 0,
            active_count: 0,
        });

    if is_new {
        stats.total_escrows += 1;
        stats.total_volume = stats.total_volume.saturating_add(volume);

        // Time-bucketed daily stats
        let day_id = env.ledger().timestamp() / 86400;
        let mut daily_map: Map<u64, DailyStats> = env
            .storage()
            .persistent()
            .get(&DAILY_STATS)
            .unwrap_or(Map::new(env));
        let mut daily = daily_map.get(day_id).unwrap_or(DailyStats {
            day_id,
            volume: 0,
            count: 0,
        });
        daily.volume = daily.volume.saturating_add(volume);
        daily.count += 1;
        daily_map.set(day_id, daily);
        env.storage().persistent().set(&DAILY_STATS, &daily_map);
    }
    if settled {
        stats.settled_count += 1;
    }
    if refunded {
        stats.refunded_count += 1;
    }
    if disputed {
        stats.disputed_count += 1;
    }

    if active_delta > 0 {
        stats.active_count = stats.active_count.saturating_add(active_delta as u64);
    } else if active_delta < 0 {
        stats.active_count = stats.active_count.saturating_sub(active_delta.abs() as u64);
    }

    env.storage().instance().set(&STATS, &stats);
}

#[contractimpl]
impl EscrowContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::Unauthorized);
        }
        env.storage().instance().set(&ADMIN, &admin);
        Ok(())
    }

    pub fn set_fee_config(
        env: Env,
        caller: Address,
        fee_receiver: Address,
        platform_fee_bps: u32,
    ) -> Result<(), Error> {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotAdmin)?;
        if caller != admin {
            return Err(Error::NotAdmin);
        }
        // basic bounds: <= 10000 bps
        if platform_fee_bps > 10_000 {
            return Err(Error::InvalidFeeBps);
        }
        let conf = FeeConfig {
            fee_receiver,
            platform_fee_bps,
        };
        env.storage().persistent().set(&FEE_CONF, &conf);
        Ok(())
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
    ) -> Result<bool, Error> {
        payer.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        if escrows.contains_key(order_id) {
            return Err(Error::EscrowExists);
        }

        let approvals = Vec::new(&env);
        let e = Escrow {
            order_id,
            payer: payer.clone(),
            payee: payee.clone(),
            amount,
            token: token.clone(),
            status: EscrowStatus::Pending,
            approvals,
            reason: String::from_str(&env, ""),
        };
        escrows.set(order_id, e);
        env.storage().persistent().set(&ESCROWS, &escrows);

        update_stats(&env, amount, true, false, false, false, 0);

        // event
        let topics = (symbol_short!("EscNew"), order_id);
        env.events().publish(topics, (payer, payee, amount, token));
        Ok(true)
    }

    pub fn mark_disputed(env: Env, caller: Address, order_id: u64) -> Result<(), Error> {
        caller.require_auth();
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).ok_or(Error::EscrowNotFound)?;

        if e.status == EscrowStatus::Settled || e.status == EscrowStatus::Refunded {
            return Err(Error::AlreadySettled);
        }

        e.status = EscrowStatus::Disputed;
        escrows.set(order_id, e.clone());
        env.storage().persistent().set(&ESCROWS, &escrows);

        update_stats(&env, 0, false, false, false, true, 0);

        env.events()
            .publish((symbol_short!("EscDisput"), order_id), ());
        Ok(())
    }

    pub fn approve_release(env: Env, order_id: u64, approver: Address) -> Result<(), Error> {
        approver.require_auth();
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).ok_or(Error::EscrowNotFound)?;

        if e.status == EscrowStatus::Settled || e.status == EscrowStatus::Refunded {
            return Err(Error::AlreadySettled);
        }

        let mut approvals = e.approvals.clone();
        if !approvals.contains(&approver) {
            approvals.push_back(approver);
        }
        e.approvals = approvals;

        // Transition to Active if at least 1 approval exists (e.g., from payer)
        if e.status == EscrowStatus::Pending && e.approvals.len() > 0 {
            e.status = EscrowStatus::Active;
            update_stats(&env, 0, false, false, false, false, 1);
        }

        escrows.set(order_id, e);
        env.storage().persistent().set(&ESCROWS, &escrows);
        Ok(())
    }

    pub fn release_escrow(env: Env, order_id: u64) -> Result<bool, Error> {
        require_not_reentrant(&env)?;
        // checks
        let fee_conf: FeeConfig = env
            .storage()
            .persistent()
            .get(&FEE_CONF)
            .ok_or(Error::FeeNotSet)?;

        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).ok_or(Error::EscrowNotFound)?;

        if e.status == EscrowStatus::Settled || e.status == EscrowStatus::Refunded {
            return Err(Error::AlreadySettled);
        }
        // sequence check: must be Active or Disputed to settle
        if e.status == EscrowStatus::Pending {
            return Err(Error::InvalidStateTransition);
        }

        // simple threshold: at least 2 approvals (payer + oracle/admin)
        if e.approvals.len() < 2 {
            return Err(Error::InsufficientApprovals);
        }

        // effects: mark settled
        e.status = EscrowStatus::Settled;
        escrows.set(order_id, e.clone());
        env.storage().persistent().set(&ESCROWS, &escrows);

        // interactions: credit balances via pull-payment pattern
        let fee = (e.amount as i128).saturating_mul(fee_conf.platform_fee_bps as i128) / 10_000;
        let provider_amount = e.amount.saturating_sub(fee);
        add_credit(&env, &e.payee, provider_amount);
        add_credit(&env, &fee_conf.fee_receiver, fee);

        update_stats(&env, 0, false, true, false, false, -1);

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
        Ok(true)
    }

    pub fn refund_escrow(env: Env, order_id: u64, reason: String) -> Result<bool, Error> {
        require_not_reentrant(&env)?;
        let mut escrows: Map<u64, Escrow> = env
            .storage()
            .persistent()
            .get(&ESCROWS)
            .unwrap_or(Map::new(&env));
        let mut e = escrows.get(order_id).ok_or(Error::EscrowNotFound)?;

        if e.status == EscrowStatus::Settled || e.status == EscrowStatus::Refunded {
            return Err(Error::AlreadySettled);
        }
        // require Active/Disputed status or admin/oracle approval
        if e.status == EscrowStatus::Pending && e.approvals.is_empty() {
            return Err(Error::NoBasisToRefund);
        }

        let was_active = e.status == EscrowStatus::Active || e.status == EscrowStatus::Disputed;
        e.status = EscrowStatus::Refunded;
        e.reason = reason.clone();
        escrows.set(order_id, e.clone());
        env.storage().persistent().set(&ESCROWS, &escrows);

        // credit payer for refund
        add_credit(&env, &e.payer, e.amount);

        update_stats(
            &env,
            0,
            false,
            false,
            true,
            false,
            if was_active { -1 } else { 0 },
        );

        // #283: Refunded event with session_id, amount, mentee_id (payer), reason
        env.events().publish(
            (symbol_short!("Refunded"), order_id),
            (e.payer, e.amount, e.token, reason),
        );
        clear_reentrancy(&env);
        Ok(true)
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

    pub fn withdraw(env: Env, caller: Address, token: Address, to: Address) -> Result<i128, Error> {
        caller.require_auth();
        if caller != to {
            return Err(Error::Unauthorized);
        }
        require_not_reentrant(&env)?;
        let mut credits: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&CREDITS)
            .unwrap_or(Map::new(&env));
        let amount = credits.get(to.clone()).unwrap_or(0);
        if amount <= 0 {
            return Err(Error::NoCredit);
        }
        credits.set(to.clone(), 0);
        env.storage().persistent().set(&CREDITS, &credits);
        env.events()
            .publish((symbol_short!("Withdrawn"),), (to.clone(), amount, token));
        clear_reentrancy(&env);
        Ok(amount)
    }

    // #284: Analytics Query Functions (10 functions)
    pub fn get_total_volume(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&STATS)
            .map(|s: PlatformStats| s.total_volume)
            .unwrap_or(0)
    }

    pub fn get_total_escrows(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&STATS)
            .map(|s: PlatformStats| s.total_escrows)
            .unwrap_or(0)
    }

    pub fn get_settled_rate(env: Env) -> u32 {
        let stats: PlatformStats = env.storage().instance().get(&STATS).unwrap_or_else(|| {
            return PlatformStats {
                total_volume: 0,
                total_escrows: 0,
                settled_count: 0,
                refunded_count: 0,
                disputed_count: 0,
                active_count: 0,
            };
        });
        if stats.total_escrows == 0 {
            return 0;
        }
        ((stats.settled_count as u64 * 10000) / stats.total_escrows) as u32
    }

    pub fn get_refund_rate(env: Env) -> u32 {
        let stats: PlatformStats = env.storage().instance().get(&STATS).unwrap_or_else(|| {
            return PlatformStats {
                total_volume: 0,
                total_escrows: 0,
                settled_count: 0,
                refunded_count: 0,
                disputed_count: 0,
                active_count: 0,
            };
        });
        if stats.total_escrows == 0 {
            return 0;
        }
        ((stats.refunded_count as u64 * 10000) / stats.total_escrows) as u32
    }

    pub fn get_dispute_rate(env: Env) -> u32 {
        let stats: PlatformStats = env.storage().instance().get(&STATS).unwrap_or_else(|| {
            return PlatformStats {
                total_volume: 0,
                total_escrows: 0,
                settled_count: 0,
                refunded_count: 0,
                disputed_count: 0,
                active_count: 0,
            };
        });
        if stats.total_escrows == 0 {
            return 0;
        }
        ((stats.disputed_count as u64 * 10000) / stats.total_escrows) as u32
    }

    pub fn get_active_escrows_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&STATS)
            .map(|s: PlatformStats| s.active_count)
            .unwrap_or(0)
    }

    pub fn get_stats_summary(env: Env) -> PlatformStats {
        env.storage()
            .instance()
            .get(&STATS)
            .unwrap_or(PlatformStats {
                total_volume: 0,
                total_escrows: 0,
                settled_count: 0,
                refunded_count: 0,
                disputed_count: 0,
                active_count: 0,
            })
    }

    pub fn get_platform_health_score(env: Env) -> u32 {
        let stats = Self::get_stats_summary(env);
        if stats.total_escrows == 0 {
            return 10000;
        }
        let failure_rate =
            (stats.disputed_count + stats.refunded_count) as u64 * 10000 / stats.total_escrows;
        10000u32.saturating_sub(failure_rate as u32)
    }

    pub fn get_token_volume(env: Env, token: Address) -> i128 {
        // In a real system, you'd index this. For now, we simulate with a subset or global.
        // We'll return global volume if token matches a tracked one (simplified).
        Self::get_total_volume(env)
    }

    pub fn get_donor_reputation(env: Env, _donor: Address) -> u32 {
        // Simulated reputation based on successful settlements
        let stats = Self::get_stats_summary(env.clone());
        if stats.total_escrows == 0 {
            return 5000;
        }
        5000 + (Self::get_settled_rate(env) / 2)
    }

    pub fn get_daily_stats(env: Env, day_id: u64) -> Option<DailyStats> {
        let daily_map: Map<u64, DailyStats> = env
            .storage()
            .persistent()
            .get(&DAILY_STATS)
            .unwrap_or(Map::new(&env));
        daily_map.get(day_id)
    }

    pub fn export_summary(env: Env, format: String) -> ExportMetadata {
        // return metadata for an external indexer to recognize an export point
        ExportMetadata {
            format,
            checksum: BytesN::from_array(&env, &[0u8; 32]), // dummy checksum
            timestamp: env.ledger().timestamp(),
        }
    }
}

#[cfg(all(test, feature = "testutils"))]
#[allow(clippy::unwrap_used)]
#[cfg(all(test, feature = "testutils"))]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String};

    #[test]
    fn test_normal_release_flow() {
        let env = Env::default();
        let cid = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = Address::generate(&env);

        client.mock_all_auths().initialize(&admin);
        client
            .mock_all_auths()
            .set_fee_config(&admin, &Address::generate(&env), &250u32); // 2.5%

        assert!(client
            .mock_all_auths()
            .create_escrow(&1u64, &payer, &payee, &1000i128, &token));

        let e_pending = client.get_escrow(&1u64).unwrap();
        assert_eq!(e_pending.status, EscrowStatus::Pending);

        client.mock_all_auths().approve_release(&1u64, &payer);

        let e_active = client.get_escrow(&1u64).unwrap();
        assert_eq!(e_active.status, EscrowStatus::Active);

        client
            .mock_all_auths()
            .approve_release(&1u64, &Address::generate(&env));
        assert!(client.release_escrow(&1u64));

        let e = client.get_escrow(&1u64).unwrap();
        assert_eq!(e.status, EscrowStatus::Settled);

        // credits: payee 975, fee 25
        let payee_credit = client.get_credit(&payee);
        assert_eq!(payee_credit, 975);

        // analytics
        assert_eq!(client.get_total_volume(), 1000);
        assert_eq!(client.get_total_escrows(), 1);
        assert_eq!(client.get_settled_rate(), 10000);
    }

    #[test]
    fn test_refund_flow_with_dispute() {
        let env = Env::default();
        let cid = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = Address::generate(&env);

        client.mock_all_auths().initialize(&admin);
        client
            .mock_all_auths()
            .set_fee_config(&admin, &Address::generate(&env), &500u32);

        assert!(client
            .mock_all_auths()
            .create_escrow(&2u64, &payer, &payee, &1000i128, &token));
        client.mock_all_auths().mark_disputed(&payer, &2u64);

        let e_disputed = client.get_escrow(&2u64).unwrap();
        assert_eq!(e_disputed.status, EscrowStatus::Disputed);

        assert!(client.refund_escrow(&2u64, &String::from_str(&env, "Dispute resolved by refund")));
        let e = client.get_escrow(&2u64).unwrap();
        assert_eq!(e.status, EscrowStatus::Refunded);
        assert_eq!(
            e.reason,
            String::from_str(&env, "Dispute resolved by refund")
        );

        // payer credited
        let payer_credit = client.get_credit(&payer);
        assert_eq!(payer_credit, 1000);
    }

    #[test]
    fn test_invalid_transitions() {
        let env = Env::default();
        let cid = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = Address::generate(&env);

        client.mock_all_auths().initialize(&admin);
        client
            .mock_all_auths()
            .create_escrow(&3u64, &payer, &payee, &1000i128, &token);

        // Try to release while Pending (should fail)
        let res = client.try_release_escrow(&3u64);
        assert!(res.is_err());
    }

    #[test]
    fn test_authorization() {
        let env = Env::default();
        let cid = env.register_contract(None, EscrowContract);
        let client = EscrowContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);
        client.mock_all_auths().initialize(&admin);

        // Try to set fee config with non-admin
        let res = client.try_set_fee_config(&attacker, &attacker, &100u32);
        // Note: try_... functions return Result<Result<...>, ...> or similar depending on toolchain
        // In modern SDK, it returns Result<Val, Error>
        assert!(res.is_err());
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::NotAdmin as u32, 102);
        assert_eq!(Error::InvalidAmount as u32, 205);
        assert_eq!(Error::EscrowNotFound as u32, 481);
        assert_eq!(Error::AlreadySettled as u32, 482);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        use soroban_sdk::symbol_short;
        assert_eq!(crate::errors::get_suggestion(Error::Unauthorized), symbol_short!("CHK_AUTH"));
        assert_eq!(crate::errors::get_suggestion(Error::InvalidAmount), symbol_short!("CHK_LEN"));
        assert_eq!(crate::errors::get_suggestion(Error::EscrowNotFound), symbol_short!("CHK_ID"));
        assert_eq!(crate::errors::get_suggestion(Error::AlreadySettled), symbol_short!("ALREADY"));
    }
}
