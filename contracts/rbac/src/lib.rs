#![no_std]

pub mod types;
pub mod storage;
pub mod queries;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Env, Vec,
};
use types::{Role, RBACConfig, RoleAssignment, DataKey};
use storage::Storage;
use queries::Queries;

#[contract]
pub struct RBAC;

#[contractimpl]
impl RBAC {
    /// Initialize the RBAC contract
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address (usually the contract deployer)
    /// * `config` - RBAC configuration
    ///
    /// # Panics
    /// Panics if already initialized
    pub fn initialize(env: Env, admin: Address, config: RBACConfig) {
        if Storage::is_initialized(&env) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        Storage::set_admin(&env, &admin);
        Storage::set_config(&env, &config);
        Storage::set_initialized(&env);

        env.events().publish(
            (symbol_short!("INIT"), symbol_short!("RBAC")),
            &admin,
        );
    }

    /// Assign a role to an address (admin only)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to assign the role to
    /// * `role` - The role to assign
    ///
    /// # Returns
    /// true if role was assigned, false if already had role or max roles exceeded
    ///
    /// # Panics
    /// Panics if caller is not admin or contract not initialized
    pub fn assign_role(
        env: Env,
        address: Address,
        role: Role,
    ) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        let admin = Storage::get_admin(&env);
        admin.require_auth();

        // Add the role
        let success = Storage::add_role(&env, &address, role);

        if success {
            // Save assignment record
            let assignment = RoleAssignment {
                address: address.clone(),
                role,
                assigned_at: env.ledger().timestamp(),
                assigned_by: admin,
            };
            Storage::save_assignment(&env, &assignment);

            // Emit event
            if let Some(config) = Storage::get_config(&env) {
                if config.emit_events {
                    env.events().publish(
                        (symbol_short!("ROLE"), symbol_short!("ASSIGN")),
                        (address, role),
                    );
                }
            }
        }

        success
    }

    /// Remove a role from an address (admin only)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to remove the role from
    /// * `role` - The role to remove
    ///
    /// # Returns
    /// true if role was removed, false if address didn't have that role
    ///
    /// # Panics
    /// Panics if caller is not admin or contract not initialized
    pub fn remove_role(
        env: Env,
        address: Address,
        role: Role,
    ) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        let admin = Storage::get_admin(&env);
        admin.require_auth();

        // Remove the role
        let success = Storage::remove_role(&env, &address, role);

        if success {
            // Emit event
            if let Some(config) = Storage::get_config(&env) {
                if config.emit_events {
                    env.events().publish(
                        (symbol_short!("ROLE"), symbol_short!("REMOVE")),
                        (address, role),
                    );
                }
            }
        }

        success
    }

    /// Check if an address has a specific role
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check
    /// * `role` - The role to check for
    ///
    /// # Returns
    /// true if address has the role, false otherwise
    pub fn has_role(env: Env, address: Address, role: Role) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Storage::has_role(&env, &address, role)
    }

    /// Get all roles for an address
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to get roles for
    ///
    /// # Returns
    /// Vector of roles assigned to the address
    pub fn get_roles(env: Env, address: Address) -> Vec<Role> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::get_roles(&env, &address)
    }

    /// Check if an address has any of the specified roles
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check
    /// * `roles` - Vector of roles to check against
    ///
    /// # Returns
    /// true if address has any of the specified roles
    pub fn has_any_role(env: Env, address: Address, roles: Vec<Role>) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::has_any_role(&env, &address, &roles)
    }

    /// Check if an address has all of the specified roles
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check
    /// * `roles` - Vector of roles to check for
    ///
    /// # Returns
    /// true if address has all specified roles
    pub fn has_all_roles(env: Env, address: Address, roles: Vec<Role>) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::has_all_roles(&env, &address, &roles)
    }

    /// Get role information for an address
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to get info for
    ///
    /// # Returns
    /// AddressRoles struct with all roles and count
    pub fn get_address_roles(env: Env, address: Address) -> types::AddressRoles {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::get_address_role_info(&env, &address)
    }

    /// Get all members of a specific role
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `role` - The role to get members for
    ///
    /// # Returns
    /// Vector of all addresses with the specified role
    pub fn get_role_members(env: Env, role: Role) -> Vec<Address> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::get_role_members(&env, role)
    }

    /// Get count of addresses with a specific role
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `role` - The role to count members for
    ///
    /// # Returns
    /// Number of addresses with the specified role
    pub fn get_role_member_count(env: Env, role: Role) -> u32 {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::get_role_member_count(&env, role)
    }

    /// Check if an address is an admin
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if address is an admin
    pub fn is_admin(env: Env, address: Address) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::is_admin(&env, &address)
    }

    /// Check if an address is a doctor
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if address is a doctor
    pub fn is_doctor(env: Env, address: Address) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::is_doctor(&env, &address)
    }

    /// Check if an address is a patient
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if address is a patient
    pub fn is_patient(env: Env, address: Address) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::is_patient(&env, &address)
    }

    /// Check if an address is staff
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// true if address is staff
    pub fn is_staff(env: Env, address: Address) -> bool {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::is_staff(&env, &address)
    }

    /// Update RBAC configuration (admin only)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `config` - New configuration
    pub fn update_config(env: Env, config: RBACConfig) {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        let admin = Storage::get_admin(&env);
        admin.require_auth();

        Storage::set_config(&env, &config);

        env.events().publish(
            (symbol_short!("CONFIG"), symbol_short!("UPDATE")),
            config,
        );
    }

    /// Get current RBAC configuration
    ///
    /// # Returns
    /// The current RBACConfig
    pub fn get_config(env: Env) -> RBACConfig {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Storage::get_config(&env).expect("Config not set")
    }
}
