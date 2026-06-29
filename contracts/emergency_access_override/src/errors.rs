use soroban_sdk::{contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    InvalidThreshold = 230,
    InvalidDuration = 231,
    RecordNotFound = 403,
    RateLimitExceeded = 429,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::InvalidThreshold => write!(f, "invalid threshold"),
            Error::InvalidDuration => write!(f, "invalid duration"),
            Error::RecordNotFound => write!(f, "record not found"),
            Error::RateLimitExceeded => write!(f, "rate limit exceeded"),
        }
    }
}

#[doc(hidden)] /// Reserved for ABI consistency; not currently called from contract code.
#[cfg(test)]
pub fn get_suggestion(error: Error) -> soroban_sdk::Symbol {
    match error {
        Error::Unauthorized => soroban_sdk::symbol_short!("CHK_AUTH"),
        Error::NotInitialized => soroban_sdk::symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => soroban_sdk::symbol_short!("ALREADY"),
        Error::InvalidThreshold | Error::InvalidDuration => soroban_sdk::symbol_short!("CHK_LEN"),
        Error::RecordNotFound => soroban_sdk::symbol_short!("CHK_ID"),
        Error::RateLimitExceeded => soroban_sdk::symbol_short!("WAIT_CD"),
    }
}
