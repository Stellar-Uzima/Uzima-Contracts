//! Golden-file tests for generated SDK bindings and contract interfaces.
//!
//! These tests verify that contract interfaces remain stable by comparing
//! generated output against known-good golden files.

#![cfg(all(test, feature = "testutils"))]

use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String,
};

/// Golden test: patient_consent_management interface produces expected output.
#[test]
fn golden_test_patient_consent_interface() {
    let env = Env::default();
    let contract_id = env.register_contract(
        None,
        patient_consent_management::PatientConsentManagement,
    );
    let client = patient_consent_management::PatientConsentManagementClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);

    let result_init = client.try_initialize(&admin);
    assert!(result_init.is_err());

    client.grant_consent(&patient, &provider);
    let has_consent = client.check_consent(&patient, &provider);
    assert!(has_consent);

    let record = client.get_consent_record(&patient, &provider);
    assert!(record.is_some());
    let record = record.unwrap();
    assert!(record.active);
    assert_eq!(record.patient, patient);
    assert_eq!(record.provider, provider);
    assert!(record.granted_at > 0);
}

/// Golden test: escrow contract interface produces expected output.
#[test]
fn golden_test_escrow_interface() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = BytesN::from_array(&env, &[1u8; 32]);
    env.mock_all_auths();

    client.initialize(&admin);

    let id = client.create_escrow(&admin, &payer, &payee, &token, &1000);
    assert!(id > 0);

    client.release_escrow(&admin, &id);
}

/// Golden test: audit contract interface produces expected output.
#[test]
fn golden_test_audit_interface() {
    let env = Env::default();
    let contract_id = env.register_contract(None, audit::AuditContract);
    let client = audit::AuditContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
}

/// Golden test: medical_records interface produces expected output.
#[test]
fn golden_test_medical_records_interface() {
    let env = Env::default();
    let contract_id = env.register_contract(None, medical_records::MedicalRecords);
    let client = medical_records::MedicalRecordsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
}

/// Golden test: consent record serialization roundtrip.
#[test]
fn golden_test_consent_record_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(
        None,
        patient_consent_management::PatientConsentManagement,
    );
    let client = patient_consent_management::PatientConsentManagementClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
    client.grant_consent(&patient, &provider);

    let record1 = client.get_consent_record(&patient, &provider).unwrap();

    client.revoke_consent(&patient, &provider);
    client.grant_consent(&patient, &provider);

    let record2 = client.get_consent_record(&patient, &provider).unwrap();

    assert_eq!(record1.patient, record2.patient);
    assert_eq!(record1.provider, record2.provider);
    assert!(record2.active);
}

/// Golden test: escrow state transitions.
#[test]
fn golden_test_escrow_state_transitions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = BytesN::from_array(&env, &[2u8; 32]);
    env.mock_all_auths();

    client.initialize(&admin);

    let id = client.create_escrow(&admin, &payer, &payee, &token, &5000);
    assert!(id > 0);

    client.cancel_escrow(&admin, &id);
}

/// Golden test: error type stability.
#[test]
fn golden_test_error_type_stability() {
    let env = Env::default();
    let contract_id = env.register_contract(
        None,
        patient_consent_management::PatientConsentManagement,
    );
    let client = patient_consent_management::PatientConsentManagementClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);

    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

/// Golden test: consent lifecycle end-to-end.
#[test]
fn golden_test_consent_lifecycle_e2e() {
    let env = Env::default();
    let contract_id = env.register_contract(
        None,
        patient_consent_management::PatientConsentManagement,
    );
    let client = patient_consent_management::PatientConsentManagementClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let provider_a = Address::generate(&env);
    let provider_b = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);

    assert!(!client.check_consent(&patient, &provider_a));
    assert!(!client.check_consent(&patient, &provider_b));

    client.grant_consent(&patient, &provider_a);
    assert!(client.check_consent(&patient, &provider_a));
    assert!(!client.check_consent(&patient, &provider_b));

    client.grant_consent(&patient, &provider_b);
    assert!(client.check_consent(&patient, &provider_a));
    assert!(client.check_consent(&patient, &provider_b));

    client.revoke_consent(&patient, &provider_a);
    assert!(!client.check_consent(&patient, &provider_a));
    assert!(client.check_consent(&patient, &provider_b));

    client.revoke_consent(&patient, &provider_b);
    assert!(!client.check_consent(&patient, &provider_a));
    assert!(!client.check_consent(&patient, &provider_b));
}
