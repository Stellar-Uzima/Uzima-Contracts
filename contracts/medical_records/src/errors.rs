use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- System & State Errors (1-9) ---
    ContractPaused = 1,
    ProposalAlreadyExecuted = 6,
    TimelockNotElasped = 7,
    NotEnoughApproval = 8,
    NotInitialized = 48,
    CryptoRegistryNotSet = 49,
    EncryptionRequired = 50,
    RateLimitExceeded = 51,

    // --- Access & Auth Errors (10-19) ---
    NotAuthorized = 2,
    CrossChainAccessDenied = 15,
    EmergencyAccessExpired = 24,
    EmergencyAccessNotFound = 25,
    NotAICoordinator = 28,

    // --- Identity & DID Errors (20-29) ---
    DIDNotFound = 18,
    DIDNotActive = 19,
    InvalidCredential = 20,
    CredentialExpired = 21,
    CredentialRevoked = 22,
    MissingRequiredCredential = 23,
    IdentityRegistryNotSet = 26,

    // --- Validation: Content Errors (30-39) ---
    InvalidCategory = 3,
    EmptyTreatment = 4,
    EmptyDiagnosis = 30,
    EmptyTag = 5,
    EmptyDataRef = 9,
    InvalidInput = 45,

    // --- Validation: Length & Format Errors (40-49) ---
    InvalidDataRefLength = 10,
    InvalidDataRefCharset = 11,
    InvalidDiagnosisLength = 31,
    InvalidTreatmentLength = 32,
    InvalidPurposeLength = 33,
    InvalidTagLength = 34,
    InvalidModelVersionLength = 38,
    InvalidExplanationLength = 39,
    InvalidTreatmentTypeLength = 42,

    // --- AI & Logic Errors (50-59) ---
    AIConfigNotSet = 27,
    InvalidAIScore = 29,
    InvalidScore = 35,
    InvalidDPEpsilon = 36,
    InvalidParticipantCount = 37,

    // --- General/Cross-Chain Errors (60+) ---
    RecordNotFound = 14,
    RecordAlreadySynced = 16,
    InvalidChain = 17,
    CrossChainNotEnabled = 12,
    CrossChainContractsNotSet = 13,
    InvalidAddress = 40,
    SameAddress = 41,
    BatchTooLarge = 43,
    InvalidBatch = 44,
    NumberOutOfBounds = 46,
}

/// AC: Recovery suggestions to help users fix issues
pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::ContractPaused => symbol_short!("RE_TRY_L"),
        Error::NotAuthorized => symbol_short!("CHK_AUTH"),
        Error::EmptyDiagnosis | Error::EmptyTreatment => symbol_short!("FILL_FLD"),
        Error::EmergencyAccessExpired => symbol_short!("NEW_EMER"),
        Error::InvalidCategory => symbol_short!("FIX_CAT"),
        _ => symbol_short!("CONTACT"),
    }
}
