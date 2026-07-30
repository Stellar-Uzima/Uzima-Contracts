#![no_std]
#![allow(clippy::too_many_arguments)]

//! # Validation Utilities Library
//!
//! This library provides comprehensive validation functions for smart contracts.
//! It ensures data integrity and prevents invalid states by validating all input parameters
//! before they are stored in the contract state.
//!
//! ## Features
//! - String validation (length, character sets, format)
//! - Address validation (non-zero, valid format)
//! - Numeric range validation (positive/negative checks, range bounds)
//! - Collection validation (vectors, maps)
//! - Timestamp validation
//! - Custom error types for clear error reporting
//! - Gas-optimized validation checks

pub mod errors;
pub mod string;
pub mod numeric;
pub mod address;
pub mod collections;

pub use errors::ValidationError;
pub use string::*;
pub use numeric::*;
pub use address::*;
pub use collections::*;

// Re-initialization guard, re-exported from `governance_commons`.
//
// The canonical implementation lives in `governance_commons::init_guard`; it is
// surfaced here so contracts that already depend on `validation_utils` for input
// checks can pull the re-initialization guard from the same crate. See
// `docs/SECURITY_CHECKLIST.md` (Item 4) for the required usage.
//
// Semantics: the first init call succeeds; every later call is rejected
// (`init_guard` panics, `try_init_guard` returns
// `GovernanceError::AlreadyInitialized`). Admin transfer is a separate,
// independent operation and never re-opens initialization.
//
// The group below re-exports both the `init_guard` module path and the guard
// functions, so `validation_utils::init_guard(&env)` and
// `validation_utils::try_init_guard(&env)` both resolve.
pub use governance_commons::{
    init_guard, is_initialized, require_initialized, try_init_guard, GovernanceError,
    INIT_GUARD_KEY,
};
