//! # Contract Template
//!
//! Boilerplate for new Soroban contracts. Demonstrates:
//! - Proper `require_auth()` pattern
//! - Standard initialization guard
//! - Typed errors and events
//! - Storage key namespacing
//!
//! To create a new contract, run:
//! ```bash
//! ./scripts/scaffold-contract.sh <your_contract_name>
//! ```
//!
//! Then verify it with:
//! ```bash
//! ./scripts/smoke-test-scaffold.sh <your_contract_name>
//! ```

#![no_std]
#![forbid(alloc)]

mod errors;
mod events;
#[cfg(test)]
mod test;
mod types;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, Address, Env, String};
use types::ContractData;
use soroban_sdk::contracttype;

#[contracttype]
pub enum DataKey {
    Admin,
    Data,
}


// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------




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
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
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

        env.storage().instance().set(&DataKey::Admin, &new_admin);
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
        let record = ContractData {
            owner: caller.clone(),
            value: data.clone(),
        };
        env.storage().persistent().set(&DataKey::Data, &record);

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
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Return the stored data, if any.
    pub fn get_data(env: Env) -> Option<ContractData> {
        env.storage().persistent().get(&DataKey::Data)
    }
}
