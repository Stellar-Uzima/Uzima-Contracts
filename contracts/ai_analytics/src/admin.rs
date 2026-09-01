use soroban_sdk::{symbol_short, Address, Env};

use crate::types::{DataKey, Error};

#[must_use]
pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
    governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
    admin.require_auth();

    env.storage().instance().set(&DataKey::Admin, &admin);
    env.events().publish(
        (symbol_short!("AI_AN"), symbol_short!("INIT")),
        admin,
    );
    Ok(true)
}
