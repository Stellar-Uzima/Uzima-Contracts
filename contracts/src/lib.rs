//! # shared_types
//!
//! Common error types and codes shared across all Uzima contracts.
//! Import this crate in any contract's `ContractError` via `CommonError`.

#![no_std]

use soroban_sdk::contracterror;

/// Shared error codes used consistently across all contracts.
/// Same variant always maps to the same numeric code.
#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CommonError {
    /// Caller lacks permission to perform this action.
    Unauthorized = 1,
    /// Requested resource does not exist.
    NotFound = 2,
    /// One or more supplied arguments are invalid.
    InvalidInput = 3,
    /// Resource already exists; duplicate creation rejected.
    AlreadyExists = 4,
    /// Caller has exceeded their allowed request rate.
    RateLimitExceeded = 5,
    /// Contract is paused; no state-changing calls are allowed.
    ContractPaused = 6,
}
