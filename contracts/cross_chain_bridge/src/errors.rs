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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Error::Unauthorized => "Unauthorized",
            Error::UnauthorizedRelayer => "Unauthorized Relayer",
            Error::InsufficientConfirmations => "Insufficient Confirmations",
            Error::InsufficientOracleReports => "Insufficient Oracle Reports",
            Error::DuplicateOracleReport => "Duplicate Oracle Report",
            Error::InvalidSignature => "Invalid Signature",
            Error::InvalidMessage => "Invalid Message",
            Error::InvalidNonce => "Invalid Nonce",
            Error::InvalidPayload => "Invalid Payload",
            Error::InvalidAddress => "Invalid Address",
            Error::BatchTooLarge => "Batch Too Large",
            Error::AlreadyInitialized => "Already Initialized",
            Error::ContractPaused => "Contract Paused",
            Error::Overflow => "Overflow",
            Error::MessageNotFound => "Message Not Found",
            Error::MessageExpired => "Message Expired",
            Error::MessageAlreadyProcessed => "Message Already Processed",
            Error::AtomicTxNotFound => "Atomic Tx Not Found",
            Error::AtomicTxExpired => "Atomic Tx Expired",
            Error::AtomicTxAlreadyProcessed => "Atomic Tx Already Processed",
            Error::RecordRefNotFound => "Record Ref Not Found",
            Error::RollbackNotFound => "Rollback Not Found",
            Error::RollbackAlreadyProcessed => "Rollback Already Processed",
            Error::EventNotFound => "Event Not Found",
            Error::ValidatorNotFound => "Validator Not Found",
            Error::ValidatorNotActive => "Validator Not Active",
            Error::DuplicateConfirmation => "Duplicate Confirmation",
            Error::ProofNotFound => "Proof Not Found",
            Error::ProofAlreadyVerified => "Proof Already Verified",
            Error::InvalidChain => "Invalid Chain",
            Error::ChainNotSupported => "Chain Not Supported",
            Error::OracleNotFound => "Oracle Not Found",
            Error::OracleNotActive => "Oracle Not Active",
            Error::OperationNotFound => "Operation Not Found",
            Error::OperationExpired => "Operation Expired",
            Error::OperationAlreadyCompleted => "Operation Already Completed",
            Error::MaxExtensionsReached => "Max Extensions Reached",
            Error::RefundFailed => "Refund Failed",
        };
        f.write_str(message)
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
        | Error::EventNotFound => symbol_short!("CHK_ID"),
        _ => symbol_short!("CONTACT"),
    }
}
