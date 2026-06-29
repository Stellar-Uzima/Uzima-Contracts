use common_error::CommonError;
use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror(export = false)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Common Errors (0–99) ---
    Unauthorized = CommonError::Unauthorized as u32,
    InvalidInput = CommonError::InvalidInput as u32,
    NotInitialized = CommonError::NotInitialized as u32,
    ContractPaused = CommonError::ContractPaused as u32,
    DeadlineExceeded = CommonError::DeadlineExceeded as u32,
    RateLimitExceeded = CommonError::RateLimitExceeded as u32,
    InsufficientFunds = CommonError::InsufficientFunds as u32,

    // --- Access Control & Authorization (1000–1099) ---
    NotAICoordinator = 1150,
    EmergencyAccessExpired = 1160,

    // --- Input Validation (1100–1199) ---
    InvalidPagination = 1202,
    InputTooLong = 1201,
    BatchTooLarge = 1208,
    InvalidSignature = 1207,
    InvalidDataRefLength = 1250,
    InvalidDataRefCharset = 1251,
    InvalidDiagnosisLength = 1252,
    InvalidTreatmentLength = 1253,
    InvalidPurposeLength = 1254,
    InvalidTagLength = 1255,
    InvalidModelVersionLength = 1256,
    InvalidExplanationLength = 1257,
    InvalidTreatmentTypeLength = 1258,
    InvalidAddress = 1290,
    SameAddress = 1291,
    InvalidBatch = 1292,
    NumberOutOfBounds = 1293,
    InvalidCategory = 1280,
    EmptyTreatment = 1281,
    EmptyDiagnosis = 1282,
    EmptyTag = 1283,
    EmptyDataRef = 1284,

    // --- Lifecycle & State (1200–1299) ---
    ProposalAlreadyExecuted = 1320,
    TimelockNotElapsed = 1321,
    NotEnoughApproval = 1322,
    CryptoRegistryNotSet = 1340,
    EncryptionRequired = 1341,
    IdentityRegistryNotSet = 1342,

    // --- Entity Existence (1300–1399) ---
    RecordNotFound = 1403,
    EmergencyAccessNotFound = 1460,
    DIDNotFound = 1470,
    DIDNotActive = 1471,
    RecordAlreadySynced = 1480,

    // --- Financial & Resource (1400–1499) ---
    StorageFull = 1502,

    // --- Cryptography & ZK (1500–1599) ---
    InvalidCredential = 1640,
    MissingRequiredCredential = 1641,
    CredentialExpired = 1605,
    CredentialRevoked = 1606,

    // --- Cross-Chain & Integration (1600–1699) ---
    CrossChainAccessDenied = 1700,
    CrossChainTimeout = 1702,
    InvalidChain = 1703,
    CrossChainNotEnabled = 1710,
    CrossChainContractsNotSet = 1711,

    // --- Domain-Specific: AI/Medical (1700–1799) ---
    AIConfigNotSet = 1830,
    InvalidAIScore = 1831,
    InvalidScore = 1832,
    InvalidDPEpsilon = 1833,
    InvalidParticipantCount = 1834,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::DeadlineExceeded => write!(f, "deadline exceeded"),
            Error::RateLimitExceeded => write!(f, "rate limit exceeded"),
            Error::InsufficientFunds => write!(f, "insufficient funds"),
            Error::NotAICoordinator => write!(f, "not a i coordinator"),
            Error::EmergencyAccessExpired => write!(f, "emergency access expired"),
            Error::InvalidPagination => write!(f, "invalid pagination"),
            Error::InputTooLong => write!(f, "input too long"),
            Error::BatchTooLarge => write!(f, "batch too large"),
            Error::InvalidSignature => write!(f, "invalid signature"),
            Error::InvalidDataRefLength => write!(f, "invalid data ref length"),
            Error::InvalidDataRefCharset => write!(f, "invalid data ref charset"),
            Error::InvalidDiagnosisLength => write!(f, "invalid diagnosis length"),
            Error::InvalidTreatmentLength => write!(f, "invalid treatment length"),
            Error::InvalidPurposeLength => write!(f, "invalid purpose length"),
            Error::InvalidTagLength => write!(f, "invalid tag length"),
            Error::InvalidModelVersionLength => write!(f, "invalid model version length"),
            Error::InvalidExplanationLength => write!(f, "invalid explanation length"),
            Error::InvalidTreatmentTypeLength => write!(f, "invalid treatment type length"),
            Error::InvalidAddress => write!(f, "invalid address"),
            Error::SameAddress => write!(f, "same address"),
            Error::InvalidBatch => write!(f, "invalid batch"),
            Error::NumberOutOfBounds => write!(f, "number out of bounds"),
            Error::InvalidCategory => write!(f, "invalid category"),
            Error::EmptyTreatment => write!(f, "empty treatment"),
            Error::EmptyDiagnosis => write!(f, "empty diagnosis"),
            Error::EmptyTag => write!(f, "empty tag"),
            Error::EmptyDataRef => write!(f, "empty data ref"),
            Error::ProposalAlreadyExecuted => write!(f, "proposal already executed"),
            Error::TimelockNotElapsed => write!(f, "timelock not elapsed"),
            Error::NotEnoughApproval => write!(f, "not enough approval"),
            Error::CryptoRegistryNotSet => write!(f, "crypto registry not set"),
            Error::EncryptionRequired => write!(f, "encryption required"),
            Error::IdentityRegistryNotSet => write!(f, "identity registry not set"),
            Error::RecordNotFound => write!(f, "record not found"),
            Error::EmergencyAccessNotFound => write!(f, "emergency access not found"),
            Error::DIDNotFound => write!(f, "d i d not found"),
            Error::DIDNotActive => write!(f, "d i d not active"),
            Error::RecordAlreadySynced => write!(f, "record already synced"),
            Error::StorageFull => write!(f, "storage full"),
            Error::InvalidCredential => write!(f, "invalid credential"),
            Error::MissingRequiredCredential => write!(f, "missing required credential"),
            Error::CredentialExpired => write!(f, "credential expired"),
            Error::CredentialRevoked => write!(f, "credential revoked"),
            Error::CrossChainAccessDenied => write!(f, "cross chain access denied"),
            Error::CrossChainTimeout => write!(f, "cross chain timeout"),
            Error::InvalidChain => write!(f, "invalid chain"),
            Error::CrossChainNotEnabled => write!(f, "cross chain not enabled"),
            Error::CrossChainContractsNotSet => write!(f, "cross chain contracts not set"),
            Error::AIConfigNotSet => write!(f, "a i config not set"),
            Error::InvalidAIScore => write!(f, "invalid a i score"),
            Error::InvalidScore => write!(f, "invalid score"),
            Error::InvalidDPEpsilon => write!(f, "invalid d p epsilon"),
            Error::InvalidParticipantCount => write!(f, "invalid participant count"),
        }
    }
}


pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::ContractPaused | Error::RateLimitExceeded => symbol_short!("RE_TRY_L"),
        Error::InvalidPagination => symbol_short!("CHK_DATA"),
        Error::Unauthorized | Error::NotAICoordinator => symbol_short!("CHK_AUTH"),
        Error::EmptyDiagnosis | Error::EmptyTreatment => symbol_short!("FILL_FLD"),
        Error::EmergencyAccessExpired => symbol_short!("NEW_EMER"),
        Error::InvalidCategory => symbol_short!("FIX_CAT"),
        Error::InvalidBatch => symbol_short!("CHK_DATA"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::RecordNotFound | Error::DIDNotFound => symbol_short!("CHK_ID"),
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        Error::StorageFull => symbol_short!("CLN_OLD"),
        _ => symbol_short!("CONTACT"),
    }
}
