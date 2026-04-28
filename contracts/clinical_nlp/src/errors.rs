use soroban_sdk::{contracterror, symbol_short, Symbol};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Authorization (100–199) ---
    Unauthorized = 100,
    InsufficientPermissions = 101,
    HIPAAComplianceViolation = 104,
    RecordAccessDenied = 112,

    // --- Input Validation (200–299) ---
    InputTooLong = 201,
    BatchTooLarge = 208,
    EmptyClinicalNote = 209,
    InvalidLanguageCode = 212,
    InvalidEncoding = 213,

    // --- Lifecycle & State (300–399) ---
    NotInitialized = 300,
    AlreadyInitialized = 301,
    ContractPaused = 302,
    RateLimitExceeded = 307,
    Timeout = 308,
    InvalidConfiguration = 310,

    // --- Entity Existence (400–499) ---
    RecordNotFound = 403,

    // --- Integration (700–799) ---
    IntegrationFailed = 704,
    ExternalContractNotSet = 705,

    // --- Domain-Specific: NLP (800–899) ---
    NLPEngineNotInitialized = 800,
    EntityExtractionFailed = 801,
    ConceptExtractionFailed = 802,
    SentimentAnalysisFailed = 803,
    CodingSuggestionFailed = 804,
    TokenizationFailed = 805,
    LanguageDetectionFailed = 806,
    MedicalTermNotFound = 807,
    InvalidMedicalTerm = 808,
    TermDatabaseNotLoaded = 809,
    ICD10CodeNotFound = 810,
    CPTCodeNotFound = 811,
    InvalidCodeFormat = 812,
    CodeMappingFailed = 813,
}

pub fn get_suggestion(error: Error) -> Symbol {
    match error {
        Error::EmptyClinicalNote => symbol_short!("ADD_TEXT"),
        Error::InputTooLong => symbol_short!("CHK_LEN"),
        Error::InvalidLanguageCode => symbol_short!("FIX_LANG"),
        Error::Timeout => symbol_short!("RE_TRY_L"),
        Error::Unauthorized | Error::InsufficientPermissions | Error::RecordAccessDenied => {
            symbol_short!("CHK_AUTH")
        }
        Error::ExternalContractNotSet => symbol_short!("SET_CNTR"),
        Error::BatchTooLarge => symbol_short!("REDUCE"),
        Error::HIPAAComplianceViolation => symbol_short!("CHK_PHI"),
        Error::NotInitialized => symbol_short!("INIT_CTR"),
        Error::AlreadyInitialized => symbol_short!("ALREADY"),
        Error::ContractPaused | Error::RateLimitExceeded => symbol_short!("RE_TRY_L"),
        _ => symbol_short!("CONTACT"),
    }
}
