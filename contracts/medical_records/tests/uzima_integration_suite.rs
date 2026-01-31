#![cfg(test)]
mod common;
use common::setup_uzima;
use medical_records::{Error, Role};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, String,
};

#[test]
fn test_workflow_and_rbac() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    let diagnosis = String::from_str(&env, "Malaria Test Positive");
    let treatment = String::from_str(&env, "Artemether");
    let category = String::from_str(&env, "Modern");
    let data_ref = String::from_str(&env, "ipfs://cid1234567890");

    let record_id = t.client.add_record(
        &t.doctor,
        &t.patient,
        &diagnosis,
        &treatment,
        &false,
        &vec![&env, String::from_str(&env, "Urgent")],
        &category,
        &String::from_str(&env, "Inpatient"),
        &data_ref,
    );

    let record = t.client.get_record(&t.patient, &record_id).unwrap();
    assert_eq!(record.diagnosis, diagnosis);
}

#[test]
fn test_emergency_pause_and_resume() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);

    t.client.pause(&t.admin1);

    let result = t.client.try_add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "D"),
        &String::from_str(&env, "T"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://cid123456"),
    );
    assert_eq!(result, Err(Ok(Error::ContractPaused)));

    t.client.unpause(&t.admin1);
    let role = t.client.get_user_role(&t.doctor);
    assert!(matches!(role, Role::Doctor));
}

#[test]
fn test_recovery_proposal_full_flow() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    let safe_address = soroban_sdk::Address::generate(&env);
    let token_contract = soroban_sdk::Address::generate(&env);
    let proposal_id = t
        .client
        .propose_recovery(&t.admin1, &token_contract, &safe_address, &100);

    t.client.approve_recovery(&t.admin2, &proposal_id);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 86_401);

    let result = t.client.execute_recovery(&t.admin1, &proposal_id);
    // FIX: Standard client calls return the value directly, not Result<T, E>
    // FIXED: Removed .unwrap() because result is already a bool
    assert!(result);
}

#[test]
fn test_performance_and_pagination() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);

    for _ in 0..20 {
        t.client.add_record(
            &t.doctor,
            &t.patient,
            &String::from_str(&env, "Diag"),
            &String::from_str(&env, "Treat"),
            &false,
            &vec![&env],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "T"),
            &String::from_str(&env, "ipfs://cid123456"),
        );
    }

    let history = t.client.get_history(&t.patient, &t.patient, &0, &10);
    assert_eq!(history.len(), 10);
}
// Add this to the existing uzima_integration_suite.rs

#[test]
fn test_did_and_identity_management() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let patient_did = String::from_str(&env, "did:stellar:GABC...");
    let registry_addr = soroban_sdk::Address::generate(&env);

    // 1. Admin sets up the Identity Registry address (Coverage: set_identity_registry)
    t.client.set_identity_registry(&t.admin1, &registry_addr);
    assert_eq!(t.client.get_identity_registry(), Some(registry_addr));

    // 2. Admin sets a required Auth Level (Coverage: set_did_auth_level)
    t.client
        .set_did_auth_level(&t.admin1, &medical_records::DIDAuthLevel::Basic);
    assert!(matches!(
        t.client.get_did_auth_level(),
        medical_records::DIDAuthLevel::Basic
    ));

    // 3. User links their own DID (Coverage: link_did_to_user)
    t.client
        .link_did_to_user(&t.patient, &t.patient, &patient_did);
    assert_eq!(t.client.get_user_did(&t.patient), Some(patient_did));
}

#[test]
fn test_ai_config_and_risk_submission() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let ai_coord = soroban_sdk::Address::generate(&env);
    let model_id = soroban_sdk::BytesN::from_array(&env, &[1; 32]);
    let features = vec![&env, (String::from_str(&env, "age"), 3000u32)];

    // 1. Admin sets AI Config (Coverage: set_ai_config)
    t.client.set_ai_config(&t.admin1, &ai_coord, &1000, &5);
    let config = t.client.get_ai_config().unwrap();
    assert_eq!(config.ai_coordinator, ai_coord);

    // 2. AI Coordinator submits a risk score (Coverage: submit_risk_score)
    let score = 8500u32;
    t.client.submit_risk_score(
        &ai_coord,
        &t.patient,
        &model_id,
        &score,
        &String::from_str(&env, "ipfs://risk_report"),
        &String::from_str(&env, "High risk category"),
        &String::from_str(&env, "v2.1"),
        &features,
    );

    // 3. Patient views their own score (Coverage: get_latest_risk_score)
    let risk_insight = t
        .client
        .get_latest_risk_score(&t.patient, &t.patient)
        .unwrap();
    assert_eq!(risk_insight.score_bps, score);

    // 4. Test unauthorized risk access (Negative Test)
    let unauthorized = soroban_sdk::Address::generate(&env);
    let result = t
        .client
        .try_get_latest_risk_score(&unauthorized, &t.patient);
    assert!(result.is_err());
}

#[test]
fn test_emergency_access_and_logging() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);
    let emergency_provider = soroban_sdk::Address::generate(&env);

    // 1. Patient grants access for 1 hour (Coverage: grant_emergency_access)
    let duration = 3600u64;
    t.client.grant_emergency_access(
        &t.patient,
        &emergency_provider,
        &duration,
        &vec![&env, 1u64, 2u64], // Scope specific records
    );

    // 2. Record access with emergency provider (Coverage: has_emergency_access)
    let has_access = t
        .client
        .has_emergency_access(&emergency_provider, &t.patient, &1);
    assert!(has_access);

    // 3. Patient revokes access (Coverage: revoke_emergency_access)
    t.client
        .revoke_emergency_access(&t.patient, &emergency_provider);

    // 4. Test access log (Coverage: log_access, get_patient_access_logs)
    // We log access in the get_record_with_did, so let's call that
    // First, let a doctor create a record so we have a log entry
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "Diag"),
        &String::from_str(&env, "Treat"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://cid1234567890"),
    );

    // Call get_record_with_did to force log_access
    let _ = t
        .client
        .try_get_record_with_did(&t.admin1, &1, &String::from_str(&env, "Audit"));

    // Check patient's log has at least 1 entry
    let logs = t
        .client
        .get_patient_access_logs(&t.admin1, &t.patient, &0, &10);
    assert!(!logs.is_empty());
}

#[test]
fn test_cross_chain_workflow() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    // Create a record first
    t.client.add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "Diag"),
        &String::from_str(&env, "Treat"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://cid1234567890"),
    );

    // 1. Admin sets cross-chain contracts (Coverage: set_cross_chain_contracts, set_cross_chain_enabled)
    let bridge = soroban_sdk::Address::generate(&env);
    let identity = soroban_sdk::Address::generate(&env);
    let access = soroban_sdk::Address::generate(&env);
    t.client
        .set_cross_chain_contracts(&t.admin1, &bridge, &identity, &access);
    t.client.set_cross_chain_enabled(&t.admin1, &true);

    // 2. Patient registers cross-chain reference (Coverage: register_cross_chain_ref)
    let external_hash = soroban_sdk::BytesN::from_array(&env, &[2; 32]);
    let chain_id = medical_records::ChainId::Ethereum; // Assuming Ethereum is supported
    t.client
        .register_cross_chain_ref(&t.patient, &1, &chain_id, &external_hash);

    // 3. Admin updates sync status (Coverage: update_cross_chain_sync)
    let new_hash = soroban_sdk::BytesN::from_array(&env, &[3; 32]);
    t.client
        .update_cross_chain_sync(&t.admin1, &1, &chain_id, &new_hash);

    // 4. Verify cross-chain reference exists (Coverage: get_cross_chain_ref)
    let ref_entry = t.client.get_cross_chain_ref(&1, &chain_id).unwrap();
    assert_eq!(ref_entry.external_record_hash, new_hash);

    // 5. Test cross-chain record access (Requires bridge auth)
    let result = t.client.try_get_record_cross_chain(
        &bridge,
        &1,
        &chain_id,
        &String::from_str(&env, "0xaccess"),
    );
    assert!(result.is_ok());
}
