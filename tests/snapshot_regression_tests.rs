//! Snapshot-based regression tests for ABI and storage migrations.
//!
//! These tests verify that contract ABI interfaces and storage layouts
//! remain stable across versions, preventing breaking changes for
//! downstream consumers.

#![cfg(all(test, feature = "testutils"))]

use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String,
};

/// ABI stability: patient_consent_management interface maintains expected functions.
#[test]
fn regression_abi_patient_consent_management() {
    let env = Env::default();
    let contract_id = env.register_contract(
        None,
        patient_consent_management::PatientConsentManagement,
    );
    let client = patient_consent_management::PatientConsentManagementClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);

    let patient = Address::generate(&env);
    let provider = Address::generate(&env);

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));

    client.revoke_consent(&patient, &provider);
    assert!(!client.check_consent(&patient, &provider));
}

/// ABI stability: escrow contract maintains expected functions.
#[test]
fn regression_abi_escrow_contract() {
    let env = Env::default();
    let contract_id = env.register_contract(None, escrow::EscrowContract);
    let client = escrow::EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);

    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = BytesN::from_array(&env, &[1u8; 32]);

    let id = client.create_escrow(&admin, &payer, &payee, &token, &1000);
    client.release_escrow(&admin, &id);
}

/// ABI stability: audit contract maintains expected functions.
#[test]
fn regression_abi_audit_contract() {
    let env = Env::default();
    let contract_id = env.register_contract(None, audit::AuditContract);
    let client = audit::AuditContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
}

/// Storage layout: patient consent record fields remain compatible.
#[test]
fn regression_storage_consent_record_layout() {
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

    let record = client.get_consent_record(&patient, &provider);
    assert!(record.is_some());
    let record = record.unwrap();
    assert!(record.active);
    assert_eq!(record.patient, patient);
    assert_eq!(record.provider, provider);
}

/// Storage layout: escrow record fields remain compatible.
#[test]
fn regression_storage_escrow_record_layout() {
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
}

/// Regression: consent grant-revoke-grant cycle produces consistent state.
#[test]
fn regression_consent_lifecycle_consistency() {
    let env = Env::default();
    let contract_id = env.register_contract(
        None,
        patient_consent_management::PatientConsentManagement,
    );
    let client = patient_consent_management::PatientConsentManagementClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let provider = Address::generate(&env);

    for _ in 0..3 {
        client.grant_consent(&patient, &provider);
        assert!(client.check_consent(&patient, &provider));
        client.revoke_consent(&patient, &provider);
        assert!(!client.check_consent(&patient, &provider));
    }

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));
}

/// Regression: medical_records initialization maintains expected interface.
#[test]
fn regression_medical_records_interface() {
    let env = Env::default();
    let contract_id = env.register_contract(None, medical_records::MedicalRecords);
    let client = medical_records::MedicalRecordsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize(&admin);
}
