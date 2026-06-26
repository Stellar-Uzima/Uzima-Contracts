//! # Role-Based Access Control (RBAC) Contract
//!
//! Provides role-based access control for Uzima healthcare contracts.
//! Supports multiple role types including Admin, Doctor, Patient, Staff,
//! Insurer, Researcher, Auditor, and Service.
//!
//! ## Purpose
//! Enables fine-grained access control across all Uzima contracts by assigning
//! roles to addresses. Roles determine what actions an address can perform.
//!
//! ## Key Dependencies
//! - None (standalone contract)
//!
//! ## Initialization Requirements
//! - Must be initialized with an admin address and RBACConfig
//!
//! ## Role/Permission Requirements
//! - **Admin**: Can assign/remove roles and update config
//! - **Anyone**: Can query role information (read-only)
//!
//! ## Supported Roles
//! - `Admin` - System administration
//! - `Doctor` - Healthcare provider
//! - `Patient` - Healthcare recipient
//! - `Staff` - Support staff
//! - `Insurer` - Insurance provider
//! - `Researcher` - Medical researcher
//! - `Auditor` - Compliance auditor
//! - `Service` - Automated service account
//!
//! ## Error Ranges
//! - 100: Unauthorized
//! - 300-301: Lifecycle & State
//!
//! ## Example Usage
//! ```rust,ignore
//! client.initialize(&admin, &config);
//! client.assign_role(&doctor_addr, &Role::Doctor);
//! let is_doctor = client.has_role(&doctor_addr, &Role::Doctor);
//! ```

#![no_std]

pub mod errors;
pub mod events;
pub mod queries;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::Error;
use events::{emit_initialized, emit_role_assigned, emit_role_removed};
use queries::Queries;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Vec};
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

        emit_initialized(&env, admin);
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
                assigned_by: admin.clone(),
            };
            let role_str = match role {
                Role::Admin => String::from_str(&env, "Admin"),
                Role::Doctor => String::from_str(&env, "Doctor"),
                Role::Patient => String::from_str(&env, "Patient"),
                Role::Staff => String::from_str(&env, "Staff"),
                Role::Insurer => String::from_str(&env, "Insurer"),
                Role::Researcher => String::from_str(&env, "Researcher"),
                Role::Auditor => String::from_str(&env, "Auditor"),
                Role::Service => String::from_str(&env, "Service"),
            };
            Storage::save_assignment(&env, &assignment);
            emit_role_assigned(&env, admin, address, role_str, success);
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
            let role_str = match role {
                Role::Admin => String::from_str(&env, "Admin"),
                Role::Doctor => String::from_str(&env, "Doctor"),
                Role::Patient => String::from_str(&env, "Patient"),
                Role::Staff => String::from_str(&env, "Staff"),
                Role::Insurer => String::from_str(&env, "Insurer"),
                Role::Researcher => String::from_str(&env, "Researcher"),
                Role::Auditor => String::from_str(&env, "Auditor"),
                Role::Service => String::from_str(&env, "Service"),
            };
            emit_role_removed(&env, admin, address, role_str, success);
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

    // ─── Private helpers ─────────────────────────────────────────────────────

    /// Verifies that `caller` is the stored admin address.
    ///
    /// This function satisfies the signature expected by `require_admin!(env, caller)`
    /// from `governance_commons`. RBAC admin functions currently use a different
    /// auth pattern (the stored admin is fetched and `.require_auth()` is called on
    /// it directly, rather than passing `caller` as an explicit parameter). Adding
    /// `governance_commons` as a dependency and introducing a `caller: Address`
    /// parameter to `assign_role`, `remove_role`, and `update_config` would allow
    /// these functions to use `require_admin!(env, caller)` uniformly.
    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin = Storage::get_admin(env);
        if admin != *caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    /// Verifies that `caller` holds `role`.
    ///
    /// This function satisfies the signature expected by `require_role!(env, caller, role)`
    /// from `governance_commons`. Add `governance_commons` as a dependency and call
    /// `require_role!(env, caller, role)` in any function that requires a specific role.
    fn require_role(env: &Env, caller: &Address, role: Role) -> Result<(), Error> {
        if !Storage::has_role(env, caller, role) {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}
