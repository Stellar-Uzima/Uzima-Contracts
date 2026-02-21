#![allow(clippy::unwrap_used)]

use crate::{
    ChainId, CrossChainIdentityContract, CrossChainIdentityContractClient, Error, RequestStatus,
    SyncStatus, VerificationStatus,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn create_contract(env: &Env) -> (CrossChainIdentityContractClient<'_>, Address, Address) {
    let contract_id = env.register_contract(None, CrossChainIdentityContract);
    let client = CrossChainIdentityContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let bridge = Address::generate(&env);
    (client, admin, bridge)
}

fn initialize_contract(
    env: &Env,
    client: &CrossChainIdentityContractClient,
    admin: &Address,
    bridge: &Address,
) {
    env.mock_all_auths();
    client.initialize(admin, bridge);
}

fn generate_proof(env: &Env) -> BytesN<64> {
    BytesN::from_array(env, &[1u8; 64])
}

fn generate_public_key(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[2u8; 32])
}

fn generate_sync_proof(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[3u8; 32])
}

// ==================== Initialization Tests ====================

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);

    initialize_contract(&env, &client, &admin, &bridge);

    assert!(!client.is_paused());
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);

    env.mock_all_auths();
    client.initialize(&admin, &bridge);

    let result = client.try_initialize(&admin, &bridge);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

// ==================== Validator Tests ====================

#[test]
fn test_add_validator() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    let result = client.add_validator(&admin, &validator, &name, &public_key);
    assert!(result);

    let validator_info = client.get_validator(&validator);
    assert!(validator_info.is_some());

    let v = validator_info.unwrap();
    assert!(v.is_active);
    assert_eq!(v.trust_score, 50);
    assert_eq!(v.total_attestations, 0);
}

#[test]
fn test_deactivate_validator() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &name, &public_key);
    client.deactivate_validator(&admin, &validator);

    let validator_info = client.get_validator(&validator).unwrap();
    assert!(!validator_info.is_active);
}

#[test]
fn test_update_trust_score() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &name, &public_key);
    client.update_trust_score(&admin, &validator, &85);

    let validator_info = client.get_validator(&validator).unwrap();
    assert_eq!(validator_info.trust_score, 85);
}

#[test]
fn test_trust_score_capped_at_100() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &name, &public_key);
    client.update_trust_score(&admin, &validator, &150);

    let validator_info = client.get_validator(&validator).unwrap();
    assert_eq!(validator_info.trust_score, 100);
}

// ==================== Verification Request Tests ====================

#[test]
fn test_request_verification() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    assert_eq!(request_id, 1);

    let request = client.get_request(&request_id).unwrap();
    assert_eq!(request.status, RequestStatus::Pending);
    assert_eq!(request.stellar_address, user);
}

#[test]
fn test_attest_verification() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &name, &public_key);

    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    let result = client.attest_verification(&validator, &request_id, &true, &signature);
    assert!(result);

    let validator_info = client.get_validator(&validator).unwrap();
    assert_eq!(validator_info.total_attestations, 1);
}

#[test]
fn test_verification_approved_with_enough_attestations() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator1, &request_id, &true, &signature);

    // First attestation - still pending
    let request = client.get_request(&request_id).unwrap();
    assert_eq!(request.status, RequestStatus::Pending);

    client.attest_verification(&validator2, &request_id, &true, &signature);

    // Second attestation - approved
    let request = client.get_request(&request_id).unwrap();
    assert_eq!(request.status, RequestStatus::Approved);

    // Identity should now be verified
    let identity = client.get_identity(&user, &ChainId::Ethereum).unwrap();
    assert_eq!(identity.verification_status, VerificationStatus::Verified);
}

#[test]
fn test_duplicate_attestation_fails() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator, &name, &public_key);

    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator, &request_id, &true, &signature);

    // Second attestation from same validator should fail
    let result = client.try_attest_verification(&validator, &request_id, &true, &signature);
    assert_eq!(result, Err(Ok(Error::DuplicateAttestation)));
}

// ==================== Identity Tests ====================

#[test]
fn test_verify_identity() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    // Before verification
    assert!(!client.verify_identity(&user, &ChainId::Ethereum));

    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator1, &request_id, &true, &signature);
    client.attest_verification(&validator2, &request_id, &true, &signature);

    // After verification
    assert!(client.verify_identity(&user, &ChainId::Ethereum));
}

#[test]
fn test_revoke_identity() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator1, &request_id, &true, &signature);
    client.attest_verification(&validator2, &request_id, &true, &signature);

    // Revoke by owner
    client.revoke_identity(&user, &user, &ChainId::Ethereum);

    let identity = client.get_identity(&user, &ChainId::Ethereum).unwrap();
    assert_eq!(identity.verification_status, VerificationStatus::Revoked);
}

#[test]
fn test_revoke_identity_by_admin() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator1, &request_id, &true, &signature);
    client.attest_verification(&validator2, &request_id, &true, &signature);

    // Revoke by admin
    client.revoke_identity(&admin, &user, &ChainId::Ethereum);

    let identity = client.get_identity(&user, &ChainId::Ethereum).unwrap();
    assert_eq!(identity.verification_status, VerificationStatus::Revoked);
}

// ==================== Sync Tests ====================

#[test]
fn test_initiate_sync() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    // First verify identity
    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator1, &request_id, &true, &signature);
    client.attest_verification(&validator2, &request_id, &true, &signature);

    // Now initiate sync
    let sync_id = client.initiate_sync(&user, &ChainId::Ethereum, &ChainId::Polygon);

    let sync = client.get_sync(&sync_id).unwrap();
    assert_eq!(sync.sync_status, SyncStatus::Initiated);
    assert_eq!(sync.stellar_address, user);
}

#[test]
fn test_update_sync_status() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    // First verify identity
    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator1, &request_id, &true, &signature);
    client.attest_verification(&validator2, &request_id, &true, &signature);

    let sync_id = client.initiate_sync(&user, &ChainId::Ethereum, &ChainId::Polygon);

    let sync_proof = generate_sync_proof(&env);
    client.update_sync_status(&validator1, &sync_id, &SyncStatus::Completed, &sync_proof);

    let sync = client.get_sync(&sync_id).unwrap();
    assert_eq!(sync.sync_status, SyncStatus::Completed);
}

// ==================== Pause Tests ====================

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    env.mock_all_auths();

    assert!(!client.is_paused());

    client.pause(&admin);
    assert!(client.is_paused());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
fn test_operations_blocked_when_paused() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let user = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.pause(&admin);

    let result =
        client.try_request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

// ==================== Storage Key Uniqueness Regression Tests ====================

/// Regression test: one user can have identities on multiple chains independently
#[test]
fn test_identities_unique_per_chain() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let eth_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let poly_address = String::from_str(&env, "0xabcdef1234567890abcdef1234567890abcdef12");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);
    let signature = generate_proof(&env);

    // Verify on Ethereum
    let req_id1 = client.request_verification(&user, &ChainId::Ethereum, &eth_address, &proof);
    client.attest_verification(&validator1, &req_id1, &true, &signature);
    client.attest_verification(&validator2, &req_id1, &true, &signature);

    // Verify on Polygon
    let req_id2 = client.request_verification(&user, &ChainId::Polygon, &poly_address, &proof);
    client.attest_verification(&validator1, &req_id2, &true, &signature);
    client.attest_verification(&validator2, &req_id2, &true, &signature);

    // Both identities should exist and be independent
    let eth_id = client.get_identity(&user, &ChainId::Ethereum).unwrap();
    let poly_id = client.get_identity(&user, &ChainId::Polygon).unwrap();

    assert_eq!(eth_id.verification_status, VerificationStatus::Verified);
    assert_eq!(poly_id.verification_status, VerificationStatus::Verified);
    assert_eq!(eth_id.external_address, eth_address);
    assert_eq!(poly_id.external_address, poly_address);

    // Revoking Ethereum should not affect Polygon
    client.revoke_identity(&user, &user, &ChainId::Ethereum);

    let eth_id_after = client.get_identity(&user, &ChainId::Ethereum).unwrap();
    let poly_id_after = client.get_identity(&user, &ChainId::Polygon).unwrap();
    assert_eq!(eth_id_after.verification_status, VerificationStatus::Revoked);
    assert_eq!(poly_id_after.verification_status, VerificationStatus::Verified);
}

/// Regression test: attestations for different requests must be independent
#[test]
fn test_attestations_unique_per_request() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let proof = generate_proof(&env);
    let sig = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    // Create two separate verification requests
    let req_id1 =
        client.request_verification(&user1, &ChainId::Ethereum, &String::from_str(&env, "0x111"), &proof);
    let req_id2 =
        client.request_verification(&user2, &ChainId::Ethereum, &String::from_str(&env, "0x222"), &proof);

    // Attest only request 1 with both validators
    client.attest_verification(&validator1, &req_id1, &true, &sig);
    client.attest_verification(&validator2, &req_id1, &true, &sig);

    // Request 1 should be approved, request 2 still pending
    let r1 = client.get_request(&req_id1).unwrap();
    let r2 = client.get_request(&req_id2).unwrap();
    assert_eq!(r1.status, RequestStatus::Approved);
    assert_eq!(r2.status, RequestStatus::Pending);

    // Attestation lookup works per (request_id, validator)
    let att = client.get_attestation(&req_id1, &validator1).unwrap();
    assert!(att.is_valid);
    assert_eq!(att.validator, validator1);

    // No attestation should exist for request 2 from validator1
    assert!(client.get_attestation(&req_id2, &validator1).is_none());
}

/// Regression test: sync operations are tracked separately from verification requests
#[test]
fn test_sync_count_independent_from_request_count() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "V1");
    let name2 = String::from_str(&env, "V2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let proof = generate_proof(&env);
    let sig = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    // Create verification request (request_count=1)
    let req_id = client.request_verification(
        &user,
        &ChainId::Ethereum,
        &String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678"),
        &proof,
    );
    client.attest_verification(&validator1, &req_id, &true, &sig);
    client.attest_verification(&validator2, &req_id, &true, &sig);

    // Initiate sync (sync_count=1)
    let sync_id = client.initiate_sync(&user, &ChainId::Ethereum, &ChainId::Polygon);

    // Verify sync ID starts from 1 (independent counter)
    assert_eq!(sync_id, 1);

    let sync = client.get_sync(&sync_id).unwrap();
    assert_eq!(sync.sync_status, SyncStatus::Initiated);
}

// ==================== Authorization Tests ====================

#[test]
fn test_add_validator_not_admin() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let non_admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let name = String::from_str(&env, "Validator1");
    let public_key = generate_public_key(&env);

    env.mock_all_auths();
    let result = client.try_add_validator(&non_admin, &validator, &name, &public_key);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_revoke_not_authorized() {
    let env = Env::default();
    let (client, admin, bridge) = create_contract(&env);
    initialize_contract(&env, &client, &admin, &bridge);

    let validator1 = Address::generate(&env);
    let validator2 = Address::generate(&env);
    let name1 = String::from_str(&env, "Validator1");
    let name2 = String::from_str(&env, "Validator2");
    let public_key = generate_public_key(&env);

    let user = Address::generate(&env);
    let other = Address::generate(&env);
    let external_address = String::from_str(&env, "0x1234567890abcdef1234567890abcdef12345678");
    let proof = generate_proof(&env);

    env.mock_all_auths();
    client.add_validator(&admin, &validator1, &name1, &public_key);
    client.add_validator(&admin, &validator2, &name2, &public_key);

    let request_id =
        client.request_verification(&user, &ChainId::Ethereum, &external_address, &proof);

    let signature = generate_proof(&env);
    client.attest_verification(&validator1, &request_id, &true, &signature);
    client.attest_verification(&validator2, &request_id, &true, &signature);

    // Try to revoke by non-owner/non-admin
    let result = client.try_revoke_identity(&other, &user, &ChainId::Ethereum);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}
