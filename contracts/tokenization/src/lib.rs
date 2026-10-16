#![no_std]
#![forbid(alloc)]
//! # Tokenization
//!
//! Registry that records tokenized assets and tracks fractional ownership of
//! each asset's supply across holders.
//!
//! ## Scope
//! - Lifecycle: `initialize`, `transfer_admin`, `set_treasury_address`
//! - Access control: `grant_role`, `revoke_role`, `renounce_role`, `has_role`,
//!   `role_members`
//! - Operations: `pause`, `unpause`
//! - Asset lifecycle: `register_asset`, `mint`, `burn`, `transfer`
//! - Queries: `get_asset`, `balance_of`, `total_supply`, `asset_count`
//!
//! ## Roles
//! | Role | Capabilities |
//! | --- | --- |
//! | `DEFAULT_ADMIN_ROLE` | Register assets, rotate admin, grant and revoke roles, pause/unpause |
//! | `ISSUER` | Mint and burn, and only on assets whose recorded `issuer` is the caller |
//! | Holder | Transfer units it holds |
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

pub use access_control::{DEFAULT_ADMIN_ROLE, ISSUER_ROLE};
pub use contract::{TokenizationContract, TokenizationContractClient};
pub use errors::Error;
pub use types::{Asset, DataKey};