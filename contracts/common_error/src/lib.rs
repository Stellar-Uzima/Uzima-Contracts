#![no_std]
//! # Common Error
//!
//! Shared `CommonError` enum and remediation hints for the Uzima Contracts
//! workspace. Every contract that surfaces a Soroban `#[contracterror]` should
//! map its module-local errors onto this canonical taxonomy so cross-contract
//! tooling (HMIs, dashboards, SDKs, indexers) only needs to learn one shape.
//!
//! ## Layout
//!
//! The error `repr(u32)` is partitioned into reserved ranges:
//!
//! | Range | Owner | Constant |
//! |-------|-------|----------|
//! | `0..=99` | `CommonError` itself (this enum) | [`COMMON_ERROR_MAX`] |
//! | `100..=999` | Generic Soroban contract errors | n/a |
//! | `1000..=1999` | `medical_records` module | [`MEDICAL_RECORDS_ERROR_BASE`] |
//! | `2000..=2999` | `rbac` module | [`RBAC_ERROR_BASE`] |
//!
//! Callers compare `as u32` against the appropriate base rather than forging
//! ad-hoc discriminant checks. New code modules must reserve an unused range
//! here instead of reusing one of the existing partitions.

use soroban_sdk::{contracterror, symbol_short, Symbol};

pub mod schema_versioning;

/// Upper bound (inclusive) of the [`CommonError`] discriminant range.
///
/// Every variant of [`CommonError`] must have a discriminant `<= COMMON_ERROR_MAX`.
/// Values above this constant indicate a module-specific error that should be
/// tagged with a base constant such as [`MEDICAL_RECORDS_ERROR_BASE`].
pub const COMMON_ERROR_MAX: u32 = 99;

/// Base discriminant (inclusive) of the `medical_records` error range.
pub const MEDICAL_RECORDS_ERROR_BASE: u32 = 1000;
/// Base discriminant (inclusive) of the `rbac` error range.
pub const RBAC_ERROR_BASE: u32 = 2000;
pub type CommonResult<T> = Result<T, CommonError>;

#[contracterror(export = false)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CommonError {
    Unknown = 0,
    Unauthorized = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    ContractPaused = 4,
    DeadlineExceeded = 5,
    RateLimitExceeded = 6,
    InsufficientFunds = 7,
    InvalidInput = 8,
    InvalidState = 9,
    NotFound = 10,
    AccessDenied = 11,
    Timeout = 12,
    InvalidArgument = 13,
    ExternalContractNotSet = 14,
    InvalidData = 15,
    InvalidPayload = 16,
    DuplicateSubmission = 17,
    UnauthorizedCaller = 18,
}

impl core::fmt::Display for CommonError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CommonError::Unknown => write!(f, "unknown"),
            CommonError::Unauthorized => write!(f, "unauthorized"),
            CommonError::NotInitialized => write!(f, "not initialized"),
            CommonError::AlreadyInitialized => write!(f, "already initialized"),
            CommonError::ContractPaused => write!(f, "contract paused"),
            CommonError::DeadlineExceeded => write!(f, "deadline exceeded"),
            CommonError::RateLimitExceeded => write!(f, "rate limit exceeded"),
            CommonError::InsufficientFunds => write!(f, "insufficient funds"),
            CommonError::InvalidInput => write!(f, "invalid input"),
            CommonError::InvalidState => write!(f, "invalid state"),
            CommonError::NotFound => write!(f, "not found"),
            CommonError::AccessDenied => write!(f, "access denied"),
            CommonError::Timeout => write!(f, "timeout"),
            CommonError::InvalidArgument => write!(f, "invalid argument"),
            CommonError::ExternalContractNotSet => write!(f, "external contract not set"),
            CommonError::InvalidData => write!(f, "invalid data"),
            CommonError::InvalidPayload => write!(f, "invalid payload"),
            CommonError::DuplicateSubmission => write!(f, "duplicate submission"),
            CommonError::UnauthorizedCaller => write!(f, "unauthorized caller"),
        }
    }
}

pub fn is_common_error_code(code: u32) -> bool {
    code <= COMMON_ERROR_MAX
}

pub fn get_suggestion(error: CommonError) -> Symbol {
    match error {
        CommonError::ContractPaused | CommonError::RateLimitExceeded => symbol_short!("RE_TRY_L"),
        CommonError::Unauthorized | CommonError::UnauthorizedCaller => symbol_short!("CHK_AUTH"),
        CommonError::NotInitialized => symbol_short!("INIT_CTR"),
        CommonError::AlreadyInitialized => symbol_short!("ALREADY"),
        CommonError::InvalidInput | CommonError::InvalidArgument | CommonError::InvalidData => {
            symbol_short!("CHK_DATA")
        },
        CommonError::NotFound => symbol_short!("CHK_ID"),
        CommonError::InsufficientFunds => symbol_short!("ADD_FUND"),
        CommonError::Timeout => symbol_short!("RE_TRY_L"),
        _ => symbol_short!("CONTACT"),
    }
}
