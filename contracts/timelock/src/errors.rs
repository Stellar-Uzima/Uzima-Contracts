use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    Unauthorized = 100,

    // --- Input Validation (200–299) ---
    InvalidSignature = 207,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    DeadlineExceeded = 306,
    AlreadyQueued = 375,
    NotQueued = 372,
    NotReady = 376,

    // --- Financial & Resource (500–599) ---
    InsufficientFunds = 500,
    StorageFull = 502,

    // --- Cross-Chain (700–799) ---
    CrossChainTimeout = 702,
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized | Error::AlreadyQueued => symbol_short!("ALREADY"),
        Error::ContractPaused | Error::DeadlineExceeded | Error::CrossChainTimeout => {
            symbol_short!("RE_TRY_L")
        },
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        Error::StorageFull => symbol_short!("CLN_OLD"),
        _ => symbol_short!("CONTACT"),
    }
}
