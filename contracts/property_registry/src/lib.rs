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
//! - Queries: `get_property`, `ownership_of`, `ownership_percent`,
//!   `total_properties`
//!
//! ## Roles
//! | Role | Capabilities |
//! | --- | --- |
//! | Admin | Rotate admin and registrar, pause/unpause |
//! | Registrar | Register properties, revalue, record ownership shares |
//! | Owner of record | Transfer the property, authorize share movements |
//!
//! ## Design guarantees
//! - **Typed errors.** Every rejection maps to a distinct `Error` variant with a
//!   stable `u32` discriminant, so a client can tell "the id was malformed" from
//!   "the currency was not alphabetic" from "the owner does not hold that much".
//! - **Bounded inputs.** [`validation`] runs before `require_auth`, so a
//!   malformed call cannot be used to probe who holds authority.
//! - **Full audit trail.** Every state change publishes one event carrying the
//!   complete before/after delta, so the register and the ownership split are
//!   reconstructible from the event stream alone.
//! - **Bounded writes.** Each entry point reads each ledger key at most once and
//!   writes back only what changed.
//! - **Consistent splits.** `record_ownership` and `transfer_property` move both
//!   legs together and credit before they debit, so a rejected call leaves the
//!   ownership split untouched and the total never exceeds 100%.

mod contract;
mod errors;
mod events;
mod storage;
mod types;
mod validation;

#[cfg(test)]
mod test;

pub use contract::{PropertyRegistryContract, PropertyRegistryContractClient};
pub use errors::Error;
pub use types::{DataKey, Property, BASIS_POINTS_SCALE};
pub use validation::{
    is_iso_4217, validate_currency, validate_debit, validate_holder,
    validate_metadata_uri, validate_property_id, validate_share,
    validate_share_total, validate_transfer_parties, validate_valuation,
    MAX_CURRENCY_LEN, MAX_METADATA_URI_LEN, MAX_PROPERTY_ID_LEN, MIN_PROPERTY_ID_LEN,
};