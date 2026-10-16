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
//! - Queries: `get_asset`, `balance_of`, `total_supply`, `asset_count`
//! - Upgrade safety: `storage_version`, `storage_gap_slots`,
//!   `is_reserved_gap_slot`
//!
//! ## Roles
//! | Role | Capabilities |
//! | --- | --- |
//! | Admin | Register assets, pause/unpause, rotate admin and treasury |
//! | Issuer | Mint and burn units of assets it registered |
//! | Holder | Transfer units it holds |
//!
//! ## Arithmetic
//! Every `u128` addition and subtraction goes through
//! [`checked_math`](crate::checked_math) so a pathological amount surfaces as
//! `ArithmeticOverflow` / `ArithmeticUnderflow` instead of a trap. See
//! [`Error::InsufficientBalance`] for the pre-check that distinguishes a short
//! balance from a pathological amount.

mod checked_math;
mod contract;
mod errors;
mod storage;
mod types;

#[cfg(test)]
mod overflow_tests;
#[cfg(test)]
mod test;

pub use checked_math::{checked_add, checked_mul, checked_sub};
pub use contract::{TokenizationContract, TokenizationContractClient};
pub use errors::Error;
pub use types::{Asset, DataKey, STORAGE_GAP_SLOTS, STORAGE_SCHEMA_VERSION};