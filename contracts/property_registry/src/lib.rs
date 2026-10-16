#![no_std]
#![forbid(alloc)]
//! # Property Registry
//!
//! On-chain registry of property records: legal owner of record, recorded
//! valuation, and fractional ownership shares expressed in basis points.
//!
//! ## Scope
//! - Lifecycle: `initialize`, `transfer_admin`, `set_registrar`
//! - Operations: `pause`, `unpause`
//! - Property lifecycle: `register_property`, `update_valuation`,
//!   `transfer_property`, `record_ownership`
//! - Queries: `get_property`, `ownership_of`, `total_properties`
//! - Upgrade safety: `storage_version`, `storage_gap_slots`,
//!   `is_reserved_gap_slot`
//!
//! ## Roles
//! | Role | Capabilities |
//! | --- | --- |
//! | Admin | Rotate admin and registrar, pause/unpause |
//! | Registrar | Register properties, revalue, record ownership shares |
//! | Owner | Transfer the property of record, authorize share transfers |
//!
//! ## Arithmetic
//! Every `u128` addition and subtraction goes through
//! [`checked_math`](crate::checked_math) so a pathological amount surfaces as
//! `ArithmeticOverflow` / `ArithmeticUnderflow` instead of a trap. Ownership
//! shares are additionally bounded by [`BASIS_POINTS_SCALE`], and the two legs
//! of a share movement are credited before they are debited so a rejected call
//! leaves the split untouched.

mod checked_math;
mod contract;
mod errors;
mod storage;
mod types;

#[cfg(test)]
mod overflow_tests;
#[cfg(test)]
mod test;

pub use checked_math::{bps_to_percent, checked_add, checked_mul, checked_sub};
pub use contract::{PropertyRegistryContract, PropertyRegistryContractClient};
pub use errors::Error;
pub use types::{DataKey, Property, BASIS_POINTS_SCALE, STORAGE_GAP_SLOTS, STORAGE_SCHEMA_VERSION};