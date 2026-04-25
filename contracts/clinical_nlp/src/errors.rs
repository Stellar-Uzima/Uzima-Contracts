use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- System & State Errors (1-9) ---
    ContractPaused = 1,
    NotInitialized = 2,
    InvalidConfiguration = 3,
    ProcessingTimeout = 4,
    RateLimitExceeded = 5,

    // --- Input Validation Errors (10-19) ---
    EmptyClinicalNote = 10,
    InvalidInputLength = 11,
    InvalidLanguageCode = 12,
    InvalidEncoding = 13,
    InputTooLarge = 14,

    // --- NLP Processing Errors (20-29) ---
    NLPEngineNotInitialized = 20,
    EntityExtractionFailed = 21,
    ConceptExtractionFailed = 22,
    SentimentAnalysisFailed = 23,
    CodingSuggestionFailed = 24,
    TokenizationFailed = 25,
    LanguageDetectionFailed = 26,

    // --- Medical Terms Errors (30-39) ---
    MedicalTermNotFound = 30,
    InvalidMedicalTerm = 31,
    TermDatabaseNotLoaded = 32,

    // --- Coding Errors (40-49) ---
    ICD10CodeNotFound = 40,
    CPTCodeNotFound = 41,
    InvalidCodeFormat = 42,
    CodeMappingFailed = 43,

    // --- Integration Errors (50-59) ---
    MedicalRecordsContractNotSet = 50,
    RecordAccessDenied = 51,
    RecordNotFound = 52,
    IntegrationFailed = 53,

    // --- Authorization Errors (60-69) ---
    NotAuthorized = 60,
    InsufficientPermissions = 61,
    HIPAAComplianceViolation = 62,

    // --- Performance Errors (70-79) ---
    ProcessingTimeExceeded = 70,
    MemoryLimitExceeded = 71,
    BatchSizeTooLarge = 72,
}

/// Recovery suggestions to help users fix issues
pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::EmptyClinicalNote => symbol_short!("ADD_TEXT"),
        Error::InvalidInputLength => symbol_short!("CHK_LEN"),
        Error::InvalidLanguageCode => symbol_short!("FIX_LANG"),
        Error::ProcessingTimeout => symbol_short!("RETRY"),
        Error::NotAuthorized => symbol_short!("CHK_AUTH"),
        Error::MedicalRecordsContractNotSet => symbol_short!("SET_CNTR"),
        Error::InputTooLarge => symbol_short!("REDUCE"),
        Error::HIPAAComplianceViolation => symbol_short!("CHK_PHI"),
        _ => symbol_short!("CONTACT"),
    }
}
