use soroban_sdk::{contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Unauthorized = 100,
    InvalidId = 206,
    InvalidSignature = 207,
    InvalidRecordHash = 251,
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    DeadlineExceeded = 306,
    DuplicateRecord = 402,
    RecordNotFound = 403,
    InsufficientFunds = 500,
    StorageFull = 502,
    CrossChainTimeout = 702,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InvalidId => write!(f, "invalid id"),
            Error::InvalidSignature => write!(f, "invalid signature"),
            Error::InvalidRecordHash => write!(f, "invalid record hash"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::DeadlineExceeded => write!(f, "deadline exceeded"),
            Error::DuplicateRecord => write!(f, "duplicate record"),
            Error::RecordNotFound => write!(f, "record not found"),
            Error::InsufficientFunds => write!(f, "insufficient funds"),
            Error::StorageFull => write!(f, "storage full"),
            Error::CrossChainTimeout => write!(f, "cross chain timeout"),
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
        Error::ContractPaused | Error::DeadlineExceeded => soroban_sdk::symbol_short!("RE_TRY_L"),
        Error::InvalidId | Error::DuplicateRecord | Error::RecordNotFound => {
            soroban_sdk::symbol_short!("CHK_ID")
        },
        Error::InsufficientFunds => soroban_sdk::symbol_short!("ADD_FUND"),
        Error::StorageFull => soroban_sdk::symbol_short!("CLN_OLD"),
        Error::CrossChainTimeout => soroban_sdk::symbol_short!("RE_TRY_L"),
        _ => soroban_sdk::symbol_short!("CONTACT"),
    }
}
