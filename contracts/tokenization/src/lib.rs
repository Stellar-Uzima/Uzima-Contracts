#![no_std]
#![forbid(alloc)]
//! # Tokenization
//!
//! Registry that records tokenized assets and tracks fractional ownership of
//! each asset's supply across holders.
//!
//! ## Scope
//! - Lifecycle: `initialize`, `transfer_admin`, `set_treasury_address`
//! - Operations: `pause`, `unpause`
//! - Asset lifecycle: `register_asset`, `mint`, `burn`, `transfer`
//! - Queries: `get_asset`, `balance_of`, `total_supply`, `supply_cap`,
//!   `asset_count`
//!
//! ## Roles
//! | Role | Capabilities |
//! | --- | --- |
//! | Admin | Register assets, pause/unpause, rotate admin and treasury |
//! | Issuer | Mint and burn units of assets it registered |
//! | Holder | Transfer units it holds |
//!
//! ## Design guarantees
//! - **Typed errors.** Every rejection maps to a distinct `Error` variant, so a
//!   client can tell "the amount was zero" from "the balance was short" from
//!   "the supply cap was reached" without parsing a string.
//! - **Bounded inputs.** [`validation`] runs before `require_auth`, so a
//!   malformed call costs nothing and reveals nothing about who holds authority.
//! - **Full audit trail.** Every state change publishes one event carrying the
//!   complete before/after delta, so contract state is reconstructible from the
//!   event stream alone.
//! - **Bounded writes.** Each entry point reads each ledger key at most once and
//!   writes back only what changed — `transfer`, for instance, never rewrites
//!   the asset record because it does not touch supply.

mod contract;
mod errors;
mod events;
mod storage;
mod types;
mod validation;

#[cfg(test)]
mod test;

pub use contract::{TokenizationContract, TokenizationContractClient};
pub use errors::Error;
pub use types::{Asset, DataKey};
pub use validation::{
    validate_amount, validate_asset_id, validate_decimals, validate_debit,
    validate_metadata_uri, validate_supply_cap, validate_transfer_parties,
    MAX_ASSET_ID_LEN, MAX_DECIMALS, MAX_METADATA_URI_LEN, MIN_ASSET_ID_LEN,
};
