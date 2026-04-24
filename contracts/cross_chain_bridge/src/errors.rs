use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    Unauthorized = 100,
    InsufficientConfirmations = 120,
    InsufficientOracleReports = 121,
    DuplicateOracleReport = 122,

    // --- Input Validation (200–299) ---
    InvalidSignature = 207,
    InvalidMessage = 280,
    InvalidNonce = 281,
    InvalidPayload = 282,
    InvalidAddress = 290,

    // --- Lifecycle & State (300–399) ---
    AlreadyInitialized = 301,
    ContractPaused = 302,
    Overflow = 580,

    // --- Entity Existence (400–499) ---
    MessageNotFound = 480,
    MessageExpired = 481,
    MessageAlreadyProcessed = 482,
    AtomicTxNotFound = 486,
    AtomicTxExpired = 487,
    AtomicTxAlreadyProcessed = 488,
    RecordRefNotFound = 489,
    RollbackNotFound = 490,
    RollbackAlreadyProcessed = 491,
    EventNotFound = 492,
    ValidatorNotFound = 483,
    ValidatorNotActive = 484,
    DuplicateConfirmation = 485,

    // --- Cryptography (600–699) ---
    ProofNotFound = 610,
    ProofAlreadyVerified = 611,

    // --- Cross-Chain (700–799) ---
    InvalidChain = 703,
    ChainNotSupported = 720,
    OracleNotFound = 721,
    OracleNotActive = 722,
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized | Error::InsufficientConfirmations | Error::InsufficientOracleReports => {
            symbol_short!("CHK_AUTH")
        }
        Error::AlreadyInitialized
        | Error::MessageAlreadyProcessed
        | Error::AtomicTxAlreadyProcessed
        | Error::RollbackAlreadyProcessed
        | Error::ProofAlreadyVerified
        | Error::DuplicateConfirmation
        | Error::DuplicateOracleReport => symbol_short!("ALREADY"),
        Error::ContractPaused => symbol_short!("RE_TRY_L"),
        Error::MessageNotFound
        | Error::AtomicTxNotFound
        | Error::ValidatorNotFound
        | Error::RecordRefNotFound
        | Error::RollbackNotFound
        | Error::EventNotFound => symbol_short!("CHK_ID"),
        _ => symbol_short!("CONTACT"),
    }
}
