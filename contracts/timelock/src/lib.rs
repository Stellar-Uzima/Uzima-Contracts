#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, Symbol,
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
    pub fn initialize(env: Env, admin: Address, delay_seconds: u64) {
        if env.storage().persistent().has(&CFG) {
            panic!("Already initialized");
        }
        let cfg = TimelockConfig {
            admin,
            delay_seconds,
        };
        env.storage().persistent().set(&CFG, &cfg);
    }

    pub fn get_config(env: Env) -> Option<TimelockConfig> {
        env.storage().persistent().get(&CFG)
    }

    pub fn queue(env: Env, id: u64, target: Address, call: BytesN<32>) {
        let cfg: TimelockConfig = env
            .storage()
            .persistent()
            .get(&CFG)
            .unwrap_or_else(|| panic!("Not init"));
        let now: u64 = env.ledger().timestamp();
        let eta = now + cfg.delay_seconds;
        let mut q: Map<u64, QueuedTx> = env
            .storage()
            .persistent()
            .get(&QUEUE)
            .unwrap_or(Map::new(&env));
        if q.contains_key(id) {
            panic!("Already queued");
        }
        q.set(id, QueuedTx { target, call, eta });
        env.storage().persistent().set(&QUEUE, &q);
        env.events().publish((symbol_short!("Queued"), id), (eta,));
    }

    pub fn execute(env: Env, id: u64) {
        let mut q: Map<u64, QueuedTx> = env
            .storage()
            .persistent()
            .get(&QUEUE)
            .unwrap_or(Map::new(&env));
        let tx = q.get(id).unwrap_or_else(|| panic!("Not queued"));
        let now: u64 = env.ledger().timestamp();
        if now < tx.eta {
            panic!("Not ready");
        }
        // In Soroban, cross-contract call dispatch is via auth + address invocations off-chain.
        // Here we just emit execution event and remove from queue.
        q.remove(id);
        env.storage().persistent().set(&QUEUE, &q);
        env.events()
            .publish((symbol_short!("Exec"), id), (tx.target, tx.call));
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Address, BytesN, Env};

    #[test]
    fn queue_and_execute_success() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        Timelock::initialize(env.clone(), admin, 10);
        let target = Address::generate(&env);
        let call = BytesN::from_array(&env, &[0u8; 32]);
        Timelock::queue(env.clone(), 1, target.clone(), call.clone());

        // Advance time past eta
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 15,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1);

        // ensure queue cleared
        let q: Map<u64, QueuedTx> = env
            .storage()
            .persistent()
            .get(&QUEUE)
            .unwrap_or(Map::new(&env));
        assert!(!q.contains_key(1));
    }

    #[test]
    #[should_panic(expected = "Not ready")]
    fn execution_too_early_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        Timelock::initialize(env.clone(), admin, 10);
        let target = Address::generate(&env);
        let call = BytesN::from_array(&env, &[0u8; 32]);
        Timelock::queue(env.clone(), 1, target.clone(), call.clone());

        // Advance time below eta
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 5,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1);
    }
}
