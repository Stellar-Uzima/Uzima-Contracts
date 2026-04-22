#![no_std]

use soroban_sdk::{
    contracterror, contracttype, symbol_short, Address, BytesN, Env, String, Symbol, Vec,
};

pub mod migration;

#[cfg(all(test, feature = "testutils"))]
mod test;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeError {
    NotAuthorized = 100,
    InvalidWasmHash = 101,
    VersionAlreadyExists = 102,
    MigrationFailed = 103,
    IncompatibleVersion = 104,
    ContractPaused = 105,
    HistoryNotFound = 106,
    IntegrityCheckFailed = 107,
    DeprecatedFunctionNotTracked = 108,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UpgradeHistory {
    pub wasm_hash: BytesN<32>,
    pub version: u32,
    pub upgraded_at: u64,
    pub description: Symbol,
    pub state_hash: BytesN<32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DeprecatedFunction {
    pub function: Symbol,
    pub since: String,
    pub replacement: Option<Symbol>,
    pub removed_in: Option<String>,
    pub note: String,
    pub migration_guide: Option<String>,
}

pub mod storage {
    use super::*;

    pub const VERSION: Symbol = symbol_short!("VERSION");
    pub const ADMIN: Symbol = symbol_short!("UP_ADMIN");
    pub const HISTORY: Symbol = symbol_short!("HISTORY");
    pub const IS_FROZEN: Symbol = symbol_short!("FROZEN");
    pub const DEPRECATED_FUNCTIONS: Symbol = symbol_short!("DEPRLIST");

    pub fn get_version(env: &Env) -> u32 {
        env.storage().instance().get(&VERSION).unwrap_or(0)
    }

    pub fn set_version(env: &Env, version: u32) {
        env.storage().instance().set(&VERSION, &version);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&ADMIN)
    }

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&ADMIN, admin);
    }

    pub fn is_frozen(env: &Env) -> bool {
        env.storage().instance().get(&IS_FROZEN).unwrap_or(false)
    }

    pub fn freeze(env: &Env) {
        env.storage().instance().set(&IS_FROZEN, &true);
    }

    pub fn add_history(env: &Env, history: UpgradeHistory) {
        let mut list: Vec<UpgradeHistory> = env
            .storage()
            .persistent()
            .get(&HISTORY)
            .unwrap_or(Vec::new(env));
        list.push_back(history);
        env.storage().persistent().set(&HISTORY, &list);
    }

    pub fn get_history(env: &Env) -> Vec<UpgradeHistory> {
        env.storage()
            .persistent()
            .get(&HISTORY)
            .unwrap_or(Vec::new(env))
    }

    pub fn set_deprecated_functions(env: &Env, deprecations: &Vec<DeprecatedFunction>) {
        env.storage()
            .instance()
            .set(&DEPRECATED_FUNCTIONS, deprecations);
    }

    pub fn get_deprecated_functions(env: &Env) -> Vec<DeprecatedFunction> {
        env.storage()
            .instance()
            .get(&DEPRECATED_FUNCTIONS)
            .unwrap_or(Vec::new(env))
    }
}

pub fn authorize_upgrade(env: &Env) -> Result<Address, UpgradeError> {
    if storage::is_frozen(env) {
        return Err(UpgradeError::ContractPaused);
    }
    let admin = storage::get_admin(env).ok_or(UpgradeError::NotAuthorized)?;
    admin.require_auth();
    Ok(admin)
}

pub fn execute_upgrade<T: migration::Migratable>(
    env: &Env,
    new_wasm_hash: BytesN<32>,
    new_version: u32,
    description: Symbol,
) -> Result<(), UpgradeError> {
    execute_upgrade_with_deprecations(
        env,
        new_wasm_hash,
        new_version,
        description,
        Vec::new(env),
    )
}

pub fn execute_upgrade_with_deprecations<T: migration::Migratable>(
    env: &Env,
    new_wasm_hash: BytesN<32>,
    new_version: u32,
    description: Symbol,
    deprecations: Vec<DeprecatedFunction>,
) -> Result<(), UpgradeError> {
    authorize_upgrade(env)?;

    let current_version = storage::get_version(env);
    if new_version <= current_version {
        return Err(UpgradeError::IncompatibleVersion);
    }

    // Optional pre-migration integrity check
    T::verify_integrity(env).map_err(|_| UpgradeError::IntegrityCheckFailed)?;

    // Perform migration
    T::migrate(env, current_version)?;

    // Post-migration integrity check
    let state_hash = T::verify_integrity(env).map_err(|_| UpgradeError::IntegrityCheckFailed)?;

    storage::add_history(
        env,
        UpgradeHistory {
            wasm_hash: new_wasm_hash.clone(),
            version: new_version,
            upgraded_at: env.ledger().timestamp(),
            description,
            state_hash,
        },
    );

    storage::set_deprecated_functions(env, &deprecations);
    storage::set_version(env, new_version);
    env.deployer().update_current_contract_wasm(new_wasm_hash);

    Ok(())
}

pub fn rollback(env: &Env) -> Result<(), UpgradeError> {
    authorize_upgrade(env)?;

    let history = storage::get_history(env);
    if history.len() < 2 {
        return Err(UpgradeError::HistoryNotFound);
    }

    // To rollback, we go to the second to last version in history
    let last_index = history
        .len()
        .checked_sub(2)
        .ok_or(UpgradeError::HistoryNotFound)?;
    let target_version = history
        .get(last_index)
        .ok_or(UpgradeError::HistoryNotFound)?;

    let current_version = storage::get_version(env);
    let next_version = current_version
        .checked_add(1)
        .ok_or(UpgradeError::IncompatibleVersion)?;
    storage::set_version(env, next_version);
    env.deployer()
        .update_current_contract_wasm(target_version.wasm_hash);

    Ok(())
}

pub fn set_deprecated_functions(
    env: &Env,
    deprecations: Vec<DeprecatedFunction>,
) -> Result<(), UpgradeError> {
    authorize_upgrade(env)?;
    storage::set_deprecated_functions(env, &deprecations);

    env.events().publish(
        (Symbol::new(env, "DeprecationsUpdated"),),
        deprecations.len(),
    );

    Ok(())
}

pub fn get_deprecated_functions(env: &Env) -> Vec<DeprecatedFunction> {
    storage::get_deprecated_functions(env)
}

pub fn get_deprecated_function(env: &Env, function: Symbol) -> Option<DeprecatedFunction> {
    let deprecations = storage::get_deprecated_functions(env);
    let mut index = 0;
    while index < deprecations.len() {
        let deprecation = deprecations.get(index).unwrap();
        if deprecation.function == function {
            return Some(deprecation);
        }
        index += 1;
    }

    None
}

pub fn emit_deprecation_warning(env: &Env, function: Symbol) -> Result<(), UpgradeError> {
    let deprecation = get_deprecated_function(env, function.clone())
        .ok_or(UpgradeError::DeprecatedFunctionNotTracked)?;

    env.events().publish(
        (Symbol::new(env, "Deprecated"), function),
        deprecation.note,
    );

    Ok(())
}
