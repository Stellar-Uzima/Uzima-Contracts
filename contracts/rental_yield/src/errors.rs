use common_error::CommonError;
use soroban_sdk::contracterror;

/// Module-local error type whose discriminants mirror `common_error::CommonError`
/// (issue #1809: reuse the shared error taxonomy instead of inventing a new
/// one), following the same pattern as `rbac::errors::Error`.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = CommonError::Unauthorized as u32,
    NotInitialized = CommonError::NotInitialized as u32,
    AlreadyInitialized = CommonError::AlreadyInitialized as u32,
    InvalidInput = CommonError::InvalidInput as u32,
    InvalidState = CommonError::InvalidState as u32,
    InsufficientFunds = CommonError::InsufficientFunds as u32,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::InvalidState => write!(f, "invalid state"),
            Error::InsufficientFunds => write!(f, "insufficient funds"),
        }
    }
}
