#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    Symbol,
};

#[contracttype]
pub enum DataKey {
    Admin,
    Queue(u64), // ID -> Proposal
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Proposal {
    pub id: u64,
    pub target: Address,
    pub call: BytesN<32>,
    pub eta: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyQueued = 3,
    NotQueued = 4,
    NotReady = 5,
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
    extern crate std; // Required for catch_unwind in tests
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, BytesN, Env};

    #[test]
    fn queue_and_execute_success() {
        let env = Env::default();
        env.mock_all_auths();
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
    }

    #[test]
    #[should_panic(expected = "Error(NotReady)")]
    fn execution_too_early_panics() {
        let env = Env::default();
        env.mock_all_auths();
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
    }
}
