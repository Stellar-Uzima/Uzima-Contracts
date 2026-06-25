use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    Unauthorized = 100,
    UnauthorizedCaller = 101,
    NotAuthorizedPauser = 102,

    // --- Input Validation (200–299) ---
    InvalidAmount = 205,
    InvalidSignature = 207,
    InvalidCoverage = 280,
    PolicyMismatch = 281,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    CircuitOpen = 303,
    InvalidStatus = 304,
    AlreadyInState = 305,
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

    // --- Reentrancy (800–899) ---
    Reentrancy = 800,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::UnauthorizedCaller => "Unauthorized Caller",
            Error::NotAuthorizedPauser => "Not Authorized Pauser",
            Error::InvalidAmount => "Invalid Amount",
            Error::InvalidSignature => "Invalid Signature",
            Error::InvalidCoverage => "Invalid Coverage",
            Error::PolicyMismatch => "Policy Mismatch",
            Error::NotInitialized => "Not Initialized",
            Error::AlreadyInitialized => "Already Initialized",
            Error::ContractPaused => "Contract Paused",
            Error::CircuitOpen => "Circuit Open",
            Error::InvalidStatus => "Invalid Status",
            Error::AlreadyInState => "Already In State",
            Error::DeadlineExceeded => "Deadline Exceeded",
            Error::ClaimNotFound => "Claim Not Found",
            Error::PreAuthNotFound => "Pre Auth Not Found",
            Error::PaymentPlanNotFound => "Payment Plan Not Found",
            Error::InsuranceProviderNotFound => "Insurance Provider Not Found",
            Error::CoveragePolicyNotFound => "Coverage Policy Not Found",
            Error::EligibilityCheckNotFound => "Eligibility Check Not Found",
            Error::ClaimSubmissionNotFound => "Claim Submission Not Found",
            Error::EobNotFound => "Eob Not Found",
            Error::InsufficientFunds => "Insufficient Funds",
            Error::StorageFull => "Storage Full",
            Error::FraudDetected => "Fraud Detected",
            Error::EscrowFailed => "Escrow Failed",
            Error::UnsupportedTransaction => "Unsupported Transaction",
            Error::CrossChainTimeout => "Cross Chain Timeout",
            Error::Reentrancy => "Reentrancy",
        };
        f.write_str(message)
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::ContractPaused | Error::DeadlineExceeded | Error::CrossChainTimeout => {
            symbol_short!("RE_TRY_L")
        },
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        Error::StorageFull => symbol_short!("CLN_OLD"),
        Error::ClaimNotFound
        | Error::PreAuthNotFound
        | Error::PaymentPlanNotFound
        | Error::InsuranceProviderNotFound => symbol_short!("CHK_ID"),
        _ => symbol_short!("CONTACT"),
    }
}
