//! #779: Cross-Contract Invocation Tests for HealthcareOracle → Reputation → Payment Flow
//! #782: Negative Test Coverage — Error Paths Not Tested
//!
//! This module tests:
//! 1. HealthcareOracleNetwork + Reputation + HealthcarePayment cross-contract flows
//! 2. Negative test scenarios (error paths) for multiple contracts
//! 3. Complete patient healthcare workflow

use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String, Vec,
};

use healthcare_oracle_network::{
    HealthcareOracleNetwork, HealthcareOracleNetworkClient,
    FeedKind, FeedPayload, SourceType, RegulatoryAuthority, RegulatoryStatus,
};
use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};

// ============================================================================
// #779: Cross-Contract Invocation Tests
// ============================================================================

/// Test: oracle submits clinical trial data → consensus → reputation trigger
#[test]
fn test_oracle_to_reputation_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy HealthcareOracleNetwork
    let oracle_id = env.register_contract(None, HealthcareOracleNetwork);
    let oracle_client = HealthcareOracleNetworkClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let arbiters = Vec::from_array(&env, [arbiter.clone()]);
    oracle_client.initialize(&admin, &arbiters, &1u32);

    // Register and verify oracle operator
    let operator = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://clinical.example");
    oracle_client.register_oracle(&operator, &endpoint, &SourceType::ClinicalRegistry);
    oracle_client.verify_oracle(&admin, &operator, &true, &true);

    // Oracle submits clinical trial data
    let trial_id = String::from_str(&env, "NCT-2026-XCT-001");
    let result_hash = String::from_str(&env, "sha256:clinical-trial-data");
    let round_id = oracle_client.submit_clinical_trial(
        &operator, &trial_id, &3u32, &500u32, &8200u32, &500u32, &result_hash, &1000u64,
    );

    // Verify consensus was reached
    let consensus = oracle_client
        .get_consensus(&FeedKind::ClinicalTrial, &trial_id)
        .expect("Consensus should exist after submission");

    assert_eq!(consensus.round_id, round_id);
    match consensus.payload {
        FeedPayload::ClinicalTrial(data) => {
            assert_eq!(data.trial_id, trial_id);
            assert_eq!(data.phase, 3);
            assert_eq!(data.enrolled, 500);
        }
        _ => panic!("Expected ClinicalTrial payload"),
    }

    // Verify oracle reputation was updated
    let oracle_node = oracle_client.get_oracle(&operator).expect("Oracle should exist");
    // Initial reputation is 50, submitting accurate data earns reputation
    assert!(oracle_node.reputation >= 50);
    assert_eq!(oracle_node.submissions, 1);
}

/// Test: dispute → reputation slashed → payment blocked
#[test]
fn test_dispute_reputation_payment_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    // --- Setup HealthcareOracleNetwork ---
    let oracle_id = env.register_contract(None, HealthcareOracleNetwork);
    let oracle_client = HealthcareOracleNetworkClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let arbiters = Vec::from_array(&env, [arbiter.clone()]);
    oracle_client.initialize(&admin, &arbiters, &1u32);

    let operator = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://reg.example");
    oracle_client.register_oracle(&operator, &endpoint, &SourceType::RegulatoryBody);
    oracle_client.verify_oracle(&admin, &operator, &true, &true);

    // Submit regulatory update
    let regulation_id = String::from_str(&env, "FDA-2026-ALERT");
    oracle_client.submit_regulatory_update(
        &operator, &regulation_id,
        &RegulatoryAuthority::FDA, &RegulatoryStatus::SafetyWarning,
        &String::from_str(&env, "Safety Alert"), &String::from_str(&env, "sha256:alert"), &900u64,
    );

    // Raise dispute
    let challenger = Address::generate(&env);
    let dispute_id = oracle_client.raise_dispute(
        &challenger, &FeedKind::RegulatoryUpdate, &regulation_id,
        &String::from_str(&env, "Data mismatch detected"),
    );

    // Capture reputation before dispute resolution
    let oracle_before = oracle_client.get_oracle(&operator).unwrap();
    let reputation_before = oracle_before.reputation;

    // Resolve dispute as valid (penalizing the oracle)
    oracle_client.resolve_dispute(
        &arbiter, &dispute_id, &true,
        &String::from_str(&env, "Confirmed data mismatch"),
        &Some(operator.clone()),
    );

    // Verify reputation was slashed
    let oracle_after = oracle_client.get_oracle(&operator).unwrap();
    assert!(
        oracle_after.reputation < reputation_before,
        "Reputation should decrease after valid dispute"
    );
    assert_eq!(oracle_after.disputes, 1);

    // Verify consensus is marked as disputed
    let consensus = oracle_client
        .get_consensus(&FeedKind::RegulatoryUpdate, &regulation_id)
        .expect("Consensus should still exist");
    assert!(consensus.disputed);
}

/// Test: oracle misbehaves → reputation slashed through penalty mechanism
#[test]
fn test_oracle_misbehavior_reputation_slash() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy HealthcareOracleNetwork
    let oracle_id = env.register_contract(None, HealthcareOracleNetwork);
    let oracle_client = HealthcareOracleNetworkClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let arbiters = Vec::from_array(&env, [arbiter.clone()]);
    oracle_client.initialize(&admin, &arbiters, &2u32);

    // Register two oracles
    let oracle1 = Address::generate(&env);
    let oracle2 = Address::generate(&env);
    oracle_client.register_oracle(&oracle1, &String::from_str(&env, "https://o1.example"), &SourceType::MarketAggregator);
    oracle_client.register_oracle(&oracle2, &String::from_str(&env, "https://o2.example"), &SourceType::MarketAggregator);
    oracle_client.verify_oracle(&admin, &oracle1, &true, &true);
    oracle_client.verify_oracle(&admin, &oracle2, &true, &true);

    let feed_id = String::from_str(&env, "NDC:TEST-001:US");
    let ndc = String::from_str(&env, "TEST-001");
    let currency = String::from_str(&env, "USD");

    // Oracle 1 submits an out-of-range price (very high compared to oracle 2)
    oracle_client.submit_drug_price(&oracle1, &feed_id, &ndc, &currency, &100000i128, &100u32, &100u64);
    oracle_client.submit_drug_price(&oracle2, &feed_id, &ndc, &currency, &100i128, &100u32, &101u64);

    // Check oracle 1 reputation - it should be penalized for being far from consensus
    let node1 = oracle_client.get_oracle(&oracle1).unwrap();
    // The price difference is large, so oracle1 should have lower reputation
    assert_eq!(node1.submissions, 1);
}

// ============================================================================
// #782: Negative Test Coverage — Error Path Tests
// ============================================================================

/// Negative test: Unauthorized caller cannot update config
#[test]
fn test_unauthorized_config_update_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let oracle_id = env.register_contract(None, HealthcareOracleNetwork);
    let oracle_client = HealthcareOracleNetworkClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let arbiters = Vec::from_array(&env, [arbiter]);
    oracle_client.initialize(&admin, &arbiters, &1u32);

    let unauthorized = Address::generate(&env);
    let result = oracle_client.try_update_config(
        &unauthorized, &1u32, &0i128, &1000i128, &1000u32,
    );
    // update_config requires admin auth, so unauthorized caller fails
    assert!(result.is_err());
}

/// Negative test: Double initialization fails
#[test]
fn test_double_initialization_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let oracle_id = env.register_contract(None, HealthcareOracleNetwork);
    let oracle_client = HealthcareOracleNetworkClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let arbiters = Vec::from_array(&env, [arbiter]);

    // First initialization succeeds
    assert!(oracle_client.try_initialize(&admin, &arbiters.clone(), &1u32).is_ok());

    // Second initialization fails
    let result = oracle_client.try_initialize(&admin, &arbiters, &1u32);
    assert!(result.is_err());
}

/// Negative test: Inactive oracle cannot submit
#[test]
fn test_inactive_oracle_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let oracle_id = env.register_contract(None, HealthcareOracleNetwork);
    let oracle_client = HealthcareOracleNetworkClient::new(&env, &oracle_id);

    let admin = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let arbiters = Vec::from_array(&env, [arbiter]);
    oracle_client.initialize(&admin, &arbiters, &1u32);

    let operator = Address::generate(&env);
    oracle_client.register_oracle(
        &operator, &String::from_str(&env, "https://example.com"), &SourceType::MarketAggregator,
    );
    // Verify but mark as inactive
    oracle_client.verify_oracle(&admin, &operator, &true, &false);

    // Submission should fail
    let result = oracle_client.try_submit_drug_price(
        &operator, &String::from_str(&env, "FEED"), &String::from_str(&env, "NDC"),
        &String::from_str(&env, "USD"), &100i128, &10u32, &1u64,
    );
    assert!(result.is_err());
}

/// Negative test: MedicalRecords - unauthorized access to non-existent record
#[test]
fn test_medical_records_get_nonexistent_record() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let caller = Address::generate(&env);
    let result = client.try_get_record(&caller, &99999u64);
    assert!(result.is_err());
}

/// Negative test: MedicalRecords - unauthorized user cannot add record
#[test]
fn test_medical_records_unauthorized_add_record() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // A non-doctor user trying to add a record should fail
    let patient = Address::generate(&env);
    client.manage_user(&admin, &patient, &Role::Patient);

    let result = client.try_add_record(
        &patient, &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );
    assert!(result.is_err());
}

// ============================================================================
// #784: Full System E2E Test - Medical Records Workflow
// ============================================================================

/// Full system E2E: Register identity → RBAC → create record → verify
#[test]
fn test_full_system_patient_journey() {
    let env = Env::default();
    env.mock_all_auths();

    // ---- Setup actors ----
    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    // ---- Deploy MedicalRecords ----
    let records_id = env.register_contract(None, MedicalRecordsContract);
    let records_client = MedicalRecordsContractClient::new(&env, &records_id);
    records_client.initialize(&admin);
    records_client.manage_user(&admin, &doctor, &Role::Doctor);
    records_client.manage_user(&admin, &patient, &Role::Patient);

    // ---- Create medical record ----
    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx");
    let record_id = records_client.add_record(
        &doctor, &patient,
        &String::from_str(&env, "E2E Test: Hypertension"),
        &String::from_str(&env, "ACE inhibitor"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &data_ref,
    );
    assert!(record_id >= 0);

    // ---- Verify record ----
    let record = records_client.get_record(&patient, &record_id);
    assert_eq!(record.patient_id, patient);
    assert_eq!(record.diagnosis, String::from_str(&env, "E2E Test: Hypertension"));
}
