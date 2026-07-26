//! Integration tests for MedicalRecordsContract
//! These cover record creation, RBAC enforcement, pause/resume, and recovery flows.

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    vec, Address, Env, String,
};
use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};

#[test]
fn test_full_medical_record_workflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let diagnosis = String::from_str(&env, "Hypertension");
    let treatment = String::from_str(&env, "ACE inhibitor medication");

    // Start by mocking all auths for setup
    env.mock_all_auths();

    assert!(client.initialize(&admin));
    assert!(client.manage_user(&admin, &doctor, &Role::Doctor));
    assert!(client.manage_user(&admin, &patient, &Role::Patient));

    // Use mock_all_auths for the add_record call as well
    let record_id = client.add_record(
        &doctor,
        &patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![
            &env,
            String::from_str(&env, "herbal"),
            String::from_str(&env, "spiritual"),
        ],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );

    let record = client.get_record(&patient, &record_id);
    assert_eq!(record.patient_id, patient);
    assert_eq!(record.diagnosis, diagnosis);
    assert_eq!(record.category, String::from_str(&env, "Traditional"));
    assert_eq!(record.treatment_type, String::from_str(&env, "Herbal Therapy"));
    assert_eq!(record.tags.len(), 2);
}

#[test]
fn test_pause_blocks_add_record_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    assert!(client.initialize(&admin));
    assert!(client.manage_user(&admin, &doctor, &Role::Doctor));
    assert!(client.manage_user(&admin, &patient, &Role::Patient));

    assert!(client.pause(&admin));

    let res = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "herbal")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );
    assert!(res.is_err());
}

#[test]
fn test_recovery_flow_integration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);

    assert!(client.initialize(&admin1));
    assert!(client.manage_user(&admin1, &admin2, &Role::Admin));

    let proposal_id = client.propose_recovery(&admin1, &token, &recipient, &100i128);
    assert!(proposal_id > 0);

    assert!(client.approve_recovery(&admin2, &proposal_id));

    // Fail before timelock
    let res = client.try_execute_recovery(&admin1, &proposal_id);
    assert!(res.is_err());

    let now = env.ledger().timestamp();
    env.ledger().with_mut(|l| l.timestamp = now + 86_401);

    assert!(client.execute_recovery(&admin1, &proposal_id));
}
