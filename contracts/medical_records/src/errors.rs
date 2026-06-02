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

#[allow(dead_code)]
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
