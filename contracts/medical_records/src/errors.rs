use common_error::CommonError;
use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror(export = false)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control & Authorization (100–199) ---
    Unauthorized = 100,
    NotAuthorized = 101,
    NotAICoordinator = 150,
    EmergencyAccessExpired = 160,
    RecordRetentionExpired = 170,

    // --- Input Validation (200–299) ---
    InvalidInput = 200,
    InputTooLong = 201,
    InvalidPagination = 202,
    InvalidSignature = 207,
    BatchTooLarge = 208,
    InvalidDataRefLength = 250,
    InvalidDataRefCharset = 251,
    InvalidDiagnosisLength = 252,
    InvalidTreatmentLength = 253,
    InvalidPurposeLength = 254,
    InvalidTagLength = 255,
    InvalidModelVersionLength = 256,
    InvalidExplanationLength = 257,
    InvalidTreatmentTypeLength = 258,
    InvalidCategory = 280,
    EmptyTreatment = 281,
    EmptyDiagnosis = 282,
    EmptyTag = 283,
    EmptyDataRef = 284,
    InvalidAddress = 290,
    SameAddress = 291,
    InvalidBatch = 292,
    NumberOutOfBounds = 293,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    DeadlineExceeded = 306,
    RateLimitExceeded = 307,
    ProposalAlreadyExecuted = 320,
    TimelockNotElapsed = 321,
    NotEnoughApproval = 322,
    CryptoRegistryNotSet = 340,
    EncryptionRequired = 341,
    IdentityRegistryNotSet = 342,
    InvalidVersion = 350,
    VersionNotFound = 351,
    SchemaVersionAlreadyExists = 352,
    SchemaVersionNotFound = 353,

    // --- Entity Existence (400–499) ---
    RecordNotFound = 403,
    NotFound = 404,
    EmergencyAccessNotFound = 460,
    DIDNotFound = 470,
    DIDNotActive = 471,
    RecordAlreadySynced = 480,

    // --- Financial & Resource (500–599) ---
    InsufficientFunds = 500,
    StorageFull = 502,

    // --- Cryptography & ZK (600–699) ---
    CredentialExpired = 605,
    CredentialRevoked = 606,
    InvalidCredential = 640,
    MissingRequiredCredential = 641,

    // --- Cross-Chain & Integration (700–799) ---
    CrossChainAccessDenied = 700,
    CrossChainTimeout = 702,
    InvalidChain = 703,
    CrossChainNotEnabled = 710,
    CrossChainContractsNotSet = 711,

    // --- Domain-Specific: AI/Medical (800–899) ---
    AIConfigNotSet = 830,
    InvalidAIScore = 831,
    InvalidScore = 832,
    InvalidDPEpsilon = 833,
    InvalidParticipantCount = 834,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized | Error::NotAuthorized => write!(f, "unauthorized"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
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
            Error::RecordNotFound | Error::NotFound => write!(f, "record not found"),
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
            Error::InvalidVersion => write!(f, "invalid version"),
            Error::VersionNotFound => write!(f, "version not found"),
            Error::SchemaVersionAlreadyExists => write!(f, "schema version already exists"),
            Error::SchemaVersionNotFound => write!(f, "schema version not found"),
            Error::InvalidParticipantCount => write!(f, "invalid participant count"),
        }
    }
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::ContractPaused | Error::RateLimitExceeded => symbol_short!("RE_TRY_L"),
        Error::InvalidPagination => symbol_short!("CHK_DATA"),
        Error::Unauthorized | Error::NotAuthorized | Error::NotAICoordinator => {
            symbol_short!("CHK_AUTH")
        }
        Error::EmptyDiagnosis | Error::EmptyTreatment => symbol_short!("FILL_FLD"),
        Error::EmergencyAccessExpired => symbol_short!("NEW_EMER"),
        Error::RecordRetentionExpired => symbol_short!("ADM_OVR"),
        Error::InvalidCategory => symbol_short!("FIX_CAT"),
        Error::InvalidBatch | Error::AlreadyInitialized => symbol_short!("CHK_DATA"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::RecordNotFound
        | Error::NotFound
        | Error::DIDNotFound
        | Error::SchemaVersionNotFound => symbol_short!("CHK_ID"),
        Error::SchemaVersionAlreadyExists => symbol_short!("CHK_SCHM"),
        Error::InsufficientFunds => symbol_short!("ADD_FUND"),
        Error::StorageFull => symbol_short!("CLN_OLD"),
        _ => symbol_short!("CONTACT"),
    }
}