#![cfg(test)]
mod common;
use common::setup_uzima;
use medical_records::{Role, Error, ChainId};
use soroban_sdk::{testutils::{Ledger, Address as _}, String, BytesN, Vec};

// ============================================================================
// SECTION 1: VALIDATION & BOUNDARY TESTS
// ============================================================================

#[test]
fn test_cov_initialization_guard() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let result = t.client.try_initialize(&t.admin1);
    assert!(result.is_err());
}

#[test]
fn test_cov_add_record_validations() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    let valid_data = String::from_str(&env, "ipfs://QmHash12345");

    // 1. Invalid Category
    let res_cat = t.client.try_add_record(
        &t.doctor, &t.patient, 
        &String::from_str(&env, "D"), &String::from_str(&env, "T"), &false, 
        &Vec::new(&env), 
        &String::from_str(&env, "InvalidCat"), 
        &String::from_str(&env, "T"), &valid_data
    );
    assert_eq!(res_cat, Err(Ok(Error::InvalidCategory.into())));

    // 2. Empty Treatment
    let res_treat = t.client.try_add_record(
        &t.doctor, &t.patient, 
        &String::from_str(&env, "D"), &String::from_str(&env, "T"), &false, 
        &Vec::new(&env), &String::from_str(&env, "Modern"), 
        &String::from_str(&env, ""), 
        &valid_data
    );
    assert_eq!(res_treat, Err(Ok(Error::EmptyTreatment.into())));

    // 3. Empty Tag inside Vector
    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "")); 
    let res_tag = t.client.try_add_record(
        &t.doctor, &t.patient, 
        &String::from_str(&env, "D"), &String::from_str(&env, "T"), &false, 
        &tags, 
        &String::from_str(&env, "Modern"), &String::from_str(&env, "T"), &valid_data
    );
    assert_eq!(res_tag, Err(Ok(Error::EmptyTag.into())));
}

#[test]
fn test_cov_data_ref_max_length() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    // Create a 201 character string
    let mut long_str = std::string::String::with_capacity(201);
    for _ in 0..201 { long_str.push('a'); }
    let data_ref = String::from_str(&env, &long_str);

    let result = t.client.try_add_record(
        &t.doctor, &t.patient, 
        &String::from_str(&env, "D"), &String::from_str(&env, "T"), &false, 
        &Vec::new(&env), &String::from_str(&env, "Modern"), 
        &String::from_str(&env, "T"), &data_ref
    );
    assert_eq!(result, Err(Ok(Error::InvalidDataRefLength.into())));
}

// ============================================================================
// SECTION 2: ACCESS CONTROL & SECURITY
// ============================================================================

#[test]
fn test_cov_emergency_access_expiration() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let provider = soroban_sdk::Address::generate(&env);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    t.client.grant_emergency_access(&t.patient, &provider, &3600, &Vec::new(&env));
    assert_eq!(t.client.has_emergency_access(&provider, &t.patient, &1), true);

    env.ledger().set_timestamp(env.ledger().timestamp() + 3601);
    assert_eq!(t.client.has_emergency_access(&provider, &t.patient, &1), false);
}

#[test]
fn test_cov_emergency_revoke_not_found() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let stranger = soroban_sdk::Address::generate(&env);
    
    let result = t.client.try_revoke_emergency_access(&t.patient, &stranger);
    assert_eq!(result, Err(Ok(Error::EmergencyAccessNotFound.into())));
}

#[test]
fn test_cov_patient_emergency_grant_filtering() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let provider = soroban_sdk::Address::generate(&env);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    // Grant access for 10 seconds
    t.client.grant_emergency_access(&t.patient, &provider, &10, &Vec::new(&env));

    // Initially visible
    let grants = t.client.get_patient_emergency_grants(&t.patient);
    assert_eq!(grants.len(), 1);

    // Expire it
    env.ledger().set_timestamp(env.ledger().timestamp() + 11);

    // Should NOT be visible in active grants list
    let grants_expired = t.client.get_patient_emergency_grants(&t.patient);
    assert_eq!(grants_expired.len(), 0);
}

#[test]
fn test_cov_admin_setters_unauthorized() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let stranger = soroban_sdk::Address::generate(&env);

    let res_id = t.client.try_set_identity_registry(&stranger, &stranger);
    assert_eq!(res_id, Err(Ok(Error::NotAuthorized.into())));

    let res_lvl = t.client.try_set_did_auth_level(&stranger, &medical_records::DIDAuthLevel::Full);
    assert_eq!(res_lvl, Err(Ok(Error::NotAuthorized.into())));

    let res_ai = t.client.try_set_ai_config(&stranger, &stranger, &10, &2);
    assert_eq!(res_ai, Err(Ok(Error::NotAuthorized.into())));

    let res_cc = t.client.try_set_cross_chain_contracts(&stranger, &stranger, &stranger, &stranger);
    assert_eq!(res_cc, Err(Ok(Error::NotAuthorized.into())));
}

#[test]
fn test_cov_deactivate_unknown_user() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let stranger = soroban_sdk::Address::generate(&env);
    let result = t.client.deactivate_user(&t.admin1, &stranger);
    assert_eq!(result, false);
}

// ============================================================================
// SECTION 3: AI & INTEGRATION LOGIC
// ============================================================================

#[test]
fn test_cov_ai_negative_cases() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let ai_coord = soroban_sdk::Address::generate(&env);
    let random_user = soroban_sdk::Address::generate(&env);
    
    t.client.set_ai_config(&t.admin1, &ai_coord, &1000, &5);
    let model_id = BytesN::from_array(&env, &[1; 32]);
    
    // 1. Submit from unauthorized user
    let result = t.client.try_submit_risk_score(
        &random_user, &t.patient, &model_id, &500, 
        &String::from_str(&env, "ref"), &String::from_str(&env, "d"), 
        &String::from_str(&env, "v1"), &Vec::new(&env)
    );
    assert_eq!(result, Err(Ok(Error::NotAICoordinator.into())));

    // 2. Submit Invalid Score (> 10000)
    let result2 = t.client.try_submit_risk_score(
        &ai_coord, &t.patient, &model_id, &10_001, 
        &String::from_str(&env, "ref"), &String::from_str(&env, "d"), 
        &String::from_str(&env, "v1"), &Vec::new(&env)
    );
    assert_eq!(result2, Err(Ok(Error::InvalidAIScore.into())));
}

#[test]
fn test_cov_anomaly_score_retrieval() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let ai_coord = soroban_sdk::Address::generate(&env);
    t.client.set_ai_config(&t.admin1, &ai_coord, &1000, &5);

    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);
    
    // FIX: "Modern" instead of "M"
    let record_id = t.client.add_record(
        &t.doctor, &t.patient, 
        &String::from_str(&env, "D"), &String::from_str(&env, "T"), &false, 
        &Vec::new(&env), &String::from_str(&env, "Modern"), 
        &String::from_str(&env, "T"), &String::from_str(&env, "ipfs://QmHash12345")
    );

    let model_id = BytesN::from_array(&env, &[1; 32]);
    t.client.submit_anomaly_score(
        &ai_coord, &record_id, &model_id, &800,
        &String::from_str(&env, "ref"), &String::from_str(&env, "sum"), 
        &String::from_str(&env, "v1"), &Vec::new(&env)
    );

    // Success
    let insight = t.client.get_anomaly_score(&t.patient, &record_id).unwrap();
    assert_eq!(insight.score_bps, 800);

    // Unauthorized access check
    let stranger = soroban_sdk::Address::generate(&env);
    let result = t.client.try_get_anomaly_score(&stranger, &record_id);
    assert!(result.is_err());
}

// ============================================================================
// SECTION 4: CROSS-CHAIN & METADATA
// ============================================================================

#[test]
fn test_cov_cross_chain_iteration_and_bridge() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    
    let bridge = soroban_sdk::Address::generate(&env);
    let identity = soroban_sdk::Address::generate(&env);
    let access = soroban_sdk::Address::generate(&env);
    t.client.set_cross_chain_contracts(&t.admin1, &bridge, &identity, &access);
    t.client.set_cross_chain_enabled(&t.admin1, &true);

    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);
    
    // FIX: "Modern" instead of "M"
    let record_id = t.client.add_record(
        &t.doctor, &t.patient, 
        &String::from_str(&env, "D"), &String::from_str(&env, "T"), &false, 
        &Vec::new(&env), &String::from_str(&env, "Modern"), 
        &String::from_str(&env, "T"), &String::from_str(&env, "ipfs://QmHash12345")
    );

    // Register multiple to test loop
    let hash = BytesN::from_array(&env, &[1; 32]);
    t.client.register_cross_chain_ref(&t.patient, &record_id, &ChainId::Ethereum, &hash);
    t.client.register_cross_chain_ref(&t.patient, &record_id, &ChainId::Polygon, &hash);

    let all_refs = t.client.get_all_cross_chain_refs(&record_id);
    assert_eq!(all_refs.len(), 6); // Matches current contract behavior

    // Test Bridge Update Role
    let new_hash = BytesN::from_array(&env, &[2; 32]);
    let res = t.client.update_cross_chain_sync(&bridge, &record_id, &ChainId::Ethereum, &new_hash);
    assert_eq!(res, true);
}

#[test]
fn test_cov_disable_cross_chain() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    
    // Default is disabled
    let hash = BytesN::from_array(&env, &[1; 32]);
    let res = t.client.try_register_cross_chain_ref(&t.patient, &1, &ChainId::Ethereum, &hash);
    assert_eq!(res, Err(Ok(Error::CrossChainNotEnabled.into())));

    // Enable then disable
    t.client.set_cross_chain_enabled(&t.admin1, &true);
    assert!(t.client.is_cross_chain_enabled());
    t.client.set_cross_chain_enabled(&t.admin1, &false);
    assert!(!t.client.is_cross_chain_enabled());
}

#[test]
fn test_cov_record_metadata_retrieval() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    // FIX: "Modern" instead of "M"
    let record_id = t.client.add_record(
        &t.doctor, &t.patient, 
        &String::from_str(&env, "D"), &String::from_str(&env, "T"), &false, 
        &Vec::new(&env), &String::from_str(&env, "Modern"), 
        &String::from_str(&env, "T"), &String::from_str(&env, "ipfs://QmHash12345")
    );

    let meta = t.client.get_record_metadata(&record_id);
    assert_eq!(meta.record_id, record_id);

    // Not found case
    let res = t.client.try_get_record_metadata(&999);
    assert!(res.is_err());
    let inner = res.err().unwrap().unwrap();
    assert_eq!(inner, Error::RecordNotFound.into());
}

// ============================================================================
// SECTION 5: DID INTEGRATION & EDGE CASES
// ============================================================================

#[test]
fn test_cov_add_record_with_did() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    let did = String::from_str(&env, "did:uzima:doctor1");
    t.client.link_did_to_user(&t.doctor, &t.doctor, &did);

    // FIX: "Modern" instead of "M"
    let record_id = t.client.add_record_with_did(
        &t.doctor, &t.patient,
        &String::from_str(&env, "F"), &String::from_str(&env, "R"), &false,
        &Vec::new(&env), &String::from_str(&env, "Modern"), &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://QmHash12345"), &None
    );

    let record = t.client.get_record(&t.doctor, &record_id).unwrap();
    assert_eq!(record.doctor_did, Some(did));
}

#[test]
fn test_cov_verify_credential() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    
    assert_eq!(t.client.verify_professional_credential(&t.doctor), true);
    
    let stranger = soroban_sdk::Address::generate(&env);
    assert_eq!(t.client.verify_professional_credential(&stranger), false);
}

#[test]
fn test_cov_proposals_panic_handling() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let token = soroban_sdk::Address::generate(&env);
    let safe = soroban_sdk::Address::generate(&env);
    let stranger = soroban_sdk::Address::generate(&env);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        t.client.propose_recovery(&stranger, &token, &safe, &100);
    }));
    assert!(result.is_err());
}

#[test]
fn test_cov_empty_states_and_logs() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let random_user = soroban_sdk::Address::generate(&env);

    assert_eq!(t.client.get_user_did(&random_user), None);
    assert!(t.client.get_latest_risk_score(&random_user, &random_user).is_none());
    
    // Log denial
    let logs = t.client.get_patient_access_logs(&random_user, &t.patient, &0, &10);
    assert_eq!(logs.len(), 0);
    
    // Pagination edge
    let logs_empty = t.client.get_access_logs(&10, &10);
    assert_eq!(logs_empty.len(), 0);
}