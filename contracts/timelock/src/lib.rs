#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelockConfig {
    pub admin: Address,
    pub delay: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedTx {
    pub target: Address,
    pub call: Symbol,
    pub eta: u64,
}

#[contract]
pub struct Timelock;

#[contractimpl]
impl Timelock {
    pub fn initialize(env: Env, admin: Address, delay: u64) {
        let cfg = TimelockConfig { admin, delay };
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "cfg"), &cfg);
    }

    pub fn get_config(env: Env) -> Option<TimelockConfig> {
        env.storage().instance().get(&Symbol::new(&env, "cfg"))
    }

    pub fn queue_transaction(env: Env, id: u64, target: Address, call: Symbol, eta: u64) {
        let mut q: Map<u64, QueuedTx> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "q"))
            .unwrap_or(Map::new(&env));
        q.set(id, QueuedTx { target, call, eta });
        env.storage().instance().set(&Symbol::new(&env, "q"), &q);
    }
}
