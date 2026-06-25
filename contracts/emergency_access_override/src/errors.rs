use soroban_sdk::{contracterror, symbol_short, Symbol};

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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::InvalidThreshold => "Invalid Threshold",
            Error::InvalidDuration => "Invalid Duration",
            Error::RecordNotFound => "Record Not Found",
            Error::RateLimitExceeded => "Rate Limit Exceeded",
        };
        f.write_str(message)
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::InvalidThreshold | Error::InvalidDuration => symbol_short!("CHK_LEN"),
        Error::RecordNotFound => symbol_short!("CHK_ID"),
        Error::RateLimitExceeded => symbol_short!("WAIT_CD"),
    }
}
