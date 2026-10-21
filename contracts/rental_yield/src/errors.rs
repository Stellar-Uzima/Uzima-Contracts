use common_error::CommonError;
use soroban_sdk::contracterror;

/// Module-local error type whose discriminants mirror `common_error::CommonError`
/// (issue #1809: reuse the shared error taxonomy instead of inventing a new
/// one), following the same pattern as `rbac::errors::Error`.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    InvalidInput = 8,
    InvalidState = 9,
    InsufficientFunds = 7,
}

// Soroban 21's contracterror macro requires literal discriminants. Keep them
// aligned with the shared taxonomy at compile time without changing the ABI.
const _: () = {
    assert!(Error::Unauthorized as u32 == CommonError::Unauthorized as u32);
    assert!(Error::NotInitialized as u32 == CommonError::NotInitialized as u32);
    assert!(Error::AlreadyInitialized as u32 == CommonError::AlreadyInitialized as u32);
    assert!(Error::InvalidInput as u32 == CommonError::InvalidInput as u32);
    assert!(Error::InvalidState as u32 == CommonError::InvalidState as u32);
    assert!(Error::InsufficientFunds as u32 == CommonError::InsufficientFunds as u32);
};

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
