/// Comprehensive integration tests for healthcare workflow scenarios
use soroban_sdk::{vec, String, testutils::Address as _};
use crate::utils::{IntegrationTestEnv};
use medical_records::{Role};

#[test]
fn test_user_registration_workflow() {
    let test_env = IntegrationTestEnv::new();
    let (_, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    // Workflow: Admin registers doctor and patient
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Verify roles
    assert_eq!(records_client.get_user_role(doctor), Role::Doctor);
    assert_eq!(records_client.get_user_role(patient), Role::Patient);
}

#[test]
fn test_record_creation_retrieval_workflow() {
    let test_env = IntegrationTestEnv::new();
    let (records_id, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // 1. Doctor creates a medical record
    let diagnosis = String::from_str(&test_env.env, "Integration Test Diagnosis");
    let treatment = String::from_str(&test_env.env, "Automated Treatment");
    
    let record_id = records_client.add_record(
        doctor,
        patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![&test_env.env, String::from_str(&test_env.env, "test")],
        &String::from_str(&test_env.env, "General"),
        &String::from_str(&test_env.env, "Testing"),
        &String::from_str(&test_env.env, "QmHash"),
    );

    // 2. Retrieve and verify
    let record = records_client.get_record(patient, &record_id);
    assert_eq!(record.diagnosis, diagnosis);
    assert_eq!(record.doctor_id, *doctor);

    // 3. Verify event
    test_env.assert_event_topics(&records_id, test_env.topics(&["EVENT", "REC_NEW"]));
}

#[test]
fn test_pause_emergency_workflow() {
    let test_env = IntegrationTestEnv::new();
    let (_, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;

    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);

    // Pause contract
    records_client.pause(admin);
    assert!(records_client.is_paused());

    // Try to add record while paused (should fail)
    let res = records_client.try_add_record(
        doctor,
        patient,
        &String::from_str(&test_env.env, "D"),
        &String::from_str(&test_env.env, "T"),
        &false,
        &vec![&test_env.env],
        &String::from_str(&test_env.env, "C"),
        &String::from_str(&test_env.env, "T"),
        &String::from_str(&test_env.env, "H"),
    );
    assert!(res.is_err());

    // Unpause
    records_client.unpause(admin);
    assert!(!records_client.is_paused());
}

