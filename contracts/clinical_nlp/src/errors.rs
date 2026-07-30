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

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Unauthorized => write!(f, "unauthorized"),
            Error::InsufficientPermissions => write!(f, "insufficient permissions"),
            Error::HIPAAComplianceViolation => write!(f, "h i p a a compliance violation"),
            Error::RecordAccessDenied => write!(f, "record access denied"),
            Error::InputTooLong => write!(f, "input too long"),
            Error::BatchTooLarge => write!(f, "batch too large"),
            Error::EmptyClinicalNote => write!(f, "empty clinical note"),
            Error::InvalidLanguageCode => write!(f, "invalid language code"),
            Error::InvalidEncoding => write!(f, "invalid encoding"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::ContractPaused => write!(f, "contract paused"),
            Error::RateLimitExceeded => write!(f, "rate limit exceeded"),
            Error::Timeout => write!(f, "timeout"),
            Error::InvalidConfiguration => write!(f, "invalid configuration"),
            Error::RecordNotFound => write!(f, "record not found"),
            Error::IntegrationFailed => write!(f, "integration failed"),
            Error::ExternalContractNotSet => write!(f, "external contract not set"),
            Error::NLPEngineNotInitialized => write!(f, "n l p engine not initialized"),
            Error::EntityExtractionFailed => write!(f, "entity extraction failed"),
            Error::ConceptExtractionFailed => write!(f, "concept extraction failed"),
            Error::SentimentAnalysisFailed => write!(f, "sentiment analysis failed"),
            Error::CodingSuggestionFailed => write!(f, "coding suggestion failed"),
            Error::TokenizationFailed => write!(f, "tokenization failed"),
            Error::LanguageDetectionFailed => write!(f, "language detection failed"),
            Error::MedicalTermNotFound => write!(f, "medical term not found"),
            Error::InvalidMedicalTerm => write!(f, "invalid medical term"),
            Error::TermDatabaseNotLoaded => write!(f, "term database not loaded"),
            Error::ICD10CodeNotFound => write!(f, "i c d10 code not found"),
            Error::CPTCodeNotFound => write!(f, "c p t code not found"),
            Error::InvalidCodeFormat => write!(f, "invalid code format"),
            Error::CodeMappingFailed => write!(f, "code mapping failed"),
        }
    }
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
