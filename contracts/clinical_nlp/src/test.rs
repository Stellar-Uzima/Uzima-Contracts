#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    let result = client.initialize(&admin);
    assert!(result.is_ok());

    // Verify initialization
    assert!(client.is_initialized());
}

#[test]
fn test_process_clinical_note() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();

    // Create a clinical note
    let note_text = String::from_str(
        &env,
        "Patient presents with hypertension and chest pain. Blood pressure is elevated at 150/90. Prescribed aspirin and recommended lifestyle modifications."
    );

    let note_id = BytesN::from_array(&env, &[0u8; 32]);
    let patient_id = Address::random(&env);
    let record_id = BytesN::from_array(&env, &[1u8; 32]);

    // Process the note
    let result = client.process_clinical_note(&note_text, &note_id, &patient_id, &record_id, &0);

    assert!(result.is_ok());
    let nlp_result = result.unwrap();

    // Verify entities were extracted
    assert!(!nlp_result.entities.is_empty());

    // Verify concepts were extracted
    assert!(!nlp_result.concepts.is_empty());

    // Verify sentiment was analyzed
    assert!(nlp_result.sentiment.is_some());

    // Verify coding suggestions were generated
    assert!(!nlp_result.coding_suggestions.is_empty());

    // Verify processing time is reasonable
    assert!(nlp_result.processing_time_ms < 2000);

    // Verify accuracy score
    assert!(nlp_result.accuracy_score_bps > 0);
}

#[test]
fn test_extract_entities() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();

    // Create text with medical terms
    let text = String::from_str(
        &env,
        "Patient has diabetes mellitus type 2 and hypertension. Symptoms include dyspnea and chest pain."
    );

    // Extract entities
    let result = client.extract_entities(&text);
    assert!(result.is_ok());

    let entities = result.unwrap();
    assert!(!entities.is_empty());

    // Verify entity types
    let mut found_diagnosis = false;
    let mut found_symptom = false;

    for entity in entities.iter() {
        if entity.entity_type == String::from_str(&env, "Diagnosis") {
            found_diagnosis = true;
        }
        if entity.entity_type == String::from_str(&env, "Symptom") {
            found_symptom = true;
        }
    }

    assert!(found_diagnosis);
    assert!(found_symptom);
}

#[test]
fn test_analyze_sentiment() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();

    // Test positive sentiment
    let positive_text = String::from_str(
        &env,
        "The patient is very satisfied with the excellent care provided. The staff was helpful and professional."
    );

    let result = client.analyze_sentiment(&positive_text);
    assert!(result.is_ok());

    let sentiment = result.unwrap();
    assert!(sentiment.score > 0);
    assert!(
        sentiment.label == sentiment::SentimentLabel::Positive
            || sentiment.label == sentiment::SentimentLabel::VeryPositive
    );

    // Test negative sentiment
    let negative_text = String::from_str(
        &env,
        "The patient is very unhappy with the terrible service. The staff was rude and unprofessional."
    );

    let result = client.analyze_sentiment(&negative_text);
    assert!(result.is_ok());

    let sentiment = result.unwrap();
    assert!(sentiment.score < 0);
    assert!(
        sentiment.label == sentiment::SentimentLabel::Negative
            || sentiment.label == sentiment::SentimentLabel::VeryNegative
    );
}

#[test]
fn test_generate_coding_suggestions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();

    // Create text with medical conditions
    let text = String::from_str(
        &env,
        "Patient diagnosed with essential hypertension and type 2 diabetes mellitus. Performed ECG and blood draw."
    );

    // Generate coding suggestions
    let result = client.generate_coding_suggestions(&text, &5);
    assert!(result.is_ok());

    let suggestions = result.unwrap();
    assert!(!suggestions.is_empty());

    // Verify suggestions have required fields
    for suggestion in suggestions.iter() {
        assert!(!suggestion.code.is_empty());
        assert!(!suggestion.description.is_empty());
        assert!(suggestion.confidence_bps > 0);
    }
}

#[test]
fn test_processing_stats() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();

    // Process a note
    let note_text = String::from_str(&env, "Patient has hypertension.");
    let note_id = BytesN::from_array(&env, &[0u8; 32]);
    let patient_id = Address::random(&env);
    let record_id = BytesN::from_array(&env, &[1u8; 32]);

    client
        .process_clinical_note(&note_text, &note_id, &patient_id, &record_id, &0)
        .unwrap();

    // Get stats
    let stats = client.get_processing_stats().unwrap();

    assert_eq!(stats.total_notes_processed, 1);
    assert!(stats.total_processing_time_ms > 0);
    assert!(stats.entities_extracted > 0);
}

#[test]
fn test_phi_detection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();

    // Create text with PHI
    let text_with_phi = String::from_str(
        &env,
        "Patient SSN: 123-45-6789. Date of birth: 01/15/1980. Address: 123 Main St.",
    );

    let note_id = BytesN::from_array(&env, &[0u8; 32]);
    let patient_id = Address::random(&env);
    let record_id = BytesN::from_array(&env, &[1u8; 32]);

    // Process note with PHI
    let result =
        client.process_clinical_note(&text_with_phi, &note_id, &patient_id, &record_id, &0);

    assert!(result.is_ok());
    let nlp_result = result.unwrap();

    // Verify PHI was detected
    assert!(nlp_result.phi_detected);
}

#[test]
fn test_empty_note_error() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let admin = Address::random(&env);

    // Initialize contract
    client.initialize(&admin).unwrap();

    // Try to process empty note
    let empty_text = String::from_str(&env, "");
    let note_id = BytesN::from_array(&env, &[0u8; 32]);
    let patient_id = Address::random(&env);
    let record_id = BytesN::from_array(&env, &[1u8; 32]);

    let result = client.process_clinical_note(&empty_text, &note_id, &patient_id, &record_id, &0);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::EmptyClinicalNote);
}

#[test]
fn test_version() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ClinicalNLP);
    let client = ClinicalNLPClient::new(&env, &contract_id);

    let version = client.version(&env);
    assert_eq!(version, 1);
}
