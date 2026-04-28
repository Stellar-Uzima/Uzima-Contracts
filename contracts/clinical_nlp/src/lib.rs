#![no_std]

mod errors;
mod events;
mod icd_cpt_codes;
mod medical_terms;
mod nlp_engine;
mod sentiment;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

pub use errors::Error;
pub use events::{EventMetadata, EventType, NLPProcessingEventData, OperationCategory};
pub use nlp_engine::{ClinicalConcept, ExtractedEntity, Language, NLPConfig, NLPEngine, NLPResult};
pub use sentiment::{SentimentLabel, SentimentResult};

#[contract]
pub struct ClinicalNLP;

#[derive(Clone)]
pub struct ProcessingStats {
    pub total_notes_processed: u64,
    pub total_processing_time_ms: u64,
    pub average_accuracy_bps: u32,
    pub entities_extracted: u64,
    pub concepts_extracted: u64,
    pub coding_suggestions_generated: u64,
    pub phi_detections: u64,
    pub last_updated: u64,
}

#[derive(Clone)]
pub struct BatchProcessingRequest {
    pub batch_id: BytesN<32>,
    pub notes: Vec<String>,
    pub patient_ids: Vec<Address>,
    pub record_ids: Vec<BytesN<32>>,
    pub language: u32, // 0=English, 1=Spanish, 2=French, 3=German, 4=Portuguese
}

#[derive(Clone)]
pub struct BatchProcessingResult {
    pub batch_id: BytesN<32>,
    pub results: Vec<NLPResult>,
    pub total_processing_time_ms: u64,
    pub success_count: u32,
    pub failure_count: u32,
    pub average_accuracy_bps: u32,
}

#[contractimpl]
impl ClinicalNLP {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&"initialized") {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&"admin", &admin);
        env.storage().instance().set(&"initialized", &true);

        let config = NLPConfig {
            medical_records_contract: None,
            enable_entity_extraction: true,
            enable_concept_extraction: true,
            enable_sentiment_analysis: true,
            enable_coding_suggestions: true,
            max_processing_time_ms: 2000,
            supported_languages: Vec::from_array(&env, [0, 1, 2, 3, 4]),
            phi_detection_enabled: true,
            accuracy_threshold_bps: 9000,
        };
        env.storage().instance().set(&"config", &config);

        let stats = ProcessingStats {
            total_notes_processed: 0,
            total_processing_time_ms: 0,
            average_accuracy_bps: 0,
            entities_extracted: 0,
            concepts_extracted: 0,
            coding_suggestions_generated: 0,
            phi_detections: 0,
            last_updated: env.ledger().timestamp(),
        };
        env.storage().instance().set(&"stats", &stats);

        Ok(())
    }

    pub fn process_clinical_note(
        env: Env,
        note_text: String,
        note_id: BytesN<32>,
        patient_id: Address,
        record_id: BytesN<32>,
        language: u32,
    ) -> Result<NLPResult, Error> {
        if !env.storage().instance().has(&"initialized") {
            return Err(Error::NotInitialized);
        }

        let config: NLPConfig = env.storage().instance().get(&"config").unwrap();
        let mut engine = NLPEngine::new(env.clone(), config);
        engine.initialize_defaults(&env);

        let start_time = env.ledger().timestamp();
        let result = engine.process_clinical_note(&env, &note_text, &note_id, language)?;
        let processing_time = env.ledger().timestamp() - start_time;

        let mut stats: ProcessingStats = env.storage().instance().get(&"stats").unwrap();
        stats.total_notes_processed += 1;
        stats.total_processing_time_ms += processing_time;
        stats.entities_extracted += result.entities.len() as u64;
        stats.concepts_extracted += result.concepts.len() as u64;
        stats.coding_suggestions_generated += result.coding_suggestions.len() as u64;
        if result.phi_detected {
            stats.phi_detections += 1;
        }
        stats.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&"stats", &stats);

        let metadata = EventMetadata {
            event_type: EventType::ClinicalNoteProcessed,
            category: OperationCategory::NLPProcessing,
            timestamp: env.ledger().timestamp(),
            user_id: patient_id.clone(),
            session_id: None,
            processing_time_ms: Some(processing_time),
            block_height: env.ledger().sequence(),
        };

        let event_data = NLPProcessingEventData {
            note_id: note_id.clone(),
            patient_id: Some(patient_id),
            record_id: Some(record_id),
            language: String::from_str(&env, "en"),
            entities_count: result.entities.len() as u32,
            concepts_count: result.concepts.len() as u32,
            processing_time_ms: processing_time,
            accuracy_score_bps: result.accuracy_score_bps,
        };

        events::emit_nlp_processing_event(&env, metadata, event_data);

        Ok(result)
    }

    pub fn extract_entities(env: Env, text: String) -> Result<Vec<ExtractedEntity>, Error> {
        if !env.storage().instance().has(&"initialized") {
            return Err(Error::NotInitialized);
        }

        let config: NLPConfig = env.storage().instance().get(&"config").unwrap();
        let mut engine = NLPEngine::new(env.clone(), config);
        engine.initialize_defaults(&env);

        engine.extract_entities(&text)
    }

    pub fn analyze_sentiment(env: Env, text: String) -> Result<SentimentResult, Error> {
        if !env.storage().instance().has(&"initialized") {
            return Err(Error::NotInitialized);
        }

        let config: NLPConfig = env.storage().instance().get(&"config").unwrap();
        let mut engine = NLPEngine::new(env.clone(), config);
        engine.initialize_defaults(&env);

        engine.analyze_sentiment(&text)
    }

    pub fn generate_coding_suggestions(
        env: Env,
        text: String,
        max_suggestions: u32,
    ) -> Result<Vec<icd_cpt_codes::CodingSuggestion>, Error> {
        if !env.storage().instance().has(&"initialized") {
            return Err(Error::NotInitialized);
        }

        let config: NLPConfig = env.storage().instance().get(&"config").unwrap();
        let mut engine = NLPEngine::new(env.clone(), config);
        engine.initialize_defaults(&env);

        engine.generate_coding_suggestions(&text, max_suggestions)
    }

    pub fn get_processing_stats(env: Env) -> Result<ProcessingStats, Error> {
        if !env.storage().instance().has(&"initialized") {
            return Err(Error::NotInitialized);
        }

        Ok(env.storage().instance().get(&"stats").unwrap())
    }

    pub fn update_config(env: Env, admin: Address, config: NLPConfig) -> Result<(), Error> {
        admin.require_auth();

        let stored_admin: Address = env.storage().instance().get(&"admin").unwrap();
        if admin != stored_admin {
            return Err(Error::Unauthorized);
        }

        env.storage().instance().set(&"config", &config);
        Ok(())
    }

    pub fn process_batch(
        env: Env,
        request: BatchProcessingRequest,
    ) -> Result<BatchProcessingResult, Error> {
        if !env.storage().instance().has(&"initialized") {
            return Err(Error::NotInitialized);
        }

        let config: NLPConfig = env.storage().instance().get(&"config").unwrap();
        let mut engine = NLPEngine::new(env.clone(), config);
        engine.initialize_defaults(&env);

        let start_time = env.ledger().timestamp();
        let mut results = Vec::new(&env);
        let mut success_count: u32 = 0;
        let mut failure_count: u32 = 0;
        let mut total_accuracy: u64 = 0;

        let batch_size = request.notes.len();
        if batch_size > 100 {
            return Err(Error::BatchTooLarge);
        }

        for i in 0..batch_size {
            let note_text = request.notes.get(i).unwrap();
            let patient_id = request.patient_ids.get(i).unwrap();
            let record_id = request.record_ids.get(i).unwrap();

            let note_id = BytesN::from_array(&env, &[i as u8; 32]);

            match engine.process_clinical_note(&env, &note_text, &note_id, request.language) {
                Ok(result) => {
                    total_accuracy += result.accuracy_score_bps as u64;
                    results.push_back(result);
                    success_count += 1;

                    let metadata = EventMetadata {
                        event_type: EventType::ClinicalNoteProcessed,
                        category: OperationCategory::NLPProcessing,
                        timestamp: env.ledger().timestamp(),
                        user_id: patient_id.clone(),
                        session_id: None,
                        processing_time_ms: Some(0),
                        block_height: env.ledger().sequence(),
                    };

                    let event_data = NLPProcessingEventData {
                        note_id: note_id.clone(),
                        patient_id: Some(patient_id.clone()),
                        record_id: Some(record_id.clone()),
                        language: String::from_str(&env, "en"),
                        entities_count: results.get(results.len() - 1).unwrap().entities.len()
                            as u32,
                        concepts_count: results.get(results.len() - 1).unwrap().concepts.len()
                            as u32,
                        processing_time_ms: 0,
                        accuracy_score_bps: results
                            .get(results.len() - 1)
                            .unwrap()
                            .accuracy_score_bps,
                    };

                    events::emit_nlp_processing_event(&env, metadata, event_data);
                }
                Err(_) => {
                    failure_count += 1;
                }
            }
        }

        let end_time = env.ledger().timestamp();
        let total_processing_time_ms = (end_time - start_time) * 1000;

        let average_accuracy_bps = if success_count > 0 {
            (total_accuracy / success_count as u64) as u32
        } else {
            0
        };

        let batch_metadata = EventMetadata {
            event_type: EventType::BatchProcessingCompleted,
            category: OperationCategory::NLPProcessing,
            timestamp: env.ledger().timestamp(),
            user_id: Address::from_str(
                &env,
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            ),
            session_id: None,
            processing_time_ms: Some(total_processing_time_ms),
            block_height: env.ledger().sequence(),
        };

        let batch_event_data = events::BatchProcessingEventData {
            batch_id: request.batch_id.clone(),
            total_notes: batch_size as u32,
            processed_notes: success_count,
            failed_notes: failure_count,
            total_processing_time_ms,
            average_accuracy_bps,
        };

        events::emit_batch_processing_event(&env, batch_metadata, batch_event_data);

        Ok(BatchProcessingResult {
            batch_id: request.batch_id,
            results,
            total_processing_time_ms,
            success_count,
            failure_count,
            average_accuracy_bps,
        })
    }

    pub fn version(_env: Env) -> u32 {
        1
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&"initialized")
    }
}
