#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, String, Vec};

fn setup(env: &Env) -> (FHIRIntegrationContractClient, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, FHIRIntegrationContract);
    (FHIRIntegrationContractClient::new(env, &id), id)
}

fn setup_initialized(env: &Env) -> (FHIRIntegrationContractClient, Address) {
    let (client, _id) = setup(env);
    let admin = Address::generate(env);
    let medical_records = Address::generate(env);
    client.initialize(&admin, &medical_records);
    (client, admin)
}

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    // Double initialization should fail
    let medical_records = Address::generate(&env);
    let result = client.try_initialize(&admin, &medical_records);
    assert!(result.is_err());
}

// ============================================================================
// PROVIDER REGISTRATION TESTS
// ============================================================================

#[test]
fn test_register_and_get_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let provider_id = String::from_str(&env, "PROV-001");
    let name = String::from_str(&env, "St. Mary Hospital");
    let facility_type = String::from_str(&env, "hospital");
    let npi = String::from_str(&env, "1234567890");
    let tax_id = String::from_str(&env, "12-3456789");
    let address = String::from_str(&env, "123 Main St");
    let contact = String::from_str(&env, "info@stmary.org");
    let emr = String::from_str(&env, "Epic Systems");
    let fhir = String::from_str(&env, "https://fhir.stmary.org");

    assert!(client.register_provider(
        &admin, &provider_id, &name, &facility_type, &npi,
        &tax_id, &address, &contact, &emr, &fhir,
    ));

    let provider = client.get_provider(&provider_id);
    assert_eq!(provider.name, name);
    assert!(!provider.is_verified);
}

#[test]
fn test_register_duplicate_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let provider_id = String::from_str(&env, "PROV-DUP");
    let npi = String::from_str(&env, "1234567890");
    let tax_id = String::from_str(&env, "12-3456789");

    let args = (&admin, &provider_id,
        &String::from_str(&env, "Name"),
        &String::from_str(&env, "hospital"),
        &npi, &tax_id,
        &String::from_str(&env, "addr"),
        &String::from_str(&env, "email"),
        &String::from_str(&env, "emr"),
        &String::from_str(&env, "fhir"),
    );
    assert!(client.register_provider(args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9));

    let result = client.try_register_provider(args.0, args.1, args.2, args.3, args.4, args.5, args.6, args.7, args.8, args.9);
    assert!(result.is_err());
}

#[test]
fn test_register_provider_invalid_npi() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let result = client.try_register_provider(
        &admin,
        &String::from_str(&env, "PROV-BAD"),
        &String::from_str(&env, "Name"),
        &String::from_str(&env, "hospital"),
        &String::from_str(&env, "1234"), // Invalid NPI (too short)
        &String::from_str(&env, "12-3456789"),
        &String::from_str(&env, "addr"),
        &String::from_str(&env, "email"),
        &String::from_str(&env, "emr"),
        &String::from_str(&env, "fhir"),
    );
    assert!(result.is_err());
}

#[test]
fn test_register_provider_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);
    let attacker = Address::generate(&env);

    let result = client.try_register_provider(
        &attacker,
        &String::from_str(&env, "PROV-HACK"),
        &String::from_str(&env, "Name"),
        &String::from_str(&env, "hospital"),
        &String::from_str(&env, "1234567890"),
        &String::from_str(&env, "12-3456789"),
        &String::from_str(&env, "addr"),
        &String::from_str(&env, "email"),
        &String::from_str(&env, "emr"),
        &String::from_str(&env, "fhir"),
    );
    assert!(result.is_err());
}

// ============================================================================
// PROVIDER VERIFICATION TESTS
// ============================================================================

#[test]
fn test_verify_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let provider_id = String::from_str(&env, "PROV-VRFY");
    client.register_provider(
        &admin, &provider_id,
        &String::from_str(&env, "Name"),
        &String::from_str(&env, "hospital"),
        &String::from_str(&env, "1234567890"),
        &String::from_str(&env, "12-3456789"),
        &String::from_str(&env, "addr"),
        &String::from_str(&env, "email"),
        &String::from_str(&env, "emr"),
        &String::from_str(&env, "fhir"),
    );

    let cred_id = BytesN::from_array(&env, &[1u8; 32]);
    assert!(client.verify_provider(&admin, &provider_id, &cred_id));

    let provider = client.get_provider(&provider_id);
    assert!(provider.is_verified);
    assert_eq!(provider.credential_id, cred_id);
}

#[test]
fn test_verify_already_verified_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let provider_id = String::from_str(&env, "PROV-DBL");
    client.register_provider(
        &admin, &provider_id,
        &String::from_str(&env, "Name"),
        &String::from_str(&env, "hospital"),
        &String::from_str(&env, "1234567890"),
        &String::from_str(&env, "12-3456789"),
        &String::from_str(&env, "addr"),
        &String::from_str(&env, "email"),
        &String::from_str(&env, "emr"),
        &String::from_str(&env, "fhir"),
    );

    let cred_id = BytesN::from_array(&env, &[1u8; 32]);
    assert!(client.verify_provider(&admin, &provider_id, &cred_id));

    // Double verification should fail
    let result = client.try_verify_provider(&admin, &provider_id, &cred_id);
    assert!(result.is_err());
}

// ============================================================================
// OBSERVATION TESTS
// ============================================================================

#[test]
fn test_store_and_get_observation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);
    let provider = Address::generate(&env);

    let obs = FHIRObservation {
        identifier: String::from_str(&env, "OBS-001"),
        status: String::from_str(&env, "final"),
        code: FHIRCode {
            system: CodingSystem::LOINC,
            code: String::from_str(&env, "8867-4"),
            display: String::from_str(&env, "Heart rate"),
        },
        category: FHIRCode {
            system: CodingSystem::LOINC,
            code: String::from_str(&env, "vital-signs"),
            display: String::from_str(&env, "Vital Signs"),
        },
        subject_reference: String::from_str(&env, "Patient/123"),
        effective_datetime: String::from_str(&env, "2024-01-15T10:30:00Z"),
        value_quantity_value: 72,
        value_quantity_unit: String::from_str(&env, "bpm"),
        interpretation: Vec::new(&env),
        reference_range: String::from_str(&env, "60-100 bpm"),
    };

    assert!(client.store_observation(&provider, &obs));

    let retrieved = client.get_observation(&String::from_str(&env, "OBS-001"));
    assert_eq!(retrieved.value_quantity_value, 72);
}

#[test]
fn test_store_observation_invalid() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);
    let provider = Address::generate(&env);

    // Empty identifier should fail
    let obs = FHIRObservation {
        identifier: String::from_str(&env, ""),
        status: String::from_str(&env, "final"),
        code: FHIRCode {
            system: CodingSystem::LOINC,
            code: String::from_str(&env, "8867-4"),
            display: String::from_str(&env, "Heart rate"),
        },
        category: FHIRCode {
            system: CodingSystem::LOINC,
            code: String::from_str(&env, ""),
            display: String::from_str(&env, ""),
        },
        subject_reference: String::from_str(&env, "Patient/123"),
        effective_datetime: String::from_str(&env, ""),
        value_quantity_value: 72,
        value_quantity_unit: String::from_str(&env, "bpm"),
        interpretation: Vec::new(&env),
        reference_range: String::from_str(&env, ""),
    };

    let result = client.try_store_observation(&provider, &obs);
    assert!(result.is_err());
}

// ============================================================================
// CONDITION TESTS
// ============================================================================

#[test]
fn test_store_and_get_condition() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);
    let provider = Address::generate(&env);

    let condition = FHIRCondition {
        identifier: String::from_str(&env, "COND-001"),
        code: FHIRCode {
            system: CodingSystem::ICD10,
            code: String::from_str(&env, "E11.9"),
            display: String::from_str(&env, "Type 2 diabetes"),
        },
        clinical_status: String::from_str(&env, "active"),
        subject_reference: String::from_str(&env, "Patient/123"),
        onset_date_time: String::from_str(&env, "2023-06-01"),
        recorded_date: String::from_str(&env, "2024-01-15"),
        severity: Vec::new(&env),
    };

    assert!(client.store_condition(&provider, &condition));

    let retrieved = client.get_condition(&String::from_str(&env, "COND-001"));
    assert_eq!(retrieved.code.code, String::from_str(&env, "E11.9"));
}

// ============================================================================
// MEDICATION TESTS
// ============================================================================

#[test]
fn test_store_and_get_medication() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);
    let provider = Address::generate(&env);

    let medication = FHIRMedicationStatement {
        identifier: String::from_str(&env, "MED-001"),
        status: String::from_str(&env, "active"),
        medication_code: FHIRCode {
            system: CodingSystem::RxNorm,
            code: String::from_str(&env, "860975"),
            display: String::from_str(&env, "Metformin 500mg"),
        },
        subject_reference: String::from_str(&env, "Patient/123"),
        effective_period_start: String::from_str(&env, "2024-01-01"),
        effective_period_end: String::from_str(&env, ""),
        dosage: String::from_str(&env, "500mg twice daily"),
        reason_code: Vec::new(&env),
    };

    assert!(client.store_medication(&provider, &medication));

    let retrieved = client.get_medication(&String::from_str(&env, "MED-001"));
    assert_eq!(retrieved.medication_code.display, medication.medication_code.display);
}

// ============================================================================
// PROCEDURE TESTS
// ============================================================================

#[test]
fn test_store_and_get_procedure() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);
    let provider = Address::generate(&env);

    let procedure = FHIRProcedure {
        identifier: String::from_str(&env, "PROC-001"),
        status: String::from_str(&env, "completed"),
        code: FHIRCode {
            system: CodingSystem::CPT,
            code: String::from_str(&env, "12345"),
            display: String::from_str(&env, "Routine checkup"),
        },
        subject_reference: String::from_str(&env, "Patient/123"),
        performed_date_time: String::from_str(&env, "2024-01-15"),
        performer: Vec::new(&env),
        reason_code: Vec::new(&env),
    };

    assert!(client.store_procedure(&provider, &procedure));

    let retrieved = client.get_procedure(&String::from_str(&env, "PROC-001"));
    assert_eq!(retrieved.status, String::from_str(&env, "completed"));
}

// ============================================================================
// ALLERGY TESTS
// ============================================================================

#[test]
fn test_store_and_get_allergy() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);
    let provider = Address::generate(&env);

    let allergy = FHIRAllergyIntolerance {
        identifier: String::from_str(&env, "ALL-001"),
        clinical_status: String::from_str(&env, "active"),
        verification_status: String::from_str(&env, "confirmed"),
        substance_code: FHIRCode {
            system: CodingSystem::RxNorm,
            code: String::from_str(&env, "1191"),
            display: String::from_str(&env, "Penicillin"),
        },
        patient_reference: String::from_str(&env, "Patient/123"),
        recorded_date: String::from_str(&env, "2024-01-15"),
        manifestation: Vec::new(&env),
        severity: String::from_str(&env, "severe"),
    };

    assert!(client.store_allergy(&provider, &allergy));

    let retrieved = client.get_allergy(&String::from_str(&env, "ALL-001"));
    assert_eq!(retrieved.severity, String::from_str(&env, "severe"));
}

// ============================================================================
// DATA MAPPING TESTS
// ============================================================================

#[test]
fn test_register_and_get_data_mapping() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);

    let mapping = DataMapping {
        source_system: String::from_str(&env, "Epic"),
        source_field: String::from_str(&env, "PAT_NAME"),
        target_system: String::from_str(&env, "FHIR"),
        target_field: String::from_str(&env, "Patient.name"),
        transformation_rule: String::from_str(&env, "Direct mapping"),
        status: String::from_str(&env, "active"),
    };

    assert!(client.register_data_mapping(&admin, &mapping));

    let retrieved = client.get_data_mapping(
        &String::from_str(&env, "Epic"),
        &String::from_str(&env, "PAT_NAME"),
    );
    assert_eq!(retrieved.target_field, mapping.target_field);
}

// ============================================================================
// PAUSE / RESUME TESTS
// ============================================================================

#[test]
fn test_pause_and_resume() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_initialized(&env);
    let provider = Address::generate(&env);

    assert!(client.pause(&admin));

    // Operations should be blocked while paused
    let result = client.try_register_provider(
        &admin,
        &String::from_str(&env, "PROV-PAUSED"),
        &String::from_str(&env, "Name"),
        &String::from_str(&env, "hospital"),
        &String::from_str(&env, "1234567890"),
        &String::from_str(&env, "12-3456789"),
        &String::from_str(&env, "addr"),
        &String::from_str(&env, "email"),
        &String::from_str(&env, "emr"),
        &String::from_str(&env, "fhir"),
    );
    assert!(result.is_err());

    assert!(client.resume(&admin));
}

// ============================================================================
// ERROR PATH TESTS
// ============================================================================

#[test]
fn test_get_nonexistent_provider() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);

    let result = client.try_get_provider(&String::from_str(&env, "NONEXISTENT"));
    assert!(result.is_err());
}

#[test]
fn test_get_nonexistent_observation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_initialized(&env);

    let result = client.try_get_observation(&String::from_str(&env, "NONEXISTENT"));
    assert!(result.is_err());
}
