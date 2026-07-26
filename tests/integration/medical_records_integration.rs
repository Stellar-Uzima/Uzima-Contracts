use crate::utils::{IntegrationTestEnv, UserRole};
use medical_records::{Role};
use soroban_sdk::{vec, String, testutils::{Address as _, MockAuth, MockAuthInvoke}, Env, Address};

#[test]
fn test_full_medical_record_workflow_with_framework() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;
    
    let (records_id, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;
    
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    let diagnosis = String::from_str(env, "Hypertension");
    let treatment = String::from_str(env, "ACE inhibitor");

    let record_id = records_client.add_record(
        doctor,
        patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![env, String::from_str(env, "herbal")],
        &String::from_str(env, "Traditional"),
        &String::from_str(env, "Herbal Therapy"),
        &String::from_str(env, "QmHash"),
    );

    let record = records_client.get_record(patient, &record_id);
    assert_eq!(record.patient_id, *patient);
    
    test_env.assert_event_topics(&records_id, test_env.topics(&["EVENT", "REC_NEW"]));
}

#[test]
fn test_multiple_records_and_audit_integration() {
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;
    let (records_id, records_client) = test_env.register_medical_records();
    
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;
    
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);

    // Add 3 records
    for i in 0..3 {
        let diagnosis = String::from_str(env, &format!("Diagnosis {}", i));
        records_client.add_record(
            doctor,
            patient,
            &diagnosis,
            &String::from_str(env, "Treatment"),
            &false,
            &vec![env],
            &String::from_str(env, "Cat"),
            &String::from_str(env, "Type"),
            &String::from_str(env, "Hash"),
        );
    }

    let count = records_client.get_patient_record_count(patient);
    assert_eq!(count, 3);
    
    // Check if we have 3 REC_NEW events
    let events = test_env.get_events();
    let record_events: Vec<_> = events.iter().filter(|(id, t, _)| {
        id == &records_id && t == &test_env.topics(&["EVENT", "REC_NEW"])
    }).collect();
    
    assert_eq!(record_events.len(), 3);
}
