use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Authorization (100–199)
    Unauthorized = 100,
    NotAuthorizedPauser = 101,

    // Input Validation (200–299)
    InvalidAmount = 205,
    InvalidCoverage = 210,
    InvalidStatus = 211,

    // Lifecycle & State (300–399)
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    AlreadyInState = 303,

    // Entity Existence (400–499)
    ClaimNotFound = 480,
    PreAuthNotFound = 481,
    PaymentPlanNotFound = 482,
    InsuranceProviderNotFound = 483,
    CoveragePolicyNotFound = 484,
    EligibilityCheckNotFound = 485,
    ClaimSubmissionNotFound = 486,
    EobNotFound = 487,
    PolicyMismatch = 488,

    // Financial & Resource (500–599)
    InsufficientFunds = 500,
    EscrowFailed = 510,
    Arithmetic = 580,

    // Transactions (600–699)
    UnsupportedTransaction = 600,

    // Security & Fraud (700–799)
    FraudDetected = 700,
    Reentrancy = 800,

    // Circuit Breaker (900–999)
    CircuitOpen = 900,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::NotAuthorizedPauser => write!(f, "not authorized pauser"),
            Error::InvalidAmount => write!(f, "invalid amount"),
            Error::InvalidCoverage => write!(f, "invalid coverage"),
            Error::InvalidStatus => write!(f, "invalid status"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::AlreadyInState => write!(f, "already in state"),
            Error::ClaimNotFound => write!(f, "claim not found"),
            Error::PreAuthNotFound => write!(f, "pre-auth not found"),
            Error::PaymentPlanNotFound => write!(f, "payment plan not found"),
            Error::InsuranceProviderNotFound => write!(f, "insurance provider not found"),
            Error::CoveragePolicyNotFound => write!(f, "coverage policy not found"),
            Error::EligibilityCheckNotFound => write!(f, "eligibility check not found"),
            Error::ClaimSubmissionNotFound => write!(f, "claim submission not found"),
            Error::EobNotFound => write!(f, "EOB not found"),
            Error::PolicyMismatch => write!(f, "policy mismatch"),
            Error::InsufficientFunds => write!(f, "insufficient funds"),
            Error::EscrowFailed => write!(f, "escrow failed"),
            Error::Arithmetic => write!(f, "arithmetic error"),
            Error::UnsupportedTransaction => write!(f, "unsupported transaction"),
            Error::FraudDetected => write!(f, "fraud detected"),
            Error::Reentrancy => write!(f, "reentrancy detected"),
            Error::CircuitOpen => write!(f, "circuit open"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized => symbol_short!("CHK_AUTH"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized | Error::AlreadyInState => symbol_short!("ALREADY"),
        Error::ContractPaused | Error::CircuitOpen => symbol_short!("RE_TRY_L"),
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        Error::ClaimNotFound => symbol_short!("CHK_ID"),
        _ => symbol_short!("CONTACT"),
    }
}
