use soroban_sdk::{Address, Env};

use crate::types::{DataKey, Error};

pub fn ensure_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::AdminNotSet)?;

    if admin != *caller {
        return Err(Error::NotAuthorized);
    }

    Ok(())
}

pub fn next_round_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::RoundCounter)
        .unwrap_or(0);
    let next = current.saturating_add(1);
    env.storage().instance().set(&DataKey::RoundCounter, &next);
    next
}
