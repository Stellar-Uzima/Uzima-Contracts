use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Map, String, Vec};

// ==================== Event Schema Definitions ====================

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    // NLP Processing Events
    ClinicalNoteProcessed,
    EntityExtractionCompleted,
    ConceptExtractionCompleted,
    SentimentAnalysisCompleted,
    CodingSuggestionGenerated,

    // Configuration Events
    NLPConfigUpdated,
    MedicalTermsLoaded,
    CodingDatabaseUpdated,

    // Integration Events
    MedicalRecordLinked,
    BatchProcessingStarted,
    BatchProcessingCompleted,

    // System Events
    ContractInitialized,
    ContractPaused,
    ContractUnpaused,

    // Performance Events
    ProcessingTimeRecorded,
    AccuracyMetricsUpdated,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    NLPProcessing,
    EntityExtraction,
    ConceptExtraction,
    SentimentAnalysis,
    CodingSuggestions,
    Configuration,
    Integration,
    System,
    Performance,
}

#[derive(Clone)]
#[contracttype]
pub struct EventMetadata {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub session_id: Option<String>,
    pub processing_time_ms: Option<u64>,
    pub block_height: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct NLPProcessingEventData {
    pub note_id: BytesN<32>,
    pub patient_id: Option<Address>,
    pub record_id: Option<BytesN<32>>,
    pub language: String,
    pub entities_count: u32,
    pub concepts_count: u32,
    pub processing_time_ms: u64,
    pub accuracy_score_bps: u32, // basis points (100 = 1%)
}

#[derive(Clone)]
#[contracttype]
pub struct EntityExtractionEventData {
    pub note_id: BytesN<32>,
    pub entity_type: String,
    pub entity_value: String,
    pub confidence_bps: u32,
    pub start_position: u32,
    pub end_position: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct SentimentAnalysisEventData {
    pub note_id: BytesN<32>,
    pub sentiment_score: i32, // -100 to 100
    pub sentiment_label: String,
    pub confidence_bps: u32,
    pub emotional_indicators: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct CodingSuggestionEventData {
    pub note_id: BytesN<32>,
    pub code_type: String, // "ICD-10" or "CPT"
    pub suggested_code: String,
    pub description: String,
    pub confidence_bps: u32,
    pub supporting_evidence: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct BatchProcessingEventData {
    pub batch_id: BytesN<32>,
    pub total_notes: u32,
    pub processed_notes: u32,
    pub failed_notes: u32,
    pub total_processing_time_ms: u64,
    pub average_accuracy_bps: u32,
}

// ==================== Event Emission Functions ====================

pub fn emit_nlp_processing_event(env: &Env, metadata: EventMetadata, data: NLPProcessingEventData) {
    let topics = (symbol_short!("NLP_PROC"), metadata.event_type);
    env.events().publish(topics, (metadata, data));
}

pub fn emit_entity_extraction_event(
    env: &Env,
    metadata: EventMetadata,
    data: EntityExtractionEventData,
) {
    let topics = (symbol_short!("ENTITY"), metadata.event_type);
    env.events().publish(topics, (metadata, data));
}

pub fn emit_sentiment_analysis_event(
    env: &Env,
    metadata: EventMetadata,
    data: SentimentAnalysisEventData,
) {
    let topics = (symbol_short!("SENTIM"), metadata.event_type);
    env.events().publish(topics, (metadata, data));
}

pub fn emit_coding_suggestion_event(
    env: &Env,
    metadata: EventMetadata,
    data: CodingSuggestionEventData,
) {
    let topics = (symbol_short!("CODING"), metadata.event_type);
    env.events().publish(topics, (metadata, data));
}

pub fn emit_batch_processing_event(
    env: &Env,
    metadata: EventMetadata,
    data: BatchProcessingEventData,
) {
    let topics = (symbol_short!("BATCH"), metadata.event_type);
    env.events().publish(topics, (metadata, data));
}
