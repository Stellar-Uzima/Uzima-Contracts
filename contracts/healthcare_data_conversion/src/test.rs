#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, String};

fn setup(env: &Env) -> (HealthcareDataConversionContractClient, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, HealthcareDataConversionContract);
    (HealthcareDataConversionContractClient::new(env, &id), id)
}

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    assert!(client.initialize(&admin).is_ok());

    // Double initialization should fail
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

#[test]
fn test_initialize_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let other = Address::generate(&env);

    // First init as admin
    assert!(client.initialize(&admin).is_ok());

    // Other address trying to init should fail
    let result = client.try_initialize(&other);
    assert!(result.is_err());
}

// ============================================================================
// CONVERSION RULE TESTS
// ============================================================================

#[test]
fn test_register_and_get_conversion_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let rule = ConversionRule {
        rule_id: String::from_str(&env, "rule-001"),
        source_format: DataFormat::FHIRJSON,
        target_format: DataFormat::HL7v2,
        source_path: String::from_str(&env, "$.patient.name"),
        target_path: String::from_str(&env, "PID.5"),
        transformation_type: String::from_str(&env, "direct"),
        field_type: FieldType::String,
        mapping_table_ref: String::from_str(&env, ""),
        validation_rules: Vec::new(&env),
        is_active: true,
    };

    assert!(client.register_conversion_rule(&admin, &rule));

    let retrieved = client.get_conversion_rule(&String::from_str(&env, "rule-001"));
    assert_eq!(retrieved.rule_id, rule.rule_id);
    assert_eq!(retrieved.source_format, DataFormat::FHIRJSON);
    assert_eq!(retrieved.target_format, DataFormat::HL7v2);
}

#[test]
fn test_register_duplicate_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let rule = ConversionRule {
        rule_id: String::from_str(&env, "rule-dup"),
        source_format: DataFormat::FHIRJSON,
        target_format: DataFormat::HL7v2,
        source_path: String::from_str(&env, "$.x"),
        target_path: String::from_str(&env, "y"),
        transformation_type: String::from_str(&env, "direct"),
        field_type: FieldType::String,
        mapping_table_ref: String::from_str(&env, ""),
        validation_rules: Vec::new(&env),
        is_active: true,
    };

    assert!(client.register_conversion_rule(&admin, &rule));
    let result = client.try_register_conversion_rule(&admin, &rule);
    assert!(result.is_err());
}

#[test]
fn test_register_rule_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin);

    let rule = ConversionRule {
        rule_id: String::from_str(&env, "rule-hack"),
        source_format: DataFormat::FHIRJSON,
        target_format: DataFormat::HL7v2,
        source_path: String::from_str(&env, "$"),
        target_path: String::from_str(&env, "x"),
        transformation_type: String::from_str(&env, "direct"),
        field_type: FieldType::String,
        mapping_table_ref: String::from_str(&env, ""),
        validation_rules: Vec::new(&env),
        is_active: true,
    };

    let result = client.try_register_conversion_rule(&attacker, &rule);
    assert!(result.is_err());
}

// ============================================================================
// CODING MAPPING TESTS
// ============================================================================

#[test]
fn test_register_and_get_coding_mapping() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let mapping = CodingMapping {
        mapping_id: String::from_str(&env, "icd9-icd10-001"),
        source_code_system: String::from_str(&env, "ICD9"),
        target_code_system: String::from_str(&env, "ICD10"),
        source_code: String::from_str(&env, "250.00"),
        target_code: String::from_str(&env, "E11.9"),
        source_description: String::from_str(&env, "Diabetes mellitus"),
        target_description: String::from_str(&env, "Type 2 diabetes"),
        confidence_score: 95,
        backward_mapping: None,
        effective_date: String::from_str(&env, "2024-01-01"),
        end_date: String::from_str(&env, ""),
    };

    assert!(client.register_coding_mapping(&admin, &mapping));

    let retrieved = client.get_coding_mapping(&String::from_str(&env, "icd9-icd10-001"));
    assert_eq!(retrieved.source_code, mapping.source_code);
    assert_eq!(retrieved.target_code, mapping.target_code);
    assert_eq!(retrieved.confidence_score, 95);
}

#[test]
fn test_register_invalid_coding_mapping() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Empty source code
    let mapping = CodingMapping {
        mapping_id: String::from_str(&env, "bad-map"),
        source_code_system: String::from_str(&env, "ICD9"),
        target_code_system: String::from_str(&env, "ICD10"),
        source_code: String::from_str(&env, ""),
        target_code: String::from_str(&env, "E11.9"),
        source_description: String::from_str(&env, "Test"),
        target_description: String::from_str(&env, "Test"),
        confidence_score: 95,
        backward_mapping: None,
        effective_date: String::from_str(&env, "2024-01-01"),
        end_date: String::from_str(&env, ""),
    };

    let result = client.try_register_coding_mapping(&admin, &mapping);
    assert!(result.is_err());
}

// ============================================================================
// FORMAT SPECIFICATION TESTS
// ============================================================================

#[test]
fn test_register_and_get_format_specification() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let spec = FormatSpecification {
        format: DataFormat::HL7v2,
        version: String::from_str(&env, "2.5.1"),
        mime_type: String::from_str(&env, "application/hl7-v2"),
        encoding: String::from_str(&env, "UTF-8"),
        character_set: String::from_str(&env, "UTF-8"),
        supported_resources: Vec::new(&env),
        description: String::from_str(&env, "HL7 v2.5.1"),
        standard_url: String::from_str(&env, "https://www.hl7.org"),
    };

    assert!(client.register_format_specification(&admin, &spec));

    let retrieved = client.get_format_specification(&DataFormat::HL7v2);
    assert_eq!(retrieved.version, spec.version);
}

// ============================================================================
// VALIDATION TESTS
// ============================================================================

#[test]
fn test_validate_conversion() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let data_hash = BytesN::from_array(&env, &[1u8; 32]);

    let result = client.validate_conversion(
        &admin,
        &DataFormat::FHIRJSON,
        &DataFormat::HL7v2,
        &data_hash,
    );
    assert!(result.is_valid);
}

#[test]
fn test_validate_unsupported_format() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let data_hash = BytesN::from_array(&env, &[1u8; 32]);

    // CSV format isn't registered by default
    let result = client.try_validate_conversion(
        &admin,
        &DataFormat::CSV,
        &DataFormat::FHIRJSON,
        &data_hash,
    );
    assert!(result.is_err());
}

// ============================================================================
// CONVERSION REQUEST TESTS
// ============================================================================

#[test]
fn test_record_and_get_conversion_request() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    let source_hash = BytesN::from_array(&env, &[1u8; 32]);
    let target_hash = BytesN::from_array(&env, &[2u8; 32]);

    let request_id = client.record_conversion(
        &user,
        &DataFormat::FHIRJSON,
        &DataFormat::HL7v2,
        &source_hash,
        &target_hash,
    );
    assert!(request_id > 0);

    let request = client.get_conversion_request(&request_id);
    assert_eq!(request.status, String::from_str(&env, "completed"));
}

// ============================================================================
// LOSSY CONVERSION WARNING TESTS
// ============================================================================

#[test]
fn test_record_lossy_conversion_warning() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let warning = LossyConversionWarning {
        warning_id: String::from_str(&env, "warn-001"),
        conversion_request_id: 1,
        lost_fields: Vec::new(&env),
        data_loss_percentage: 10,
        mitigation_recommendation: String::from_str(&env, "Review lost fields"),
    };

    assert!(client.record_lossy_conversion_warning(&admin, &warning));

    let retrieved = client.get_lossy_conversion_warning(&String::from_str(&env, "warn-001"));
    assert_eq!(retrieved.data_loss_percentage, 10);
}

#[test]
fn test_lossy_warning_invalid_percentage() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Data loss percentage > 100 should fail
    let warning = LossyConversionWarning {
        warning_id: String::from_str(&env, "warn-bad"),
        conversion_request_id: 1,
        lost_fields: Vec::new(&env),
        data_loss_percentage: 150,
        mitigation_recommendation: String::from_str(&env, ""),
    };

    let result = client.try_record_lossy_conversion_warning(&admin, &warning);
    assert!(result.is_err());
}

// ============================================================================
// PAUSE / RESUME TESTS
// ============================================================================

#[test]
fn test_pause_and_resume() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    client.initialize(&admin);

    // Pause the contract
    assert!(client.pause(&admin));

    // Operations should be blocked while paused
    let rule = ConversionRule {
        rule_id: String::from_str(&env, "rule-paused"),
        source_format: DataFormat::FHIRJSON,
        target_format: DataFormat::HL7v2,
        source_path: String::from_str(&env, "$"),
        target_path: String::from_str(&env, "x"),
        transformation_type: String::from_str(&env, "direct"),
        field_type: FieldType::String,
        mapping_table_ref: String::from_str(&env, ""),
        validation_rules: Vec::new(&env),
        is_active: true,
    };
    let result = client.try_register_conversion_rule(&admin, &rule);
    assert!(result.is_err());

    // Resume the contract
    assert!(client.resume(&admin));

    // Operations should work again
    assert!(client.register_conversion_rule(&admin, &rule));
}

#[test]
fn test_pause_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let other = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_pause(&other);
    assert!(result.is_err());
}

// ============================================================================
// ERROR PATH TESTS
// ============================================================================

#[test]
fn test_get_nonexistent_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_get_conversion_rule(&String::from_str(&env, "nonexistent"));
    assert!(result.is_err());
}

#[test]
fn test_get_nonexistent_coding_mapping() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_get_coding_mapping(&String::from_str(&env, "nonexistent"));
    assert!(result.is_err());
}

#[test]
fn test_get_nonexistent_conversion_request() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_get_conversion_request(&999);
    assert!(result.is_err());
}
