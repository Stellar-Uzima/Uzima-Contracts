#![cfg(test)]

mod common;
use common::setup_uzima;
use medical_records::{DIDAuthLevel, Error, Role};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, String,
};

#[test]
fn test_input_validation_coverage() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);

    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);

    // 1. Test empty diagnosis
    let res_diag = t.client.try_add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, ""), // Empty diagnosis
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://cid"),
    );
    assert_eq!(res_diag, Err(Ok(Error::EmptyDiagnosis)));

    // 2. Test invalid category
    let res_cat = t.client.try_add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "D"),
        &String::from_str(&env, "T"),
        &false,
        &vec![&env],
        &String::from_str(&env, "InvalidCat"), // Invalid category
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://cid"),
    );
    assert_eq!(res_cat, Err(Ok(Error::InvalidCategory)));

    // 3. Test empty treatment
    let res_treat = t.client.try_add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "D"),
        &String::from_str(&env, ""), // Empty treatment
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://cid"),
    );
    assert_eq!(res_treat, Err(Ok(Error::EmptyTreatment)));

    // 4. Test empty tag in vector
    let res_tag = t.client.try_add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "D"),
        &String::from_str(&env, "T"),
        &false,
        &vec![&env, String::from_str(&env, "")], // Empty tag
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs://cid"),
    );
    assert_eq!(res_tag, Err(Ok(Error::EmptyTag)));
}

#[test]
fn test_emergency_access_full_lifecycle() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);
    let provider = soroban_sdk::Address::generate(&env);

    // 1. Grant access
    t.client
        .grant_emergency_access(&t.patient, &provider, &3600, &vec![&env, 1u64]);

    // 2. Check access valid
    assert!(t.client.has_emergency_access(&provider, &t.patient, &1));

    // 3. Revoke access
    t.client.revoke_emergency_access(&t.patient, &provider);

    // 4. Check access invalid
    assert!(!t.client.has_emergency_access(&provider, &t.patient, &1));

    // 5. Try to revoke non-existent access
    let result = t.client.try_revoke_emergency_access(&t.patient, &provider);
    assert_eq!(result, Err(Ok(Error::EmergencyAccessNotFound)));
}

#[test]
fn test_ai_integration_security() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let ai_coord = soroban_sdk::Address::generate(&env);
    let model_id = soroban_sdk::BytesN::from_array(&env, &[1; 32]);
    let features = vec![&env];

    t.client.set_ai_config(&t.admin1, &ai_coord, &1000, &5);

    // 2. Try submitting from non-coordinator address (unregistered user)
    let imposter = soroban_sdk::Address::generate(&env);
    let res_id = t.client.try_submit_risk_score(
        &imposter,
        &t.patient,
        &model_id,
        &5000,
        &String::from_str(&env, "ref"),
        &String::from_str(&env, "reason"),
        &String::from_str(&env, "v1"),
        &features,
    );
    // FIXED: Unregistered user should get NotAICoordinator, not NotAuthorized
    assert_eq!(res_id, Err(Ok(Error::NotAICoordinator)));

    // 3. Try setting AI config as non-admin
    let res_lvl = t.client.try_set_ai_config(&imposter, &ai_coord, &1000, &5);
    assert_eq!(res_lvl, Err(Ok(Error::NotAuthorized)));

    // 4. Try setting DID auth level as non-admin
    // FIXED: Used `Basic` which is confirmed to exist
    let res_ai = t
        .client
        .try_set_did_auth_level(&imposter, &DIDAuthLevel::Basic);
    assert_eq!(res_ai, Err(Ok(Error::NotAuthorized)));

    // 5. Try setting cross chain contracts as non-admin
    let addr = soroban_sdk::Address::generate(&env);
    let res_cc = t
        .client
        .try_set_cross_chain_contracts(&imposter, &addr, &addr, &addr);
    assert_eq!(res_cc, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_did_linkage_verification() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let did = String::from_str(&env, "did:stellar:123");

    // 1. Link DID
    t.client.link_did_to_user(&t.patient, &t.patient, &did);

    // Verify via get_user_did
    let retrieved_did = t.client.get_user_did(&t.patient);
    assert_eq!(retrieved_did, Some(did));
}

#[test]
fn test_ai_anomaly_detection_flow() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let ai_coord = soroban_sdk::Address::generate(&env);
    let model_id = soroban_sdk::BytesN::from_array(&env, &[2; 32]);

    t.client.set_ai_config(&t.admin1, &ai_coord, &1000, &5);

    // 1. Add record
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    let record_id = t.client.add_record(
        &t.doctor,
        &t.patient,
        &String::from_str(&env, "D"),
        &String::from_str(&env, "T"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "T"),
        &String::from_str(&env, "ipfs"),
    );

    // 2. Submit anomaly score (valid)
    t.client.submit_anomaly_score(
        &ai_coord,
        &record_id,
        &model_id,
        &8000,
        &String::from_str(&env, "ref"),
        &String::from_str(&env, "summary"),
        &String::from_str(&env, "v1"),
        &vec![&env],
    );

    // 3. Try submit anomaly from non-coordinator (unregistered user)
    let imposter = soroban_sdk::Address::generate(&env);
    let result = t.client.try_submit_anomaly_score(
        &imposter,
        &record_id,
        &model_id,
        &8000,
        &String::from_str(&env, "ref"),
        &String::from_str(&env, "summary"),
        &String::from_str(&env, "v1"),
        &vec![&env],
    );
    assert_eq!(result, Err(Ok(Error::NotAICoordinator)));

    // 4. Try submit invalid score (> 10000)
    let result2 = t.client.try_submit_anomaly_score(
        &ai_coord,
        &record_id,
        &model_id,
        &10001, // Invalid
        &String::from_str(&env, "ref"),
        &String::from_str(&env, "summary"),
        &String::from_str(&env, "v1"),
        &vec![&env],
    );
    assert_eq!(result2, Err(Ok(Error::InvalidAIScore)));
}

#[test]
fn test_role_transition_rules() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let user = soroban_sdk::Address::generate(&env);

    // 1. Initial role assignment (Patient)
    t.client.manage_user(&t.admin1, &user, &Role::Patient);
    assert!(matches!(t.client.get_user_role(&user), Role::Patient));

    // 2. Admin upgrades to Doctor
    t.client.manage_user(&t.admin1, &user, &Role::Doctor);
    assert!(matches!(t.client.get_user_role(&user), Role::Doctor));

    // 3. Admin upgrades to Admin
    t.client.manage_user(&t.admin1, &user, &Role::Admin);
    assert!(matches!(t.client.get_user_role(&user), Role::Admin));
}

#[test]
fn test_history_pagination_edge_cases() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);

    // Add 5 records
    for _ in 0..5 {
        t.client.add_record(
            &t.doctor,
            &t.patient,
            &String::from_str(&env, "D"),
            &String::from_str(&env, "T"),
            &false,
            &vec![&env],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "T"),
            &String::from_str(&env, "ipfs"),
        );
    }

    // 1. Request more than available
    let history = t.client.get_history(&t.patient, &t.patient, &0, &100);
    assert_eq!(history.len(), 5);

    // 2. Request offset beyond range
    let history_empty = t.client.get_history(&t.patient, &t.patient, &10, &5);
    assert_eq!(history_empty.len(), 0);

    // 3. Request zero limit
    let history_zero = t.client.get_history(&t.patient, &t.patient, &0, &0);
    assert_eq!(history_zero.len(), 0);
}

#[test]
fn test_time_bound_access_expiry() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    t.client.manage_user(&t.admin1, &t.patient, &Role::Patient);
    let provider = soroban_sdk::Address::generate(&env);

    // 1. Grant 1 hour access
    t.client
        .grant_emergency_access(&t.patient, &provider, &3600, &vec![&env, 1u64]);

    // 2. Check access immediately
    assert!(t.client.has_emergency_access(&provider, &t.patient, &1));

    // 3. Advance time 30 mins
    env.ledger().set_timestamp(env.ledger().timestamp() + 1800);
    assert!(t.client.has_emergency_access(&provider, &t.patient, &1));

    // 4. Advance time past 1 hour total
    env.ledger().set_timestamp(env.ledger().timestamp() + 1801);
    assert!(!t.client.has_emergency_access(&provider, &t.patient, &1));
}

#[test]
fn test_cross_chain_sync_updates() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let chain_id = medical_records::ChainId::Ethereum;
    let hash1 = soroban_sdk::BytesN::from_array(&env, &[1; 32]);
    let hash2 = soroban_sdk::BytesN::from_array(&env, &[2; 32]);

    t.client.set_cross_chain_enabled(&t.admin1, &true);

    // 1. Register initial reference
    t.client
        .register_cross_chain_ref(&t.patient, &1, &chain_id, &hash1);
    let ref1 = t.client.get_cross_chain_ref(&1, &chain_id).unwrap();
    assert_eq!(ref1.external_record_hash, hash1);

    // 2. Admin updates sync (simulating external change)
    t.client
        .update_cross_chain_sync(&t.admin1, &1, &chain_id, &hash2);
    let ref2 = t.client.get_cross_chain_ref(&1, &chain_id).unwrap();
    assert_eq!(ref2.external_record_hash, hash2);
}

#[test]
fn test_disabled_cross_chain_operations() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let chain_id = medical_records::ChainId::Ethereum;
    let hash = soroban_sdk::BytesN::from_array(&env, &[1; 32]);

    // Ensure disabled (default)
    // 1. Try register ref
    let res = t
        .client
        .try_register_cross_chain_ref(&t.patient, &1, &chain_id, &hash);
    assert_eq!(res, Err(Ok(Error::CrossChainNotEnabled)));
}

#[test]
fn test_credential_verification_logic() {
    let env = soroban_sdk::Env::default();
    let t = setup_uzima(&env);
    let stranger = soroban_sdk::Address::generate(&env);

    // 1. Verify doctor
    t.client.manage_user(&t.admin1, &t.doctor, &Role::Doctor);
    assert!(t.client.verify_professional_credential(&t.doctor));

    // 2. Verify stranger (should be false)
    assert!(!t.client.verify_professional_credential(&stranger));
}
