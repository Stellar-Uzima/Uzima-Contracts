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
