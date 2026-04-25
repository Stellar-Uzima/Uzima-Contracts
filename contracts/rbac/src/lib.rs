#![no_std]

pub mod errors;
pub mod queries;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::Error;
use queries::Queries;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};
use storage::Storage;
use types::{RBACConfig, Role, RoleAssignment};

#[contract]
pub struct RBAC;

#[contractimpl]
impl RBAC {
    pub fn initialize(env: Env, admin: Address, config: RBACConfig) -> Result<(), Error> {
        if Storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();

        Storage::set_admin(&env, &admin);
        Storage::set_config(&env, &config);
        Storage::set_initialized(&env);

        env.events()
            .publish((symbol_short!("INIT"), symbol_short!("RBAC")), &admin);
        Ok(())
    }

    pub fn assign_role(env: Env, address: Address, role: Role) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        let admin = Storage::get_admin(&env);
        admin.require_auth();

        let success = Storage::add_role(&env, &address, role);

        if success {
            let assignment = RoleAssignment {
                address: address.clone(),
                role,
                assigned_at: env.ledger().timestamp(),
                assigned_by: admin,
            };
            Storage::save_assignment(&env, &assignment);

            if let Some(config) = Storage::get_config(&env) {
                if config.emit_events {
                    env.events().publish(
                        (symbol_short!("ROLE"), symbol_short!("ASSIGN")),
                        (address, role),
                    );
                }
            }
        }

        Ok(success)
    }

    pub fn remove_role(env: Env, address: Address, role: Role) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        let admin = Storage::get_admin(&env);
        admin.require_auth();

        let success = Storage::remove_role(&env, &address, role);

        if success {
            if let Some(config) = Storage::get_config(&env) {
                if config.emit_events {
                    env.events().publish(
                        (symbol_short!("ROLE"), symbol_short!("REMOVE")),
                        (address, role),
                    );
                }
            }
        }

        Ok(success)
    }

    pub fn has_role(env: Env, address: Address, role: Role) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Storage::has_role(&env, &address, role))
    }

    pub fn get_roles(env: Env, address: Address) -> Result<Vec<Role>, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::get_roles(&env, &address))
    }

    pub fn has_any_role(env: Env, address: Address, roles: Vec<Role>) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::has_any_role(&env, &address, &roles))
    }

    pub fn has_all_roles(env: Env, address: Address, roles: Vec<Role>) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::has_all_roles(&env, &address, &roles))
    }

    pub fn get_address_roles(env: Env, address: Address) -> Result<types::AddressRoles, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::get_address_role_info(&env, &address))
    }

    pub fn get_role_members(env: Env, role: Role) -> Result<Vec<Address>, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::get_role_members(&env, role))
    }

    pub fn get_role_member_count(env: Env, role: Role) -> Result<u32, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::get_role_member_count(&env, role))
    }

    pub fn is_admin(env: Env, address: Address) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::is_admin(&env, &address))
    }

    pub fn is_doctor(env: Env, address: Address) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::is_doctor(&env, &address))
    }

    pub fn is_patient(env: Env, address: Address) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::is_patient(&env, &address))
    }

    pub fn is_staff(env: Env, address: Address) -> Result<bool, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Ok(Queries::is_staff(&env, &address))
    }

    pub fn update_config(env: Env, config: RBACConfig) -> Result<(), Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        let admin = Storage::get_admin(&env);
        admin.require_auth();

        Storage::set_config(&env, &config);

        env.events()
            .publish((symbol_short!("CONFIG"), symbol_short!("UPDATE")), config);
        Ok(())
    }

    pub fn get_config(env: Env) -> Result<RBACConfig, Error> {
        if !Storage::is_initialized(&env) {
            return Err(Error::NotInitialized);
        }

        Storage::get_config(&env).ok_or(Error::NotInitialized)
    }
}
