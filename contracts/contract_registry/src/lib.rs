#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAdmin = 3,
    ContractAlreadyRegistered = 4,
    ContractNotRegistered = 5,
    InvalidName = 6,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Contract(String),
}

#[contractclient(name = "ContractRegistryClient")]
pub trait ContractRegistryClientInterface {
    fn initialize(env: Env, admin: Address);
    fn register_contract(env: Env, caller: Address, name: String, address: Address) -> Result<(), RegistryError>;
    fn unregister_contract(env: Env, caller: Address, name: String) -> Result<(), RegistryError>;
    fn get_contract(env: Env, name: String) -> Result<Option<Address>, RegistryError>;
    fn has_contract(env: Env, name: String) -> Result<bool, RegistryError>;
}

#[contract]
pub struct ContractRegistry;

#[contractimpl]
impl ContractRegistry {
    pub fn initialize(env: Env, admin: Address) -> Result<(), RegistryError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(RegistryError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);

        Ok(())
    }

    pub fn register_contract(
        env: Env,
        caller: Address,
        name: String,
        address: Address,
    ) -> Result<(), RegistryError> {
        if name.is_empty() {
            return Err(RegistryError::InvalidName);
        }

        Self::require_initialized(&env)?;
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Contract(name.clone());
        if env.storage().persistent().has(&key) {
            return Err(RegistryError::ContractAlreadyRegistered);
        }

        env.storage().persistent().set(&key, &address);
        Ok(())
    }

    pub fn unregister_contract(
        env: Env,
        caller: Address,
        name: String,
    ) -> Result<(), RegistryError> {
        Self::require_initialized(&env)?;
        Self::require_admin(&env, &caller)?;

        let key = DataKey::Contract(name.clone());
        if !env.storage().persistent().has(&key) {
            return Err(RegistryError::ContractNotRegistered);
        }

        env.storage().persistent().remove(&key);
        Ok(())
    }

    pub fn get_contract(
        env: Env,
        name: String,
    ) -> Result<Option<Address>, RegistryError> {
        Self::require_initialized(&env)?;
        if name.is_empty() {
            return Err(RegistryError::InvalidName);
        }

        Ok(env.storage().persistent().get(&DataKey::Contract(name)))
    }

    pub fn has_contract(env: Env, name: String) -> Result<bool, RegistryError> {
        Self::require_initialized(&env)?;
        if name.is_empty() {
            return Err(RegistryError::InvalidName);
        }

        Ok(env.storage().persistent().has(&DataKey::Contract(name)))
    }

    fn require_initialized(env: &Env) -> Result<(), RegistryError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(RegistryError::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), RegistryError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RegistryError::NotInitialized)?;

        if caller == &admin {
            Ok(())
        } else {
            Err(RegistryError::NotAdmin)
        }
    }
}

#[cfg(test)]
mod test;
