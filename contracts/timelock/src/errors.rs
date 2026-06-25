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

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::InvalidSignature => "Invalid Signature",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::ContractPaused => "Contract Paused",
            Error::DeadlineExceeded => "Deadline Exceeded",
            Error::AlreadyQueued => "Already Queued",
            Error::NotQueued => "Not Queued",
            Error::NotReady => "Not Ready",
            Error::InsufficientFunds => "Insufficient Funds",
            Error::StorageFull => "Storage Full",
            Error::CrossChainTimeout => "Cross Chain Timeout",
        };
        f.write_str(message)
    }
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
