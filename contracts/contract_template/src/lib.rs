//! # Contract Template
//!
//! Boilerplate for new Soroban contracts. Demonstrates:
//! - Proper `require_auth()` pattern (Issue #439)
//! - Standard initialization guard
//! - Typed errors and events
//! - Storage key namespacing
//!
//! Copy this directory and rename `contract-template` / `ContractTemplate` throughout.

#![no_std]

mod errors;
mod events;
#[cfg(test)]
mod test;
mod types;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, Address, Env, String};
use types::ContractData;

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

const KEY_ADMIN: &str = "Admin";
const KEY_DATA: &str = "Data";

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct ContractTemplate;

#[contractimpl]
impl ContractTemplate {
    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    /// Initialize the contract. Can only be called once.
    ///
    /// # Auth
    /// No auth required — the deployer becomes the admin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&KEY_ADMIN) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&KEY_ADMIN, &admin);
        events::emit_initialized(&env, &admin);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Admin functions
    // -----------------------------------------------------------------------

    /// Transfer admin rights to a new address.
    ///
    /// # Auth
    /// Requires auth from the **current** admin.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin = Self::get_admin(&env)?;
        // Always call require_auth() before any state changes.
        admin.require_auth();

        env.storage().instance().set(&KEY_ADMIN, &new_admin);
        events::emit_admin_transferred(&env, &admin, &new_admin);
        Ok(())
    }

    /// Update the contract's stored data.
    ///
    /// # Auth
    /// Requires auth from the admin.
    pub fn update_data(env: Env, caller: Address, data: String) -> Result<(), Error> {
        // 1. Authenticate the caller first.
        caller.require_auth();

        // 2. Verify the caller has the required role/permission.
        let admin = Self::get_admin(&env)?;
        if caller != admin {
            return Err(Error::Unauthorized);
        }

        // 3. Validate inputs.
        if data.len() > 256 {
            return Err(Error::InputTooLong);
        }

        // 4. Execute the state change.
        let record = ContractData { owner: caller.clone(), value: data.clone() };
        env.storage().persistent().set(&KEY_DATA, &record);

        // 5. Emit an event for auditability.
        events::emit_data_updated(&env, &caller, &data);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Read-only queries
    // -----------------------------------------------------------------------

    /// Return the current admin address.
    pub fn get_admin(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&KEY_ADMIN)
            .ok_or(Error::NotInitialized)
    }

    /// Return the stored data, if any.
    pub fn get_data(env: Env) -> Option<ContractData> {
        env.storage().persistent().get(&KEY_DATA)
    }
}
