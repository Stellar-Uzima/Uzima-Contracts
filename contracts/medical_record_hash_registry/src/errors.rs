use soroban_sdk::{contracterror, symbol_short, Symbol};

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

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::ContractPaused | Error::DeadlineExceeded => symbol_short!("RE_TRY_L"),
        Error::InvalidId | Error::DuplicateRecord | Error::RecordNotFound => {
            symbol_short!("CHK_ID")
        },
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        Error::StorageFull => symbol_short!("CLN_OLD"),
        Error::CrossChainTimeout => symbol_short!("RE_TRY_L"),
        _ => symbol_short!("CONTACT"),
    }
}
