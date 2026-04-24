#![no_std]

pub mod errors;
pub use errors::Error;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    Symbol,
};

#[derive(Clone)]
#[contracttype]
pub struct TimelockConfig {
    pub admin: Address,
    pub delay_seconds: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct QueuedTx {
    pub target: Address,
    pub call: BytesN<32>,
    pub eta: u64,
}


const CFG: Symbol = symbol_short!("cfg");
const QUEUE: Symbol = symbol_short!("queue");

#[contract]
pub struct Timelock;

#[contractimpl]
impl Timelock {
    pub fn initialize(env: Env, admin: Address, delay_seconds: u64) -> Result<(), Error> {
        if env.storage().persistent().has(&CFG) {
            return Err(Error::AlreadyInitialized);
        }
        let cfg = TimelockConfig {
            admin,
            delay_seconds,
        };
        env.storage().persistent().set(&CFG, &cfg);
        Ok(())
    }

    pub fn get_config(env: Env) -> Option<TimelockConfig> {
        env.storage().persistent().get(&CFG)
    }

    pub fn queue(env: Env, id: u64, target: Address, call: BytesN<32>) -> Result<(), Error> {
        let cfg: TimelockConfig = env
            .storage()
            .persistent()
            .get(&CFG)
            .ok_or(Error::NotInitialized)?;
        let now: u64 = env.ledger().timestamp();
        let eta = now.saturating_add(cfg.delay_seconds);
        let mut q: Map<u64, QueuedTx> = env
            .storage()
            .persistent()
            .get(&QUEUE)
            .unwrap_or(Map::new(&env));
        if q.contains_key(id) {
            return Err(Error::AlreadyQueued);
        }
        q.set(id, QueuedTx { target, call, eta });
        env.storage().persistent().set(&QUEUE, &q);
        env.events().publish((symbol_short!("Queued"), id), (eta,));
        Ok(())
    }

    pub fn execute(env: Env, id: u64) -> Result<(), Error> {
        let mut q: Map<u64, QueuedTx> = env
            .storage()
            .persistent()
            .get(&QUEUE)
            .unwrap_or(Map::new(&env));
        let tx = q.get(id).ok_or(Error::NotQueued)?;
        let now: u64 = env.ledger().timestamp();
        if now < tx.eta {
            return Err(Error::NotReady);
        }
        // In Soroban, cross-contract call dispatch is via auth + address invocations off-chain.
        // Here we just emit execution event and remove from queue.
        q.remove(id);
        env.storage().persistent().set(&QUEUE, &q);
        env.events()
            .publish((symbol_short!("Exec"), id), (tx.target, tx.call));
        Ok(())
    }
}

#[cfg(all(test, feature = "testutils"))]
#[allow(clippy::unwrap_used, clippy::panic)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, BytesN, Env};

    #[test]
    fn queue_and_execute_success() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, Timelock);
        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            Timelock::initialize(env.clone(), admin, 10).unwrap();
            let target = Address::generate(&env);
            let call = BytesN::from_array(&env, &[0u8; 32]);
            Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

            // Advance time past eta
            env.ledger().set(LedgerInfo {
                timestamp: env.ledger().timestamp() + 15,
                ..Default::default()
            });
            Timelock::execute(env.clone(), 1).unwrap();

            // ensure queue cleared
            let q: Map<u64, QueuedTx> = env
                .storage()
                .persistent()
                .get(&QUEUE)
                .unwrap_or(Map::new(&env));
            assert!(!q.contains_key(1));
        });
    }

    #[test]
    #[should_panic(expected = "NotReady")]
    fn execution_too_early_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, Timelock);
        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            Timelock::initialize(env.clone(), admin, 10).unwrap();
            let target = Address::generate(&env);
            let call = BytesN::from_array(&env, &[0u8; 32]);
            Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

            // Advance time below eta
            env.ledger().set(LedgerInfo {
                timestamp: env.ledger().timestamp() + 5,
                ..Default::default()
            });
            // This returns Err, but in tests unhandled Result can panic?
            // Or rather we should unwrap it to force panic for should_panic test
            Timelock::execute(env.clone(), 1).unwrap();
        });
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::ContractPaused as u32, 302);
        assert_eq!(Error::DeadlineExceeded as u32, 306);
        assert_eq!(Error::InsufficientFunds as u32, 500);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        use soroban_sdk::symbol_short;
        assert_eq!(crate::errors::get_suggestion(Error::Unauthorized), symbol_short!("CHK_AUTH"));
        assert_eq!(crate::errors::get_suggestion(Error::NotInitialized), symbol_short!("INIT_CTR"));
        assert_eq!(crate::errors::get_suggestion(Error::AlreadyInitialized), symbol_short!("ALREADY"));
        assert_eq!(crate::errors::get_suggestion(Error::ContractPaused), symbol_short!("RE_TRY_L"));
        assert_eq!(crate::errors::get_suggestion(Error::InsufficientFunds), symbol_short!("ADD_FUND"));
    }
}
