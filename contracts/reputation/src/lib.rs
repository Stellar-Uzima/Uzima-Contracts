#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol, symbol_short};

const ADMIN: Symbol = symbol_short!("admin");
const SCORES: Symbol = symbol_short!("scores");

#[contract]
pub struct ReputationSystem;

#[contractimpl]
impl ReputationSystem {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN) { panic!("init"); }
        env.storage().instance().set(&ADMIN, &admin);
    }

    // Read-only view
    pub fn get_score(env: Env, user: Address) -> i128 {
        let scores: Map<Address, i128> = env.storage().persistent().get(&SCORES).unwrap_or(Map::new(&env));
        scores.get(user).unwrap_or(0)
    }

    // Only admin (Governor) can mint reputation
    pub fn mint(env: Env, user: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
        
        let mut scores: Map<Address, i128> = env.storage().persistent().get(&SCORES).unwrap_or(Map::new(&env));
        let current = scores.get(user.clone()).unwrap_or(0);
        scores.set(user, current + amount);
        env.storage().persistent().set(&SCORES, &scores);
    }

    // Slash reputation for bad behavior
    pub fn slash(env: Env, user: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut scores: Map<Address, i128> = env.storage().persistent().get(&SCORES).unwrap_or(Map::new(&env));
        let current = scores.get(user.clone()).unwrap_or(0);
        let new_score = if amount > current { 0 } else { current - amount };
        scores.set(user, new_score);
        env.storage().persistent().set(&SCORES, &scores);
    }
}
