#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    AlertLevel, AlertStatus, AnomalyDetectorContract, AnomalyDetectorContractClient, Error,
    HealthcarePatternType,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

// ==================== Helpers ====================

fn setup(env: &Env) -> (AnomalyDetectorContractClient<'_>, Address) {
    let id = env.register_contract(None, AnomalyDetectorContract);
    let client = AnomalyDetectorContractClient::new(env, &id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.initialize(&admin);
    (client, admin)
}

fn make_model_id(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// Register a 3-feature model with equal weights (5000 bps each) and threshold 5000 bps.
fn register_default_model(
    client: &AnomalyDetectorContractClient,
    env: &Env,
    admin: &Address,
    model_id: &BytesN<32>,
) {
    let weights = soroban_sdk::vec![env, 5_000u32, 5_000u32, 5_000u32];
    env.mock_all_auths();
    client.register_model(
        admin,
        model_id,
        &String::from_str(env, "test_model"),
        &3,
        &weights,
        &5_000,
    );
}

// ==================== Initialization ====================

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    assert!(!client.is_paused());
    assert_eq!(client.get_alert_count(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    env.mock_all_auths();
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

// ==================== Validator Management ====================

#[test]
fn test_add_and_remove_validator() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let validator = Address::generate(&env);

    env.mock_all_auths();
    assert!(!client.is_validator(&validator));

    client.add_validator(&admin, &validator);
    assert!(client.is_validator(&validator));

    client.remove_validator(&admin, &validator);
    assert!(!client.is_validator(&validator));
}

#[test]
fn test_non_admin_cannot_add_validator() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let rogue = Address::generate(&env);
    let validator = Address::generate(&env);

    env.mock_all_auths();
    let result = client.try_add_validator(&rogue, &validator);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

// ==================== Pause / Unpause ====================

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    env.mock_all_auths();
    client.pause(&admin);
    assert!(client.is_paused());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
fn test_paused_blocks_operations() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let model_id = make_model_id(&env, 1);
    let weights = soroban_sdk::vec![&env, 5_000u32];

    env.mock_all_auths();
    client.pause(&admin);

    let result = client.try_register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &1,
        &weights,
        &5_000,
    );
    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

// ==================== Model Registration ====================

#[test]
fn test_register_model() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 1);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 6_000u32, 4_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "access_model"),
        &2,
        &weights,
        &6_000,
    );

    let model = client.get_model(&model_id).unwrap();
    assert_eq!(model.feature_count, 2);
    assert_eq!(model.threshold_bps, 6_000);
    assert_eq!(model.version, 1);
    assert_eq!(model.total_inferences, 0);

    let stored_weights = client.get_model_weights(&model_id).unwrap();
    assert_eq!(stored_weights.get(0).unwrap(), 6_000u32);
    assert_eq!(stored_weights.get(1).unwrap(), 4_000u32);
}

#[test]
fn test_register_model_feature_count_mismatch_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 2);

    env.mock_all_auths();
    // 3 weights but feature_count = 2 → mismatch
    let weights = soroban_sdk::vec![&env, 5_000u32, 5_000u32, 5_000u32];
    let result = client.try_register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &2,
        &weights,
        &5_000,
    );
    assert_eq!(result, Err(Ok(Error::FeatureCountMismatch)));
}

#[test]
fn test_register_model_invalid_threshold_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 3);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32];
    let result = client.try_register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &1,
        &weights,
        &10_001, // > 10000 → invalid
    );
    assert_eq!(result, Err(Ok(Error::InvalidThreshold)));
}

// ==================== ML Inference ====================

#[test]
fn test_run_inference_normal() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 10);

    // Register model with threshold 6000
    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "test"),
        &2,
        &weights,
        &6_000,
    );

    let patient = Address::generate(&env);
    // Low feature values → score well below threshold
    let features = soroban_sdk::vec![&env, 1_000u32, 1_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
    ];

    env.mock_all_auths();
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    assert_eq!(result.anomaly_score, 1_000);
    assert!(!result.is_anomalous);
    assert_eq!(result.pattern_type, HealthcarePatternType::MlScored);
    assert_eq!(result.top_features.len(), 2);

    // Inference counter should be incremented
    let model = client.get_model(&model_id).unwrap();
    assert_eq!(model.total_inferences, 1);
}

#[test]
fn test_run_inference_anomaly() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 11);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "test"),
        &2,
        &weights,
        &5_000, // threshold = 5000
    );

    let patient = Address::generate(&env);
    // High feature values → score above threshold
    let features = soroban_sdk::vec![&env, 9_000u32, 8_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "access_rate"),
        String::from_str(&env, "risk_score"),
    ];

    env.mock_all_auths();
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    assert!(result.anomaly_score > 5_000);
    assert!(result.is_anomalous);
    assert!(result.confidence > 5_000);
    assert_eq!(result.alert_level, AlertLevel::Critical); // 8500 > 7500 → Critical
}

#[test]
fn test_run_inference_feature_mismatch_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 12);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "test"),
        &2,
        &weights,
        &5_000,
    );

    let patient = Address::generate(&env);
    // Only 1 feature for a 2-feature model
    let features = soroban_sdk::vec![&env, 8_000u32];
    let names = soroban_sdk::vec![&env, String::from_str(&env, "f1")];

    env.mock_all_auths();
    let result = client.try_run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    match result {
        Err(Ok(e)) => assert_eq!(e, Error::FeatureCountMismatch),
        _ => panic!("expected FeatureCountMismatch error"),
    }
}

#[test]
fn test_critical_alert_level() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 13);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 10_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "test"),
        &1,
        &weights,
        &3_000,
    );

    let patient = Address::generate(&env);
    let features = soroban_sdk::vec![&env, 10_000u32]; // max score
    let names = soroban_sdk::vec![&env, String::from_str(&env, "f1")];

    env.mock_all_auths();
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    assert_eq!(result.anomaly_score, 10_000);
    assert!(result.is_anomalous);
    assert_eq!(result.alert_level, AlertLevel::Critical);
}

// ==================== Healthcare Patterns: Prescription ====================

#[test]
fn test_prescription_anomaly_high_risk_ratio() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    // 10 drugs total, 8 are high-risk → 80% high-risk ratio → anomalous
    let result = client.detect_prescription_anomaly(
        &admin,
        &patient,
        &10, // drug_count
        &8,  // high_risk_count
        &2,  // unique_pharmacies
        &24, // 24-hour window
        &String::from_str(&env, "{}"),
    );

    assert!(result.is_anomalous);
    assert_eq!(
        result.pattern_type,
        HealthcarePatternType::PrescriptionAnomaly
    );
    assert!(result.anomaly_score > 5_000);
    assert_eq!(result.top_features.len(), 3);
}

#[test]
fn test_prescription_anomaly_normal() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    // 5 drugs, 0 high-risk, 1 pharmacy → normal
    let result = client.detect_prescription_anomaly(
        &admin,
        &patient,
        &5,  // drug_count
        &0,  // high_risk_count (0%)
        &1,  // unique_pharmacies
        &48, // time_window_hours
        &String::from_str(&env, "{}"),
    );

    assert!(!result.is_anomalous);
    assert_eq!(
        result.pattern_type,
        HealthcarePatternType::PrescriptionAnomaly
    );
}

#[test]
fn test_prescription_anomaly_many_pharmacies() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    // 4 pharmacies → pharmacy_score = 10000, pushes total over threshold
    let result = client.detect_prescription_anomaly(
        &admin,
        &patient,
        &8,  // drug_count
        &1,  // 12.5% high-risk (low)
        &4,  // 4 pharmacies → max pharmacy score
        &24, // time_window_hours
        &String::from_str(&env, "{}"),
    );

    assert!(result.is_anomalous);
    assert_eq!(
        result.pattern_type,
        HealthcarePatternType::PrescriptionAnomaly
    );
}

// ==================== Healthcare Patterns: Access ====================

#[test]
fn test_access_anomaly_after_hours() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    // 15 records accessed after hours → suspicious
    let result = client.detect_access_anomaly(
        &admin,
        &patient,
        &15,   // access_count
        &3600, // 1 hour window
        &true, // is_after_hours
        &3,    // distinct_record_types
        &String::from_str(&env, "{}"),
    );

    assert!(result.is_anomalous);
    assert_eq!(
        result.pattern_type,
        HealthcarePatternType::UnusualTimeAccess
    );
}

#[test]
fn test_access_anomaly_bulk_access() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    // 30 records accessed during business hours but bulk
    let result = client.detect_access_anomaly(
        &admin,
        &patient,
        &30,   // access_count > 20 → BulkRecordAccess
        &3600, // 1 hour
        &false,
        &6, // 6 distinct types → high diversity
        &String::from_str(&env, "{}"),
    );

    assert!(result.is_anomalous);
    assert_eq!(result.pattern_type, HealthcarePatternType::BulkRecordAccess);
}

#[test]
fn test_access_anomaly_rapid_sequential() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    // 6 records in 30 seconds → RapidSequentialAccess
    let result = client.detect_access_anomaly(
        &admin,
        &patient,
        &6,  // access_count > 5
        &30, // time_window_secs < 60
        &false,
        &2,
        &String::from_str(&env, "{}"),
    );

    assert_eq!(
        result.pattern_type,
        HealthcarePatternType::RapidSequentialAccess
    );
}

#[test]
fn test_access_anomaly_normal() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    // 3 records in 1 hour during business hours → normal
    let result = client.detect_access_anomaly(
        &admin,
        &patient,
        &3,
        &3600,
        &false,
        &1,
        &String::from_str(&env, "{}"),
    );

    assert!(!result.is_anomalous);
}

// ==================== Alert Lifecycle ====================

#[test]
fn test_alert_create_and_read() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 20);
    let patient = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();
    // Produce a high-score detection
    let features = soroban_sdk::vec![&env, 9_000u32, 9_000u32, 9_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    let alert_id = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &result,
        &String::from_str(&env, r#"{"context":"test"}"#),
    );

    assert_eq!(alert_id, 1);
    assert_eq!(client.get_alert_count(), 1);

    let alert = client.get_alert(&alert_id).unwrap();
    assert_eq!(alert.patient, patient);
    assert_eq!(alert.status, AlertStatus::Active);
    assert_eq!(alert.anomaly_score, 9_000);
    assert_eq!(alert.alert_level, AlertLevel::Critical);
}

#[test]
fn test_alert_lifecycle_acknowledge_then_resolve() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 21);
    let patient = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();
    let features = soroban_sdk::vec![&env, 8_000u32, 8_000u32, 8_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    let alert_id = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &result,
        &String::from_str(&env, "{}"),
    );

    // Acknowledge
    client.acknowledge_alert(&admin, &alert_id);
    assert_eq!(
        client.get_alert(&alert_id).unwrap().status,
        AlertStatus::Acknowledged
    );

    // Resolve
    client.resolve_alert(
        &admin,
        &alert_id,
        &String::from_str(&env, "Investigated: confirmed breach, contained"),
    );
    assert_eq!(
        client.get_alert(&alert_id).unwrap().status,
        AlertStatus::Resolved
    );

    // Patient active_alerts decremented
    let profile = client.get_patient_profile(&patient).unwrap();
    assert_eq!(profile.active_alerts, 0);
}

#[test]
fn test_alert_mark_false_positive() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 22);
    let patient = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();
    let features = soroban_sdk::vec![&env, 8_000u32, 8_000u32, 8_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    let alert_id = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &result,
        &String::from_str(&env, "{}"),
    );

    client.mark_false_positive(&admin, &alert_id);

    let alert = client.get_alert(&alert_id).unwrap();
    assert_eq!(alert.status, AlertStatus::FalsePositive);

    let profile = client.get_patient_profile(&patient).unwrap();
    assert_eq!(profile.false_positive_count, 1);
}

#[test]
fn test_double_resolve_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 23);
    let patient = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();
    let features = soroban_sdk::vec![&env, 8_000u32, 8_000u32, 8_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    let alert_id = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &result,
        &String::from_str(&env, "{}"),
    );

    client.resolve_alert(&admin, &alert_id, &String::from_str(&env, "resolved"));

    // Second resolve should fail
    let r = client.try_resolve_alert(&admin, &alert_id, &String::from_str(&env, "again"));
    assert_eq!(r, Err(Ok(Error::AlertAlreadyResolved)));
}

#[test]
fn test_acknowledge_resolved_alert_fails() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 24);
    let patient = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();
    let features = soroban_sdk::vec![&env, 8_000u32, 8_000u32, 8_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    let alert_id = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &result,
        &String::from_str(&env, "{}"),
    );

    client.resolve_alert(&admin, &alert_id, &String::from_str(&env, "done"));

    let r = client.try_acknowledge_alert(&admin, &alert_id);
    assert_eq!(r, Err(Ok(Error::AlertAlreadyResolved)));
}

// ==================== Adaptive Learning ====================

#[test]
fn test_adaptive_learning_false_positive_raises_threshold() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 30);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "adaptive_model"),
        &2,
        &weights,
        &5_000, // initial threshold
    );

    // Create an alert to reference
    let features = soroban_sdk::vec![&env, 8_000u32, 8_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
    ];
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    let alert_id = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &result,
        &String::from_str(&env, "{}"),
    );

    let initial_threshold = client.get_model(&model_id).unwrap().threshold_bps;

    // False positive feedback → threshold rises
    client.submit_feedback(&admin, &alert_id, &model_id, &false);

    let updated_threshold = client.get_model(&model_id).unwrap().threshold_bps;
    assert!(updated_threshold > initial_threshold);
    assert_eq!(updated_threshold, initial_threshold + 50); // LEARNING_RATE = 50 bps
}

#[test]
fn test_adaptive_learning_confirmed_lowers_threshold() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 31);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &1,
        &weights,
        &5_000,
    );

    let features = soroban_sdk::vec![&env, 8_000u32];
    let names = soroban_sdk::vec![&env, String::from_str(&env, "f1")];
    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    let alert_id = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &result,
        &String::from_str(&env, "{}"),
    );

    let initial_threshold = client.get_model(&model_id).unwrap().threshold_bps;

    // Confirmed feedback → threshold lowers
    client.submit_feedback(&admin, &alert_id, &model_id, &true);

    let updated = client.get_model(&model_id).unwrap().threshold_bps;
    assert!(updated < initial_threshold);
    assert_eq!(updated, initial_threshold - 50);

    // confirmed_anomalies counter incremented
    assert_eq!(client.get_model(&model_id).unwrap().confirmed_anomalies, 1);
}

#[test]
fn test_multiple_feedback_updates_converge() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 32);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &1,
        &weights,
        &5_000,
    );

    // Create 3 alerts and submit mixed feedback
    for i in 0u64..3 {
        let features = soroban_sdk::vec![&env, 8_000u32];
        let names = soroban_sdk::vec![&env, String::from_str(&env, "f1")];
        let result = client.run_inference(
            &admin,
            &patient,
            &model_id,
            &features,
            &names,
            &String::from_str(&env, "{}"),
        );
        let alert_id = client.create_alert(
            &admin,
            &patient,
            &model_id,
            &result,
            &String::from_str(&env, "{}"),
        );
        // Alternate: confirm, false-positive, confirm
        let confirmed = i != 1;
        client.submit_feedback(&admin, &alert_id, &model_id, &confirmed);
    }

    let m = client.get_model(&model_id).unwrap();
    assert_eq!(m.confirmed_anomalies, 2);
    assert_eq!(m.false_positives, 1);
    // net threshold: 5000 - 50 + 50 - 50 = 4950
    assert_eq!(m.threshold_bps, 4_950);
}

#[test]
fn test_update_model_weight_manually() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 33);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32, 3_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &2,
        &weights,
        &5_000,
    );

    // Increase feature 0 weight by 500
    client.update_model_weight(&admin, &model_id, &0, &500, &true);
    let w = client.get_model_weights(&model_id).unwrap();
    assert_eq!(w.get(0).unwrap(), 5_500u32);
    assert_eq!(w.get(1).unwrap(), 3_000u32); // unchanged

    // Decrease feature 1 weight by 1000
    client.update_model_weight(&admin, &model_id, &1, &1_000, &false);
    let w2 = client.get_model_weights(&model_id).unwrap();
    assert_eq!(w2.get(1).unwrap(), 2_000u32);

    // Model version bumped twice
    assert_eq!(client.get_model(&model_id).unwrap().version, 3); // 1 + 2 updates
}

// ==================== Federated Learning ====================

#[test]
fn test_submit_federated_update() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let participant = Address::generate(&env);
    let update_hash = BytesN::from_array(&env, &[0xabu8; 32]);

    env.mock_all_auths();
    client.submit_federated_update(&participant, &1, &update_hash, &100);

    let stored = client.get_federated_update(&1, &participant).unwrap();
    assert_eq!(stored.round_id, 1);
    assert_eq!(stored.participant, participant);
    assert_eq!(stored.num_samples, 100);
    assert_eq!(stored.update_hash, update_hash);
}

#[test]
fn test_duplicate_federated_update_fails() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let participant = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0xbbu8; 32]);

    env.mock_all_auths();
    client.submit_federated_update(&participant, &1, &hash, &50);

    let result = client.try_submit_federated_update(&participant, &1, &hash, &50);
    assert_eq!(result, Err(Ok(Error::DuplicateFederatedUpdate)));
}

#[test]
fn test_multiple_participants_same_round() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    let p3 = Address::generate(&env);
    let h1 = BytesN::from_array(&env, &[0x11u8; 32]);
    let h2 = BytesN::from_array(&env, &[0x22u8; 32]);
    let h3 = BytesN::from_array(&env, &[0x33u8; 32]);

    env.mock_all_auths();
    client.submit_federated_update(&p1, &5, &h1, &200);
    client.submit_federated_update(&p2, &5, &h2, &300);
    client.submit_federated_update(&p3, &5, &h3, &150);

    assert_eq!(
        client.get_federated_update(&5, &p1).unwrap().num_samples,
        200
    );
    assert_eq!(
        client.get_federated_update(&5, &p2).unwrap().num_samples,
        300
    );
    assert_eq!(
        client.get_federated_update(&5, &p3).unwrap().num_samples,
        150
    );
}

// ==================== Patient Risk Profile ====================

#[test]
fn test_patient_risk_profile_updates() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 40);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &1,
        &weights,
        &3_000, // low threshold → everything is anomalous
    );

    // No profile initially
    assert!(client.get_patient_profile(&patient).is_none());

    let features = soroban_sdk::vec![&env, 8_000u32];
    let names = soroban_sdk::vec![&env, String::from_str(&env, "f1")];

    // First anomaly detection updates profile
    client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    let profile = client.get_patient_profile(&patient).unwrap();
    assert!(profile.rolling_risk_score > 0);
    assert_eq!(profile.total_alerts, 1);
}

#[test]
fn test_patient_risk_score_ema() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 41);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "m"),
        &1,
        &weights,
        &2_000, // very low threshold
    );

    let features_high = soroban_sdk::vec![&env, 10_000u32];
    let names = soroban_sdk::vec![&env, String::from_str(&env, "f1")];

    // 3 high-score inferences → rolling score increases toward 10000
    for _ in 0..3 {
        client.run_inference(
            &admin,
            &patient,
            &model_id,
            &features_high,
            &names,
            &String::from_str(&env, "{}"),
        );
    }

    let profile = client.get_patient_profile(&patient).unwrap();
    assert!(profile.rolling_risk_score > 5_000); // converging toward 10000
    assert_eq!(profile.total_alerts, 3);
}

// ==================== Explainability ====================

#[test]
fn test_inference_feature_contributions_present() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 50);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    let weights = soroban_sdk::vec![&env, 8_000u32, 2_000u32, 5_000u32];
    client.register_model(
        &admin,
        &model_id,
        &String::from_str(&env, "xai_model"),
        &3,
        &weights,
        &5_000,
    );

    let features = soroban_sdk::vec![&env, 9_000u32, 1_000u32, 6_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "access_rate"),
        String::from_str(&env, "time_of_day"),
        String::from_str(&env, "record_count"),
    ];

    let result = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    // 3 feature contributions returned for audit compliance
    assert_eq!(result.top_features.len(), 3);

    // Highest-weight feature (access_rate, w=8000) should have largest contribution
    let fc0 = result.top_features.get(0).unwrap();
    assert_eq!(fc0.feature_index, 0);
    assert_eq!(fc0.weight, 8_000u32);
    assert_eq!(fc0.feature_value, 9_000u32);
    // contribution = 9000 * 8000 / 10000 = 7200
    assert_eq!(fc0.contribution, 7_200u32);
}

#[test]
fn test_prescription_anomaly_feature_contributions() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);

    env.mock_all_auths();
    let result = client.detect_prescription_anomaly(
        &admin,
        &patient,
        &10,
        &5, // 50% high-risk
        &3,
        &24,
        &String::from_str(&env, "{}"),
    );

    assert_eq!(result.top_features.len(), 3);
    let contrib0 = result.top_features.get(0).unwrap();
    assert_eq!(
        contrib0.feature_name,
        String::from_str(&env, "high_risk_ratio")
    );
    assert_eq!(contrib0.feature_value, 5_000u32); // 50% = 5000 bps
}

// ==================== Multiple Alerts Uniqueness ====================

#[test]
fn test_multiple_alerts_unique_ids() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 60);
    let patient = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();
    let features = soroban_sdk::vec![&env, 9_000u32, 9_000u32, 9_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];

    let r1 = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    let r2 = client.run_inference(
        &admin,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    let id1 = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &r1,
        &String::from_str(&env, "{}"),
    );
    let id2 = client.create_alert(
        &admin,
        &patient,
        &model_id,
        &r2,
        &String::from_str(&env, "{}"),
    );

    assert_ne!(id1, id2);
    assert_eq!(client.get_alert_count(), 2);

    // Both independently retrievable
    assert!(client.get_alert(&id1).is_some());
    assert!(client.get_alert(&id2).is_some());
}

// ==================== Validator as Authorized Caller ====================

#[test]
fn test_validator_can_run_inference() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 70);
    let patient = Address::generate(&env);
    let validator = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();
    client.add_validator(&admin, &validator);

    let features = soroban_sdk::vec![&env, 3_000u32, 3_000u32, 3_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];

    // Validator (not admin) can run inference
    let result = client.run_inference(
        &validator,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );

    assert_eq!(result.anomaly_score, 3_000);
}

#[test]
fn test_unauthorized_caller_blocked() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let model_id = make_model_id(&env, 71);
    let patient = Address::generate(&env);
    let rogue = Address::generate(&env);
    register_default_model(&client, &env, &admin, &model_id);

    env.mock_all_auths();

    let features = soroban_sdk::vec![&env, 5_000u32, 5_000u32, 5_000u32];
    let names = soroban_sdk::vec![
        &env,
        String::from_str(&env, "f1"),
        String::from_str(&env, "f2"),
        String::from_str(&env, "f3"),
    ];

    let result = client.try_run_inference(
        &rogue,
        &patient,
        &model_id,
        &features,
        &names,
        &String::from_str(&env, "{}"),
    );
    match result {
        Err(Ok(e)) => assert_eq!(e, Error::NotAuthorized),
        _ => panic!("expected NotAuthorized error"),
    }
}
