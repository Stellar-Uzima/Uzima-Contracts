use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror(export = false)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Access Control & Authorization (100–199) ---
    Unauthorized = 100,
    NotAICoordinator = 150,
    EmergencyAccessExpired = 160,

    // --- Input Validation (200–299) ---
    InvalidInput = 200,
    InputTooLong = 201,
    BatchTooLarge = 208,
    InvalidSignature = 207,
    InvalidDataRefLength = 250,
    InvalidDataRefCharset = 251,
    InvalidDiagnosisLength = 252,
    InvalidTreatmentLength = 253,
    InvalidPurposeLength = 254,
    InvalidTagLength = 255,
    InvalidModelVersionLength = 256,
    InvalidExplanationLength = 257,
    InvalidTreatmentTypeLength = 258,
    InvalidAddress = 290,
    SameAddress = 291,
    InvalidBatch = 292,
    NumberOutOfBounds = 293,
    InvalidCategory = 280,
    EmptyTreatment = 281,
    EmptyDiagnosis = 282,
    EmptyTag = 283,
    EmptyDataRef = 284,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    ContractPaused = 302,
    DeadlineExceeded = 306,
    RateLimitExceeded = 307,
    ProposalAlreadyExecuted = 320,
    TimelockNotElapsed = 321,
    NotEnoughApproval = 322,
    CryptoRegistryNotSet = 340,
    EncryptionRequired = 341,
    IdentityRegistryNotSet = 342,

    // --- Entity Existence (400–499) ---
    RecordNotFound = 403,
    EmergencyAccessNotFound = 460,
    DIDNotFound = 470,
    DIDNotActive = 471,
    RecordAlreadySynced = 480,

    // --- Financial & Resource (500–599) ---
    InsufficientFunds = 500,
    StorageFull = 502,

    // --- Cryptography & ZK (600–699) ---
    InvalidCredential = 640,
    MissingRequiredCredential = 641,
    CredentialExpired = 605,
    CredentialRevoked = 606,

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

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::ContractPaused | Error::RateLimitExceeded => symbol_short!("RE_TRY_L"),
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
