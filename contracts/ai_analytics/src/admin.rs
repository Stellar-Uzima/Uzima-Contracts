use soroban_sdk::{Address, Env};

use crate::types::{DataKey, Error};

pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
    admin.require_auth();

    if env.storage().instance().has(&DataKey::Admin) {
        return Err(Error::AlreadyInitialized);
    }

    env.storage().instance().set(&DataKey::Admin, &admin);
    Ok(true)
}
