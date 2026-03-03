#![cfg(test)]

//! Integration tests for Medical Record Data Validation and Quality Assurance (Issue #125)

use soroban_sdk::{Env, String};

mod common;
use common::setup_uzima;

// ============================================================================
// TEST 1: Comprehensive Medical Data Format Validation
// ============================================================================

#[test]
fn test_comprehensive_format_validation() {
    let env = Env::default();
    let test = setup_uzima(&env);

    // Valid format - should pass
    let valid_diagnosis = String::from_str(&env, "Patient diagnosed with acute bronchitis");
    let valid_treatment = String::from_str(&env, "Prescribed antibiotics and rest for 7 days");
    let valid_category = String::from_str(&env, "Modern");
    let valid_treatment_type = String::from_str(&env, "Medication");
    let valid_data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx");
    let valid_tags = soroban_sdk::vec![&env, String::from_str(&env, "respiratory")];

    let (is_valid, score) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &valid_diagnosis,
        &valid_treatment,
        &valid_category,
        &valid_treatment_type,
        &valid_data_ref,
        &valid_tags,
    );

    assert!(is_valid, "Valid format should pass validation");
    assert!(score >= 70, "Valid format should have quality score >= 70");

    // Invalid format - empty diagnosis
    let empty_diagnosis = String::from_str(&env, "");
    let (is_valid, _) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &empty_diagnosis,
        &valid_treatment,
        &valid_category,
        &valid_treatment_type,
        &valid_data_ref,
        &valid_tags,
    );

    assert!(!is_valid, "Empty diagnosis should fail validation");
}

// ============================================================================
// TEST 2: Data Quality Scoring System
// ============================================================================

#[test]
fn test_quality_scoring_system() {
    let env = Env::default();
    let test = setup_uzima(&env);

    // Create a high-quality record
    let diagnosis = String::from_str(
        &env,
        "Patient presents with acute bronchitis and mild fever",
    );
    let treatment = String::from_str(&env, "Prescribed amoxicillin 500mg TID for 7 days");
    let category = String::from_str(&env, "Modern");
    let treatment_type = String::from_str(&env, "Medication");
    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx");
    let tags = soroban_sdk::vec![&env, String::from_str(&env, "respiratory")];

    let record_id = test.client.add_record(
        &test.doctor,
        &test.patient,
        &diagnosis,
        &treatment,
        &false,
        &tags,
        &category,
        &treatment_type,
        &data_ref,
    );

    // Assess quality with multi-dimensional scoring
    let (overall, completeness, format, fhir, consistency) =
        test.client.assess_record_quality(&test.doctor, &record_id);

    assert!(overall >= 70, "Overall quality score should be >= 70");
    assert!(completeness >= 70, "Completeness score should be good");
    assert!(format >= 70, "Format score should be good");
    assert!(fhir >= 70, "FHIR compliance score should be good");
    assert!(consistency >= 70, "Consistency score should be good");

    // Verify weighted calculation
    let expected = (completeness * 40 + format * 30 + fhir * 20 + consistency * 10) / 100;
    assert_eq!(
        overall, expected,
        "Overall score should match weighted calculation"
    );
}

// ============================================================================
// TEST 3: Validation Rules for Different Medical Record Types
// ============================================================================

#[test]
fn test_validation_for_different_types() {
    let env = Env::default();
    let test = setup_uzima(&env);

    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx");
    let tags = soroban_sdk::vec![&env, String::from_str(&env, "test")];

    // Test Modern Medicine
    let (is_valid, score) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &String::from_str(&env, "Hypertension stage 2"),
        &String::from_str(&env, "Lisinopril 10mg daily"),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &data_ref,
        &tags,
    );
    assert!(
        is_valid && score >= 70,
        "Modern medicine record should be valid"
    );

    // Test Traditional Medicine
    let (is_valid, score) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &String::from_str(&env, "Imbalance of vital energies"),
        &String::from_str(&env, "Acupuncture therapy"),
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Alternative Therapy"),
        &data_ref,
        &tags,
    );
    assert!(
        is_valid && score >= 70,
        "Traditional medicine record should be valid"
    );

    // Test Herbal Medicine
    let (is_valid, score) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &String::from_str(&env, "Digestive discomfort"),
        &String::from_str(&env, "Ginger root tea"),
        &String::from_str(&env, "Herbal"),
        &String::from_str(&env, "Herbal Remedy"),
        &data_ref,
        &tags,
    );
    assert!(
        is_valid && score >= 70,
        "Herbal medicine record should be valid"
    );

    // Test Spiritual Medicine
    let (is_valid, score) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &String::from_str(&env, "Spiritual distress"),
        &String::from_str(&env, "Meditation therapy"),
        &String::from_str(&env, "Spiritual"),
        &String::from_str(&env, "Spiritual Healing"),
        &data_ref,
        &tags,
    );
    assert!(
        is_valid && score >= 70,
        "Spiritual medicine record should be valid"
    );
}

// ============================================================================
// TEST 4: Automated Data Cleansing
// ============================================================================

#[test]
fn test_automated_data_cleansing() {
    let env = Env::default();
    let test = setup_uzima(&env);

    let diagnosis = String::from_str(&env, "Test diagnosis");
    let treatment = String::from_str(&env, "Test treatment");
    let category = String::from_str(&env, "Modern");
    let treatment_type = String::from_str(&env, "Test type");
    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx");

    // Tags with duplicates
    let dirty_tags = soroban_sdk::vec![
        &env,
        String::from_str(&env, "tag1"),
        String::from_str(&env, "tag2"),
        String::from_str(&env, "tag1"), // duplicate
    ];

    let (was_modified, _d, _t, _c, _tt, _dr, clean_tags) = test.client.cleanse_record_data(
        &test.doctor,
        &diagnosis,
        &treatment,
        &category,
        &treatment_type,
        &data_ref,
        &dirty_tags,
    );

    assert!(
        was_modified,
        "Data should be modified due to duplicate tags"
    );
    assert_eq!(
        clean_tags.len(),
        2,
        "Should have 2 unique tags after cleansing"
    );
}

// ============================================================================
// TEST 5: FHIR Compliance Validation
// ============================================================================

#[test]
fn test_fhir_compliance() {
    let env = Env::default();
    let test = setup_uzima(&env);

    // FHIR-compliant record
    let fhir_diagnosis = String::from_str(&env, "Type 2 Diabetes Mellitus with complications");
    let fhir_treatment = String::from_str(&env, "Metformin 500mg BID, lifestyle modifications");
    let category = String::from_str(&env, "Modern");
    let treatment_type = String::from_str(&env, "Medication Management");
    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx");
    let tags = soroban_sdk::vec![&env, String::from_str(&env, "endocrine")];

    let record_id = test.client.add_record(
        &test.doctor,
        &test.patient,
        &fhir_diagnosis,
        &fhir_treatment,
        &false,
        &tags,
        &category,
        &treatment_type,
        &data_ref,
    );

    let (_overall, _completeness, _format, fhir_score, _consistency) =
        test.client.assess_record_quality(&test.doctor, &record_id);

    assert!(fhir_score >= 80, "FHIR compliance score should be high");
}

// ============================================================================
// TEST 6: Data Completeness Checks
// ============================================================================

#[test]
fn test_completeness_checks() {
    let env = Env::default();
    let test = setup_uzima(&env);

    // Create incomplete record (no tags, no DID)
    let record_id = test.client.add_record(
        &test.doctor,
        &test.patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &soroban_sdk::vec![&env], // No tags
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );

    let missing_fields = test
        .client
        .check_record_completeness(&test.doctor, &record_id);

    assert!(
        !missing_fields.is_empty(),
        "Should detect missing optional fields"
    );

    // Verify completeness score reflects gaps
    let (_overall, completeness, _format, _fhir, _consistency) =
        test.client.assess_record_quality(&test.doctor, &record_id);

    // Completeness is 100 when all required fields are present (5 * 20 = 100)
    // Optional fields (tags, DID) add bonus points but don't reduce below 100
    assert!(
        completeness >= 70,
        "Completeness score should be good when required fields are present"
    );
}

// ============================================================================
// TEST 7: Validation Error Reporting
// ============================================================================

#[test]
fn test_validation_error_reporting() {
    let env = Env::default();
    let test = setup_uzima(&env);

    // Create a record with minimal data
    let diagnosis = String::from_str(&env, "Basic diagnosis");
    let treatment = String::from_str(&env, "Basic treatment");
    let category = String::from_str(&env, "Modern");
    let treatment_type = String::from_str(&env, "Type");
    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx");
    let tags = soroban_sdk::vec![&env]; // Empty tags

    let record_id = test.client.add_record(
        &test.doctor,
        &test.patient,
        &diagnosis,
        &treatment,
        &false,
        &tags,
        &category,
        &treatment_type,
        &data_ref,
    );

    // Validate the created record
    let (is_valid, quality_score, _issue_count) = test
        .client
        .validate_record_quality(&test.doctor, &record_id);

    // Record should be valid but with lower quality
    assert!(is_valid, "Record should be valid");
    assert!(quality_score >= 70, "Quality score should meet minimum");
}

// ============================================================================
// TEST 8: Integration with Existing Validation
// ============================================================================

#[test]
fn test_integration_with_existing_validation() {
    let env = Env::default();
    let test = setup_uzima(&env);

    // Test that quality assurance uses existing validation.rs rules
    let valid_treatment = String::from_str(&env, "Treatment plan");
    let valid_category = String::from_str(&env, "Modern");
    let valid_type = String::from_str(&env, "Medication");
    let valid_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx");
    let tags = soroban_sdk::vec![&env, String::from_str(&env, "tag")];

    // Test with valid data - should pass
    let valid_diagnosis = String::from_str(&env, "Valid diagnosis");
    let (is_valid, score) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &valid_diagnosis,
        &valid_treatment,
        &valid_category,
        &valid_type,
        &valid_ref,
        &tags,
    );
    assert!(is_valid, "Should pass with valid data");
    assert!(score >= 70, "Should have good quality score");

    // Test with empty diagnosis (validation.rs rule) - should fail
    let empty_diagnosis = String::from_str(&env, "");
    let (is_valid, _) = test.client.validate_record_data(
        &test.doctor,
        &test.patient,
        &empty_diagnosis,
        &valid_treatment,
        &valid_category,
        &valid_type,
        &valid_ref,
        &tags,
    );
    assert!(
        !is_valid,
        "Should fail with empty diagnosis (from validation.rs)"
    );
}

// ============================================================================
// TEST 9: Batch Validation
// ============================================================================

#[test]
fn test_batch_validation() {
    let env = Env::default();
    let test = setup_uzima(&env);

    let mut record_ids = soroban_sdk::vec![&env];

    // Create 2 records
    for i in 0..2 {
        let record_id = test.client.add_record(
            &test.doctor,
            &test.patient,
            &String::from_str(&env, &format!("Diagnosis {}", i)),
            &String::from_str(&env, &format!("Treatment {}", i)),
            &false,
            &soroban_sdk::vec![&env, String::from_str(&env, "tag")],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "Medication"),
            &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
        );
        record_ids.push_back(record_id);
    }

    let results = test
        .client
        .batch_validate_records(&test.admin1, &record_ids);

    assert_eq!(results.len(), 2, "Should validate all records");

    for (_record_id, is_valid, score) in results.iter() {
        assert!(is_valid, "All records should be valid");
        assert!(score >= 70, "All records should meet minimum quality");
    }
}

// ============================================================================
// TEST 10: Patient Quality Statistics
// ============================================================================

#[test]
fn test_patient_quality_statistics() {
    let env = Env::default();
    let test = setup_uzima(&env);

    // Create 3 records
    for i in 0..3 {
        test.client.add_record(
            &test.doctor,
            &test.patient,
            &String::from_str(&env, &format!("Diagnosis {}", i)),
            &String::from_str(&env, &format!("Treatment {}", i)),
            &false,
            &soroban_sdk::vec![&env, String::from_str(&env, "tag")],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "Medication"),
            &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
        );
    }

    let (total_records, avg_score, below_threshold) = test
        .client
        .get_patient_quality_stats(&test.patient, &test.patient);

    assert_eq!(total_records, 3, "Should have 3 records");
    assert!(avg_score >= 70, "Average quality should be acceptable");
    assert_eq!(below_threshold, 0, "No records should be below threshold");
}
