#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke};
use soroban_sdk::{log, Address, BytesN, Env, String, Vec, symbol_short};

use medical_records_contract::medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};
use medical_records_contract::events;

fn create_contract(env: &Env) -> (MedicalRecordsContractClient, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordsContract);

    let client = MedicalRecordsContractClient::new(env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_user_management_events() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);

    // Test user creation event (from initialize)
    let events = env.events().all();
    assert!(events.len() >= 1); // At least the initialization event

    // Check for user created event
    let user_created_events: Vec<_> = events.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("USER_CREATED"))
        .collect();
    assert_eq!(user_created_events.len(), 1);

    // Test user role update event
    client.manage_user(&admin, &doctor, &Role::Doctor);

    let events_after = env.events().all();
    let role_update_events: Vec<_> = events_after.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("USER_ROLE_UPD"))
        .collect();
    assert_eq!(role_update_events.len(), 1);

    // Test user deactivation event
    client.deactivate_user(&admin, &doctor);

    let events_final = env.events().all();
    let deactivation_events: Vec<_> = events_final.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("USER_DEACT"))
        .collect();
    assert_eq!(deactivation_events.len(), 1);
}

#[test]
fn test_record_creation_events() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    let initial_event_count = env.events().all().len();

    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );

    let events_after = env.events().all();
    assert!(events_after.len() > initial_event_count);

    // Check for record created event
    let record_events: Vec<_> = events_after.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("RECORD_CREATED"))
        .collect();
    assert_eq!(record_events.len(), 1);

    // Check for AI trigger event
    let ai_events: Vec<_> = events_after.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("AI_TRIGGER"))
        .collect();
    assert_eq!(ai_events.len(), 1);

    // Check for metric update event
    let metric_events: Vec<_> = events_after.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("METRIC_UPD"))
        .collect();
    assert_eq!(metric_events.len(), 1);
}

#[test]
fn test_record_access_events() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );

    let initial_event_count = env.events().all().len();

    // Access the record as patient
    let _record = client.get_record(&patient, &record_id);

    let events_after = env.events().all();
    let access_events: Vec<_> = events_after.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("RECORD_ACCESS"))
        .collect();
    assert_eq!(access_events.len(), 1);
}

#[test]
fn test_administrative_events() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);

    // Test contract pause event
    client.pause(&admin);

    let events_after_pause = env.events().all();
    let pause_events: Vec<_> = events_after_pause.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("CONTRACT_PAUSE"))
        .collect();
    assert_eq!(pause_events.len(), 1);

    // Test contract unpause event
    client.unpause(&admin);

    let events_after_unpause = env.events().all();
    let unpause_events: Vec<_> = events_after_unpause.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("CONTRACT_UNPAUSE"))
        .collect();
    assert_eq!(unpause_events.len(), 1);
}

#[test]
fn test_recovery_events() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin1) = create_contract(&env);
    let admin2 = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);

    client.manage_user(&admin1, &admin2, &Role::Admin);

    // Test recovery proposal event
    let proposal_id = client.propose_recovery(&admin1, &token, &recipient, &100i128);

    let events_after_proposal = env.events().all();
    let proposal_events: Vec<_> = events_after_proposal.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("RECOVERY_PROP"))
        .collect();
    assert_eq!(proposal_events.len(), 1);

    // Test recovery approval event
    client.approve_recovery(&admin2, &proposal_id);

    let events_after_approval = env.events().all();
    let approval_events: Vec<_> = events_after_approval.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("RECOVERY_APPR"))
        .collect();
    assert_eq!(approval_events.len(), 1);

    // Advance time and test recovery execution event
    env.ledger().with_mut(|l| {
        l.timestamp = env.ledger().timestamp() + 86_400 + 1;
    });

    client.execute_recovery(&admin1, &proposal_id);

    let events_final = env.events().all();
    let execution_events: Vec<_> = events_final.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("RECOVERY_EXEC"))
        .collect();
    assert_eq!(execution_events.len(), 1);
}

#[test]
fn test_ai_events() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let ai_coordinator = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Set AI config
    client.set_ai_config(&admin, &ai_coordinator, &100u32, &2u32);

    let events_after_config = env.events().all();
    let config_events: Vec<_> = events_after_config.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("AI_CONFIG_UPD"))
        .collect();
    assert_eq!(config_events.len(), 1);

    // Create a record
    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );

    // Submit anomaly score
    let model_id = BytesN::from_array(&env, &[1; 32]);
    let explanation_ref = String::from_str(&env, "ipfs://report");
    let explanation_summary = String::from_str(&env, "Anomaly detected");
    let model_version = String::from_str(&env, "v1.0.0");
    let feature_importance = vec![&env, (String::from_str(&env, "feature"), 5000u32)];

    client.submit_anomaly_score(
        &ai_coordinator,
        &record_id,
        &model_id,
        &7500u32,
        &explanation_ref,
        &explanation_summary,
        &model_version,
        &feature_importance,
    );

    let events_after_anomaly = env.events().all();
    let anomaly_events: Vec<_> = events_after_anomaly.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("ANOMALY_SCORE"))
        .collect();
    assert_eq!(anomaly_events.len(), 1);

    // Submit risk score
    client.submit_risk_score(
        &ai_coordinator,
        &patient,
        &model_id,
        &8000u32,
        &explanation_ref,
        &explanation_summary,
        &model_version,
        &feature_importance,
    );

    let events_final = env.events().all();
    let risk_events: Vec<_> = events_final.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("RISK_SCORE"))
        .collect();
    assert_eq!(risk_events.len(), 1);
}

#[test]
fn test_health_check_events() {
    let env = Env::default();

    let (client, _admin) = create_contract(&env);

    let initial_event_count = env.events().all().len();

    // Call health check
    let _result = client.health_check();

    let events_after = env.events().all();
    let health_events: Vec<_> = events_after.iter()
        .filter(|e| e.topics.len() >= 2 && e.topics[1] == symbol_short!("HEALTH_CHECK"))
        .collect();
    assert_eq!(health_events.len(), 1);
}

#[test]
fn test_event_filtering() {
    let env = Env::default();

    // Test event filtering functionality (this would be expanded with actual stored events)
    let filter = events::EventFilter {
        event_types: Some(vec![&env, events::EventType::RecordCreated]),
        categories: None,
        user_id: None,
        start_time: Some(1000),
        end_time: Some(2000),
        limit: Some(10),
    };

    // Since we don't have persistent event storage in this test,
    // we test the filtering logic with empty event set
    let events = Vec::<events::BaseEvent>::new(&env);
    let filtered = events::filter_events(&events, &filter);
    assert_eq!(filtered.len(), 0);
}

#[test]
fn test_event_aggregation() {
    let env = Env::default();

    // Test event aggregation with empty event set
    let events = Vec::<events::BaseEvent>::new(&env);
    let stats = events::aggregate_events(&events);

    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.events_by_type.len(), 0);
    assert_eq!(stats.events_by_category.len(), 0);
    assert_eq!(stats.events_by_user.len(), 0);
}

#[test]
fn test_monitoring_dashboard_generation() {
    let env = Env::default();

    // Test dashboard generation with empty events
    let events = Vec::<events::BaseEvent>::new(&env);
    let dashboard = events::create_monitoring_dashboard(&env, &events, 10);

    assert_eq!(dashboard.stats.total_events, 0);
    assert_eq!(dashboard.recent_events.len(), 0);
    assert_eq!(dashboard.health_status, String::from_str(&env, "unknown"));
}

#[test]
fn test_gas_efficiency() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Reset budget to measure gas usage
    env.budget().reset_unlimited();

    // Perform operation that emits events
    let _record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );

    // Check that gas usage is reasonable
    let cpu_cost = env.budget().cpu_instruction_cost();
    assert!(cpu_cost < 200_000); // Should be well under 200k instructions

    std::println!("Gas used for record creation with events: {}", cpu_cost);
}

#[test]
fn test_event_data_structure() {
    let env = Env::default();

    // Test that event data structures are properly defined
    let user_event = events::EventData::UserEvent {
        target_user: Address::generate(&env),
        role: Some(String::from_str(&env, "Doctor")),
        previous_role: None,
        did_reference: None,
    };

    let record_event = events::EventData::RecordEvent {
        record_id: 123,
        patient_id: Address::generate(&env),
        doctor_id: Some(Address::generate(&env)),
        is_confidential: false,
        category: String::from_str(&env, "Modern"),
        tags: vec![&env, String::from_str(&env, "cardiology")],
    };

    // Test that we can create events with different data types
    match user_event {
        events::EventData::UserEvent { target_user, .. } => {
            assert!(!target_user.is_zero());
        }
        _ => panic!("Wrong event type"),
    }

    match record_event {
        events::EventData::RecordEvent { record_id, tags, .. } => {
            assert_eq!(record_id, 123);
            assert_eq!(tags.len(), 1);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_event_metadata_completeness() {
    let env = Env::default();

    // Test that event metadata includes all required fields
    let metadata = events::EventMetadata {
        event_type: events::EventType::RecordCreated,
        category: events::OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: Address::generate(&env),
        session_id: Some(BytesN::from_array(&env, &[1; 32])),
        ipfs_ref: Some(String::from_str(&env, "ipfs://test")),
        gas_used: Some(5000),
        block_height: env.ledger().sequence(),
    };

    assert_eq!(metadata.event_type, events::EventType::RecordCreated);
    assert_eq!(metadata.category, events::OperationCategory::RecordOperations);
    assert!(metadata.timestamp > 0);
    assert!(!metadata.user_id.is_zero());
    assert!(metadata.session_id.is_some());
    assert!(metadata.ipfs_ref.is_some());
    assert_eq!(metadata.gas_used, Some(5000));
    assert!(metadata.block_height >= 0);
}