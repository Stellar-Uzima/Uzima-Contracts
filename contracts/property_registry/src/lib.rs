#![no_std]
#![forbid(alloc)]
//! # Property Registry
//!
//! On-chain registry of property records: legal owner of record, recorded
//! valuation, and fractional ownership shares expressed in basis points.
//!
//! ## Scope
//! - Lifecycle: `initialize`, `transfer_admin`, `set_registrar`
//! - Access control: `grant_role`, `revoke_role`, `renounce_role`, `has_role`,
//!   `role_members`
//! - Operations: `pause`, `unpause`
//! - Property lifecycle: `register_property`, `update_valuation`,
//!   `transfer_property`, `record_ownership`
//! - Queries: `get_property`, `ownership_of`, `total_properties`
//!
//! ## Roles
//! | Role | Capabilities |
//! | --- | --- |
//! | `DEFAULT_ADMIN_ROLE` | Rotate admin and registrar, grant and revoke roles, pause/unpause |
//! | `REGISTRAR` | Register properties, revalue, record ownership shares |
//! | Owner of record | Transfer the property, authorize debits from their position |
//!
//! See `src/access_control.rs` for why these primitives live in-tree rather
//! than being pulled from an OpenZeppelin Soroban crate.

mod access_control;
mod contract;
mod errors;
mod storage;
mod types;

#[cfg(test)]
mod invariant_tests;
#[cfg(test)]
mod test;

pub use access_control::{DEFAULT_ADMIN_ROLE, REGISTRAR_ROLE};
pub use contract::{PropertyRegistryContract, PropertyRegistryContractClient};
pub use errors::Error;
pub use types::{DataKey, Property, BASIS_POINTS_SCALE};
