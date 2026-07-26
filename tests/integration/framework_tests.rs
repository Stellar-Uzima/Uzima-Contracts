/// Integration tests demonstrating the usage of the integration testing framework
use soroban_sdk::{vec, String, testutils::Address as _};
use crate::utils::{IntegrationTestEnv, UserRole};
use medical_records::{Role};

#[test]
fn test_integration_framework_comprehensive() {
    // Initialize the unified testing environment
    let test_env = IntegrationTestEnv::new();
    let env = &test_env.env;
    
    // Deploy contracts using framework helpers
    let (records_id, records_client) = test_env.register_medical_records();
    let (token_id, token_client) = test_env.register_token(&test_env.admin);
    
    // Access team members via fixtures
    let admin = &test_env.team.admin.address;
    let doctor = &test_env.team.doctors[0].address;
    let patient = &test_env.team.patients[0].address;
    
    // 1. Test Role Management
    records_client.initialize(admin);
    records_client.manage_user(admin, doctor, &Role::Doctor);
    records_client.manage_user(admin, patient, &Role::Patient);
    
    // 2. Test Token Integration (Minting)
    // SUT Token initialize is already called by the helper
    token_client.mint(admin, patient, &1000_i128);
    assert_eq!(token_client.balance_of(patient), 1000_i128);
    
    // 3. Test Time Control
    let start_time = env.ledger().timestamp();
    let one_day = 86400;
    test_env.jump_time(one_day);
    assert_eq!(env.ledger().timestamp(), start_time + one_day);
    
    // 4. Perform Integrated Operation
    let diagnosis = String::from_str(env, "Comprehensive Framework Test");
    let treatment = String::from_str(env, "Automated Verification");
    
    let record_id = records_client.add_record(
        doctor,
        patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![env, String::from_str(env, "framework_v2")],
        &String::from_str(env, "General"),
        &String::from_str(env, "Testing"),
        &String::from_str(env, "QmFrameworkTestHashV2"),
    );
    
    // 5. Verify State
    let record = records_client.get_record(patient, &record_id);
    assert_eq!(record.diagnosis, diagnosis);
    assert_eq!(record.timestamp, start_time + one_day);
    
    // 6. Assert Events
    // Check for record creation event topics
    test_env.assert_event_topics(&records_id, test_env.topics(&["EVENT", "REC_NEW"]));
    
    // Check for token mint event
    test_env.assert_event_topics(&token_id, test_env.topics(&["mint"]));
}

#[test]
fn test_framework_address_generation() {
    let test_env = IntegrationTestEnv::new();
    let addr1 = test_env.generate_address();
    let addr2 = test_env.generate_address();
    assert_ne!(addr1, addr2);
}

#[test]
fn test_framework_time_control_advanced() {
    let test_env = IntegrationTestEnv::new();
    let target_time = 2000000000;
    test_env.set_time(target_time);
    assert_eq!(test_env.env.ledger().timestamp(), target_time);
}

