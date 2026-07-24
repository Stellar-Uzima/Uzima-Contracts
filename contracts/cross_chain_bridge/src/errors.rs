use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control (100–199) ---
    Unauthorized = 100,
    UnauthorizedRelayer = 101,
    InsufficientConfirmations = 120,
    InsufficientOracleReports = 121,
    DuplicateOracleReport = 122,

    // --- Input Validation (200–299) ---
    InvalidSignature = 207,
    InvalidMessage = 280,
    InvalidNonce = 281,
    InvalidPayload = 282,
    InvalidAddress = 290,
    BatchTooLarge = 283,

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
    ReconciliationNotFound = 493,
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

    // --- Timeout / Operation (800–899) ---
    OperationNotFound = 800,
    OperationExpired = 801,
    OperationAlreadyCompleted = 802,
    MaxExtensionsReached = 803,
    RefundFailed = 804,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::UnauthorizedRelayer => write!(f, "unauthorized relayer"),
            Error::InsufficientConfirmations => write!(f, "insufficient confirmations"),
            Error::InsufficientOracleReports => write!(f, "insufficient oracle reports"),
            Error::DuplicateOracleReport => write!(f, "duplicate oracle report"),
            Error::InvalidSignature => write!(f, "invalid signature"),
            Error::InvalidMessage => write!(f, "invalid message"),
            Error::InvalidNonce => write!(f, "invalid nonce"),
            Error::InvalidPayload => write!(f, "invalid payload"),
            Error::InvalidAddress => write!(f, "invalid address"),
            Error::BatchTooLarge => write!(f, "batch too large"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::Overflow => write!(f, "overflow"),
            Error::MessageNotFound => write!(f, "message not found"),
            Error::MessageExpired => write!(f, "message expired"),
            Error::MessageAlreadyProcessed => write!(f, "message already processed"),
            Error::AtomicTxNotFound => write!(f, "atomic tx not found"),
            Error::AtomicTxExpired => write!(f, "atomic tx expired"),
            Error::AtomicTxAlreadyProcessed => write!(f, "atomic tx already processed"),
            Error::RecordRefNotFound => write!(f, "record ref not found"),
            Error::RollbackNotFound => write!(f, "rollback not found"),
            Error::RollbackAlreadyProcessed => write!(f, "rollback already processed"),
            Error::EventNotFound => write!(f, "event not found"),
            Error::ReconciliationNotFound => write!(f, "reconciliation not found"),
            Error::ValidatorNotFound => write!(f, "validator not found"),
            Error::ValidatorNotActive => write!(f, "validator not active"),
            Error::DuplicateConfirmation => write!(f, "duplicate confirmation"),
            Error::ProofNotFound => write!(f, "proof not found"),
            Error::ProofAlreadyVerified => write!(f, "proof already verified"),
            Error::InvalidChain => write!(f, "invalid chain"),
            Error::ChainNotSupported => write!(f, "chain not supported"),
            Error::OracleNotFound => write!(f, "oracle not found"),
            Error::OracleNotActive => write!(f, "oracle not active"),
            Error::OperationNotFound => write!(f, "operation not found"),
            Error::OperationExpired => write!(f, "operation expired"),
            Error::OperationAlreadyCompleted => write!(f, "operation already completed"),
            Error::MaxExtensionsReached => write!(f, "max extensions reached"),
            Error::RefundFailed => write!(f, "refund failed"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::Unauthorized
        | Error::InsufficientConfirmations
        | Error::InsufficientOracleReports => {
            symbol_short!("CHK_AUTH")
        },
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
        | Error::EventNotFound
        | Error::ReconciliationNotFound => symbol_short!("CHK_ID"),
        _ => symbol_short!("CONTACT"),
    }
}