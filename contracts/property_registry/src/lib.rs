#![no_std]
#![forbid(alloc)]
//! # Property Registry
//!
//! On-chain registry of property records: legal owner of record, recorded
//! valuation, and fractional ownership shares expressed in basis points
//! ([`BASIS_POINTS_SCALE`] = 10 000 == 100%).
//!
//! ## Entry points
//!
//! | Group | Functions |
//! | --- | --- |
//! | Lifecycle | `initialize`, `transfer_admin`, `set_registrar` |
//! | Operations | `pause`, `unpause` |
//! | Property lifecycle | `register_property`, `update_valuation`, `transfer_property`, `record_ownership` |
//! | Queries | `get_property`, `ownership_of`, `total_properties`, `admin`, `registrar`, `paused` |
//!
//! Each carries `# Auth`, `# Errors` and `# Events` sections on the definition
//! itself; see [`contract`] for the contract-wide execution ordering.
//!
//! ## Authorisation
//!
//! Three independent authorities — admin, registrar, and the owner of record.
//! `record_ownership` requires **two** of them, which is the point most worth
//! reading before extending this contract: the registrar says who may amend the
//! register, the owner says whose position may be debited, and either alone would
//! be unsafe. See [`contract::require_registrar`] and the `# Auth` section on
//! `record_ownership`.
//!
//! ## Reentrancy
//!
//! There is no cross-contract call in any value path. The one dispatch out of
//! this contract is the one behind `require_auth`, which for an address backed
//! by a custom `SorobanAuthorization` contract is a real contract invocation.
//! The two multi-key entry points therefore take the guard in
//! [`reentrancy_guard`]; see that module for why, and
//! [`reentrancy_tests`] for the regression coverage.
//!
//! ## Tests
//!
//! - [`test`] — behaviour and error paths.
//! - [`reentrancy_tests`] — the guard, driven from a contract that calls back in.
//! - `tests/fuzz/property_registry` — proptest harness for the arguments that
//!   reach storage.
//!
//! ## Example
//!
//! ```rust,ignore
//! use property_registry::{PropertyRegistryContract, PropertyRegistryContractClient};
//! use soroban_sdk::{Env, Address, String};
//!
//! let env = Env::default();
//! let id = env.register_contract(None, PropertyRegistryContract);
//! let client = PropertyRegistryContractClient::new(&env, &id);
//!
//! let admin = Address::generate(&env);
//! let registrar = Address::generate(&env);
//! let owner = Address::generate(&env);
//!
//! client.initialize(&admin, &registrar);
//! client.register_property(
//!     &String::from_str(&env, "PROP-001"),
//!     &owner,
//!     &250_000_00,
//!     &String::from_str(&env, "USD"),
//!     &String::from_str(&env, "ipfs://deed"),
//! );
//! ```

pub mod contract;
pub mod errors;
pub mod reentrancy_guard;
pub mod storage;
pub mod types;

#[cfg(test)]
mod reentrancy_tests;
#[cfg(test)]
mod test;

pub use contract::{PropertyRegistryContract, PropertyRegistryContractClient};
pub use errors::Error;
pub use types::{
    DataKey, Property, BASIS_POINTS_SCALE, DATA_KEY_VARIANT_COUNT,
};
