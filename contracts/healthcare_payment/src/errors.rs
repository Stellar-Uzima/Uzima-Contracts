use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    Unauthorized = 100,

    // --- Input Validation (200–299) ---
    InvalidAmount = 205,
    InvalidSignature = 207,
    InvalidCoverage = 280,
    PolicyMismatch = 281,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    InvalidStatus = 304,
    DeadlineExceeded = 306,

    // --- Entity Existence (400–499) ---
    ClaimNotFound = 480,
    PreAuthNotFound = 481,
    PaymentPlanNotFound = 482,
    InsuranceProviderNotFound = 483,
    CoveragePolicyNotFound = 484,
    EligibilityCheckNotFound = 485,
    ClaimSubmissionNotFound = 486,
    EobNotFound = 487,

    // --- Financial & Resource (500–599) ---
    InsufficientFunds = 500,
    StorageFull = 502,
    FraudDetected = 580,
    EscrowFailed = 581,
    UnsupportedTransaction = 582,

    // --- Cross-Chain (700–799) ---
    CrossChainTimeout = 702,
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::ContractPaused | Error::DeadlineExceeded | Error::CrossChainTimeout => {
            symbol_short!("RE_TRY_L")
        }
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        Error::StorageFull => symbol_short!("CLN_OLD"),
        Error::ClaimNotFound
        | Error::PreAuthNotFound
        | Error::PaymentPlanNotFound
        | Error::InsuranceProviderNotFound => symbol_short!("CHK_ID"),
        _ => symbol_short!("CONTACT"),
    }
}
