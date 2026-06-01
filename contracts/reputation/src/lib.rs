#![no_std]

pub mod types;

pub use types::DataKey;
use soroban_sdk::{contract, contracterror, contractimpl, Address, Env, Map};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
}

#[contract]
pub struct ReputationSystem;

#[contractimpl]
impl ReputationSystem {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::ReputAdmin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::ReputAdmin, &admin);
        Ok(())
    }

    // Read-only view
    pub fn get_score(env: Env, user: Address) -> i128 {
        let scores: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&DataKey::ReputScores)
            .unwrap_or(Map::new(&env));
        scores.get(user).unwrap_or(0)
    }

    // Only admin (Governor) can mint reputation
    pub fn mint(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::ReputAdmin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let mut scores: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&DataKey::ReputScores)
            .unwrap_or(Map::new(&env));
        let current = scores.get(user.clone()).unwrap_or(0);
        scores.set(user, current.saturating_add(amount));
        env.storage().persistent().set(&DataKey::ReputScores, &scores);
        Ok(())
    }

    // Slash reputation for bad behavior
    pub fn slash(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::ReputAdmin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let mut scores: Map<Address, i128> = env
            .storage()
            .persistent()
            .get(&DataKey::ReputScores)
            .unwrap_or(Map::new(&env));
        let current = scores.get(user.clone()).unwrap_or(0);
        let new_score = current.saturating_sub(amount).max(0);
        scores.set(user, new_score);
        env.storage().persistent().set(&DataKey::ReputScores, &scores);
        Ok(())
    }
}
