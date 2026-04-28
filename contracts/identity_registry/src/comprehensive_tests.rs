use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

// Helper function to create a contract instance
fn create_test_contract() -> (Env, IdentityRegistryContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(10_000);
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let network_id = String::from_str(&env, "testnet");
    client.initialize(&owner, &network_id);
    (env, client, owner)
}

// Helper to create DID for testing
fn create_test_did(
    env: &Env,
    client: &IdentityRegistryContractClient,
    subject: &Address,
) -> String {
    let public_key = BytesN::from_array(env, &[1u8; 32]);
    let services: Vec<ServiceEndpoint> = Vec::new(env);
    client.create_did(subject, &public_key, &services)
}

// ============================================================================
// INITIALIZATION EDGE CASES
// ============================================================================

#[test]
fn test_initialize_with_different_networks() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    let network_id = String::from_str(&env, "mainnet");
    client.initialize(&owner, &network_id);

    assert!(client.is_verifier(&owner));
}

#[test]
#[should_panic(expected = "Error(Contract, #300)")]
fn test_get_owner_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    client.get_owner();
}

// ============================================================================
// DID DOCUMENT EDGE CASES
// ============================================================================

#[test]
#[should_panic(expected = "Error(Contract, #470)")]
fn test_resolve_nonexistent_did() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    client.resolve_did(&subject);
}

#[test]
#[should_panic(expected = "Error(Contract, #470)")]
fn test_resolve_did_by_invalid_string() {
    let (env, client, _owner) = create_test_contract();
    let invalid_did = String::from_str(&env, "did:stellar:uzima:testnet:invalid");

    client.resolve_did_by_string(&invalid_did);
}

#[test]
#[should_panic(expected = "Error(Contract, #472)")]
fn test_update_deactivated_did() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);
    client.deactivate_did(&subject);

    let new_services: Vec<ServiceEndpoint> = Vec::new(&env);
    let new_also_known_as: Vec<String> = Vec::new(&env);
    client.update_did(&subject, &new_services, &new_also_known_as);
}

#[test]
fn test_did_version_increments() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);
    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.version, 1);

    let new_services: Vec<ServiceEndpoint> = Vec::new(&env);
    let new_also_known_as: Vec<String> = Vec::new(&env);
    client.update_did(&subject, &new_services, &new_also_known_as);

    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.version, 2);
}

#[test]
fn test_resolve_did_by_string() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    let did_string = create_test_did(&env, &client, &subject);
    let did_doc = client.resolve_did_by_string(&did_string);

    assert_eq!(did_doc.id, did_string);
    assert_eq!(did_doc.controller, subject);
}

// ============================================================================
// VERIFICATION METHOD EDGE CASES
// ============================================================================

#[test]
#[should_panic(expected = "Error(Contract, #450)")]
fn test_rotate_nonexistent_key() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let new_key = BytesN::from_array(&env, &[2u8; 32]);
    let method_id = String::from_str(&env, "#nonexistent-key");

    client.rotate_key(&subject, &method_id, &new_key);
}

#[test]
#[should_panic(expected = "Error(Contract, #603)")]
fn test_rotate_key_cooldown() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let new_key1 = BytesN::from_array(&env, &[2u8; 32]);
    let method_id = String::from_str(&env, "#key-1");

    env.ledger().set_timestamp(10_000);
    client.rotate_key(&subject, &method_id, &new_key1);

    // Try to rotate again immediately (should fail due to cooldown)
    env.ledger().set_timestamp(10_500); // Less than 1 hour cooldown
    let new_key2 = BytesN::from_array(&env, &[3u8; 32]);
    client.rotate_key(&subject, &method_id, &new_key2);
}

#[test]
fn test_rotate_key_after_cooldown() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let new_key1 = BytesN::from_array(&env, &[2u8; 32]);
    let method_id = String::from_str(&env, "#key-1");

    env.ledger().set_timestamp(10_000);
    client.rotate_key(&subject, &method_id, &new_key1);

    // Wait for cooldown to pass
    env.ledger().set_timestamp(14_000); // More than 1 hour later
    let new_key2 = BytesN::from_array(&env, &[3u8; 32]);
    client.rotate_key(&subject, &method_id, &new_key2);

    let did_doc = client.resolve_did(&subject);
    let vm = did_doc.verification_methods.get(0).unwrap();
    assert_eq!(vm.public_key, new_key2);
}

#[test]
#[should_panic(expected = "Error(Contract, #250)")]
fn test_revoke_last_verification_method() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    // Try to revoke the only verification method (should fail)
    let method_id = String::from_str(&env, "#key-1");
    client.revoke_verification_method(&subject, &method_id);
}

#[test]
fn test_add_multiple_verification_methods() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    // Add multiple verification methods
    let new_key2 = BytesN::from_array(&env, &[2u8; 32]);
    let method_id2 = String::from_str(&env, "#key-2");
    let mut relationships2: Vec<VerificationRelationship> = Vec::new(&env);
    relationships2.push_back(VerificationRelationship::Authentication);

    client.add_verification_method(
        &subject,
        &method_id2,
        &VerificationMethodType::Ed25519VerificationKey2020,
        &new_key2,
        &relationships2,
    );

    let new_key3 = BytesN::from_array(&env, &[3u8; 32]);
    let method_id3 = String::from_str(&env, "#key-3");
    let mut relationships3: Vec<VerificationRelationship> = Vec::new(&env);
    relationships3.push_back(VerificationRelationship::Authentication);

    client.add_verification_method(
        &subject,
        &method_id3,
        &VerificationMethodType::Ed25519VerificationKey2020,
        &new_key3,
        &relationships3,
    );

    let new_key4 = BytesN::from_array(&env, &[4u8; 32]);
    let method_id4 = String::from_str(&env, "#key-4");
    let mut relationships4: Vec<VerificationRelationship> = Vec::new(&env);
    relationships4.push_back(VerificationRelationship::Authentication);

    client.add_verification_method(
        &subject,
        &method_id4,
        &VerificationMethodType::Ed25519VerificationKey2020,
        &new_key4,
        &relationships4,
    );

    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.verification_methods.len(), 4);
}

// ============================================================================
// VERIFIABLE CREDENTIALS EDGE CASES
// ============================================================================

#[test]
#[should_panic(expected = "Error(Contract, #110)")]
fn test_issue_credential_not_verifier() {
    let (env, client, _owner) = create_test_contract();
    let non_verifier = Address::generate(&env);
    let subject = Address::generate(&env);

    let credential_hash = BytesN::from_array(&env, &[1u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");

    client.issue_credential(
        &non_verifier,
        &subject,
        &CredentialType::MedicalLicense,
        &credential_hash,
        &credential_uri,
        &0u64,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #460)")]
fn test_get_nonexistent_credential() {
    let (_env, client, _owner) = create_test_contract();
    let fake_id = BytesN::from_array(&_env, &[99u8; 32]);

    client.get_credential(&fake_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #100)")]
fn test_revoke_credential_not_issuer() {
    let (env, client, owner) = create_test_contract();
    let subject = Address::generate(&env);
    let non_issuer = Address::generate(&env);

    let credential_hash = BytesN::from_array(&env, &[1u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");

    let credential_id = client.issue_credential(
        &owner,
        &subject,
        &CredentialType::MedicalLicense,
        &credential_hash,
        &credential_uri,
        &0u64,
    );

    let reason = String::from_str(&env, "Unauthorized revocation");
    client.revoke_credential(&non_issuer, &credential_id, &reason);
}

#[test]
#[should_panic(expected = "Error(Contract, #606)")]
fn test_revoke_already_revoked_credential() {
    let (env, client, owner) = create_test_contract();
    let subject = Address::generate(&env);

    let credential_hash = BytesN::from_array(&env, &[1u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");

    let credential_id = client.issue_credential(
        &owner,
        &subject,
        &CredentialType::MedicalLicense,
        &credential_hash,
        &credential_uri,
        &0u64,
    );

    let reason = String::from_str(&env, "First revocation");
    client.revoke_credential(&owner, &credential_id, &reason);

    // Try to revoke again
    let reason2 = String::from_str(&env, "Second revocation");
    client.revoke_credential(&owner, &credential_id, &reason2);
}

#[test]
fn test_verify_credential_not_found() {
    let (env, client, _owner) = create_test_contract();
    let fake_id = BytesN::from_array(&env, &[99u8; 32]);

    let status = client.verify_credential(&fake_id);
    assert!(matches!(status, CredentialStatus::NotFound));
}

#[test]
fn test_multiple_credential_types() {
    let (env, client, owner) = create_test_contract();
    let subject = Address::generate(&env);

    // Issue multiple credential types
    let cred_hash1 = BytesN::from_array(&env, &[1u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");
    client.issue_credential(
        &owner,
        &subject,
        &CredentialType::MedicalLicense,
        &cred_hash1,
        &credential_uri,
        &0u64,
    );

    let cred_hash2 = BytesN::from_array(&env, &[2u8; 32]);
    client.issue_credential(
        &owner,
        &subject,
        &CredentialType::SpecialistCertification,
        &cred_hash2,
        &credential_uri,
        &0u64,
    );

    let cred_hash3 = BytesN::from_array(&env, &[3u8; 32]);
    client.issue_credential(
        &owner,
        &subject,
        &CredentialType::HospitalAffiliation,
        &cred_hash3,
        &credential_uri,
        &0u64,
    );

    let cred_hash4 = BytesN::from_array(&env, &[4u8; 32]);
    client.issue_credential(
        &owner,
        &subject,
        &CredentialType::ResearchAuthorization,
        &cred_hash4,
        &credential_uri,
        &0u64,
    );

    let credentials = client.get_subject_credentials(&subject);
    assert_eq!(credentials.len(), 4);
}

#[test]
fn test_has_valid_credential_with_revoked() {
    let (env, client, owner) = create_test_contract();
    let subject = Address::generate(&env);

    let credential_hash = BytesN::from_array(&env, &[1u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");

    let credential_id = client.issue_credential(
        &owner,
        &subject,
        &CredentialType::MedicalLicense,
        &credential_hash,
        &credential_uri,
        &0u64,
    );

    assert!(client.has_valid_credential(&subject, &CredentialType::MedicalLicense));

    let reason = String::from_str(&env, "Revoked");
    client.revoke_credential(&owner, &credential_id, &reason);

    assert!(!client.has_valid_credential(&subject, &CredentialType::MedicalLicense));
}

// ============================================================================
// IDENTITY RECOVERY EDGE CASES
// ============================================================================

#[test]
#[should_panic(expected = "Error(Contract, #120)")]
fn test_initiate_recovery_invalid_guardian() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);
    let fake_guardian = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[5u8; 32]);

    client.initiate_recovery(&fake_guardian, &subject, &new_controller, &new_key);
}

#[test]
#[should_panic(expected = "Error(Contract, #361)")]
fn test_initiate_recovery_already_pending() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let guardian = Address::generate(&env);
    client.add_recovery_guardian(&subject, &guardian, &2u32);

    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[5u8; 32]);

    client.initiate_recovery(&guardian, &subject, &new_controller, &new_key);

    // Try to initiate another recovery
    let new_controller2 = Address::generate(&env);
    let new_key2 = BytesN::from_array(&env, &[6u8; 32]);
    client.initiate_recovery(&guardian, &subject, &new_controller2, &new_key2);
}

#[test]
fn test_remove_recovery_guardian() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let guardian = Address::generate(&env);
    client.add_recovery_guardian(&subject, &guardian, &2u32);
    client.remove_recovery_guardian(&subject, &guardian);

    // Guardian should be removed (verify by trying to initiate recovery)
    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[5u8; 32]);

    let result = client.try_initiate_recovery(&guardian, &subject, &new_controller, &new_key);
    assert!(result.is_err());
}

#[test]
fn test_set_recovery_threshold() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    client.set_recovery_threshold(&subject, &5u32);

    // Threshold should be set (verify through recovery process)
    let guardian1 = Address::generate(&env);
    let guardian2 = Address::generate(&env);

    client.add_recovery_guardian(&subject, &guardian1, &2u32);
    client.add_recovery_guardian(&subject, &guardian2, &2u32);

    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[5u8; 32]);

    let request_id = client.initiate_recovery(&guardian1, &subject, &new_controller, &new_key);
    client.approve_recovery(&guardian2, &request_id);

    // Should fail to execute because total weight (4) < threshold (5)
    env.ledger().set_timestamp(100_000);
    let result = client.try_execute_recovery(&request_id);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Error(Contract, #362)")]
fn test_execute_recovery_before_timelock() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let guardian = Address::generate(&env);
    client.add_recovery_guardian(&subject, &guardian, &2u32);

    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[5u8; 32]);

    let request_id = client.initiate_recovery(&guardian, &subject, &new_controller, &new_key);

    // Try to execute immediately (should fail due to timelock)
    client.execute_recovery(&request_id);
}

#[test]
fn test_execute_recovery_success() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let guardian1 = Address::generate(&env);
    let guardian2 = Address::generate(&env);

    client.add_recovery_guardian(&subject, &guardian1, &1u32);
    client.add_recovery_guardian(&subject, &guardian2, &1u32);

    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[5u8; 32]);

    let request_id = client.initiate_recovery(&guardian1, &subject, &new_controller, &new_key);
    client.approve_recovery(&guardian2, &request_id);

    // Wait for timelock
    env.ledger().set_timestamp(100_000);

    client.execute_recovery(&request_id);

    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.controller, new_controller);
    assert!(matches!(did_doc.status, DIDStatus::Active));
}

#[test]
fn test_approve_recovery_duplicate() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let guardian = Address::generate(&env);
    client.add_recovery_guardian(&subject, &guardian, &2u32);

    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[5u8; 32]);

    let request_id = client.initiate_recovery(&guardian, &subject, &new_controller, &new_key);

    // Approve twice (should not increase weight twice)
    client.approve_recovery(&guardian, &request_id);
    client.approve_recovery(&guardian, &request_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #360)")]
fn test_cancel_recovery_not_initiated() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    client.cancel_recovery(&subject);
}

// ============================================================================
// SERVICE ENDPOINT EDGE CASES
// ============================================================================

#[test]
#[should_panic(expected = "Error(Contract, #462)")]
fn test_remove_nonexistent_service() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let service_id = String::from_str(&env, "#nonexistent");
    client.remove_service(&subject, &service_id);
}

#[test]
fn test_add_multiple_services() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let service_id1 = String::from_str(&env, "#service-1");
    let service_type1 = String::from_str(&env, "TestService");
    let endpoint1 = String::from_str(&env, "https://service1.com");
    client.add_service(&subject, &service_id1, &service_type1, &endpoint1);

    let service_id2 = String::from_str(&env, "#service-2");
    let service_type2 = String::from_str(&env, "TestService");
    let endpoint2 = String::from_str(&env, "https://service2.com");
    client.add_service(&subject, &service_id2, &service_type2, &endpoint2);

    let service_id3 = String::from_str(&env, "#service-3");
    let service_type3 = String::from_str(&env, "TestService");
    let endpoint3 = String::from_str(&env, "https://service3.com");
    client.add_service(&subject, &service_id3, &service_type3, &endpoint3);

    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.services.len(), 3);
}

#[test]
#[should_panic(expected = "Error(Contract, #472)")]
fn test_add_service_to_deactivated_did() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);
    client.deactivate_did(&subject);

    let service_id = String::from_str(&env, "#service-1");
    let service_type = String::from_str(&env, "TestService");
    let endpoint = String::from_str(&env, "https://test.com");

    client.add_service(&subject, &service_id, &service_type, &endpoint);
}

// ============================================================================
// VERIFIER MANAGEMENT EDGE CASES
// ============================================================================

#[test]
fn test_is_verifier_false() {
    let (env, client, _owner) = create_test_contract();
    let non_verifier = Address::generate(&env);

    assert!(!client.is_verifier(&non_verifier));
}

#[test]
fn test_add_multiple_verifiers() {
    let (env, client, _owner) = create_test_contract();

    let verifier1 = Address::generate(&env);
    let verifier2 = Address::generate(&env);
    let verifier3 = Address::generate(&env);

    client.add_verifier(&verifier1);
    client.add_verifier(&verifier2);
    client.add_verifier(&verifier3);

    assert!(client.is_verifier(&verifier1));
    assert!(client.is_verifier(&verifier2));
    assert!(client.is_verifier(&verifier3));
}

// ============================================================================
// LEGACY FUNCTION EDGE CASES
// ============================================================================

#[test]
fn test_legacy_get_identity_hash_nonexistent() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    let result = client.get_identity_hash(&subject);
    assert!(result.is_none());
}

#[test]
fn test_legacy_get_identity_meta_nonexistent() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    let result = client.get_identity_meta(&subject);
    assert!(result.is_none());
}

#[test]
fn test_legacy_is_attested_false() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);
    let claim_hash = BytesN::from_array(&env, &[1; 32]);

    assert!(!client.is_attested(&subject, &claim_hash));
}

#[test]
fn test_legacy_get_attestations_empty() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    let attestations = client.get_attestations(&subject);
    assert_eq!(attestations.len(), 0);
}

#[test]
fn test_legacy_multiple_attestations() {
    let (env, client, _owner) = create_test_contract();
    let verifier = Address::generate(&env);
    let subject = Address::generate(&env);

    client.add_verifier(&verifier);

    for i in 0..5 {
        let claim_hash = BytesN::from_array(&env, &[i as u8; 32]);
        client.attest(&verifier, &subject, &claim_hash);
    }

    let attestations = client.get_attestations(&subject);
    assert_eq!(attestations.len(), 5);
}

// ============================================================================
// DID AUTHORIZATION EDGE CASES
// ============================================================================

#[test]
fn test_verify_did_authorization_no_did() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    assert!(!client.verify_did_authorization(&subject, &VerificationRelationship::Authentication));
}

#[test]
fn test_verify_did_authorization_multiple_relationships() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    // Add a key for key agreement
    let new_key = BytesN::from_array(&env, &[2u8; 32]);
    let method_id = String::from_str(&env, "#key-agreement-1");
    let mut relationships: Vec<VerificationRelationship> = Vec::new(&env);
    relationships.push_back(VerificationRelationship::KeyAgreement);

    client.add_verification_method(
        &subject,
        &method_id,
        &VerificationMethodType::X25519KeyAgreementKey2020,
        &new_key,
        &relationships,
    );

    assert!(client.verify_did_authorization(&subject, &VerificationRelationship::Authentication));
    assert!(client.verify_did_authorization(&subject, &VerificationRelationship::KeyAgreement));
    // CapabilityDelegation is added by default in create_did, so it should be true
    assert!(
        client.verify_did_authorization(&subject, &VerificationRelationship::CapabilityDelegation)
    );
}

#[test]
fn test_verify_did_authorization_revoked_method() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    // Add second method
    let new_key = BytesN::from_array(&env, &[2u8; 32]);
    let method_id = String::from_str(&env, "#key-2");
    let mut relationships: Vec<VerificationRelationship> = Vec::new(&env);
    relationships.push_back(VerificationRelationship::Authentication);

    client.add_verification_method(
        &subject,
        &method_id,
        &VerificationMethodType::Ed25519VerificationKey2020,
        &new_key,
        &relationships,
    );

    // Revoke first method
    let first_method_id = String::from_str(&env, "#key-1");
    client.revoke_verification_method(&subject, &first_method_id);

    // Should still be authorized (second method is active)
    assert!(client.verify_did_authorization(&subject, &VerificationRelationship::Authentication));
}

// ============================================================================
// FIDO2 DEVICE TESTS
// ============================================================================

#[test]
fn test_add_fido2_device_eddsa() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let device_name = String::from_str(&env, "#fido2-yubikey");
    let algorithm_tag = 1u32; // EdDSA
    let public_key_hash = BytesN::from_array(&env, &[10u8; 32]);

    client.add_fido2_device(&subject, &device_name, &algorithm_tag, &public_key_hash);

    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.verification_methods.len(), 2);

    let fido2_vm = did_doc.verification_methods.get(1).unwrap();
    assert!(matches!(
        fido2_vm.method_type,
        VerificationMethodType::Fido2EdDsa2024
    ));
}

#[test]
fn test_add_fido2_device_es256() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    let device_name = String::from_str(&env, "#fido2-touchid");
    let algorithm_tag = 2u32; // ES256
    let public_key_hash = BytesN::from_array(&env, &[11u8; 32]);

    client.add_fido2_device(&subject, &device_name, &algorithm_tag, &public_key_hash);

    let did_doc = client.resolve_did(&subject);
    let fido2_vm = did_doc.verification_methods.get(1).unwrap();
    assert!(matches!(
        fido2_vm.method_type,
        VerificationMethodType::Fido2Es2562024
    ));
}

#[test]
fn test_add_fido2_device_no_did() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    // Don't create DID - should silently succeed
    let device_name = String::from_str(&env, "#fido2-device");
    let algorithm_tag = 1u32;
    let public_key_hash = BytesN::from_array(&env, &[12u8; 32]);

    client.add_fido2_device(&subject, &device_name, &algorithm_tag, &public_key_hash);
    // Should complete without error
}

#[test]
fn test_add_fido2_device_deactivated_did() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);
    client.deactivate_did(&subject);

    let device_name = String::from_str(&env, "#fido2-device");
    let algorithm_tag = 1u32;
    let public_key_hash = BytesN::from_array(&env, &[13u8; 32]);

    // Should silently succeed (non-blocking)
    client.add_fido2_device(&subject, &device_name, &algorithm_tag, &public_key_hash);
    // Should complete without error
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_full_identity_lifecycle() {
    let (env, client, owner) = create_test_contract();
    let subject = Address::generate(&env);

    // 1. Create DID
    let did_string = create_test_did(&env, &client, &subject);
    assert!(!did_string.is_empty());

    // 2. Add verification method
    let new_key = BytesN::from_array(&env, &[2u8; 32]);
    let method_id = String::from_str(&env, "#key-2");
    let mut relationships: Vec<VerificationRelationship> = Vec::new(&env);
    relationships.push_back(VerificationRelationship::Authentication);

    client.add_verification_method(
        &subject,
        &method_id,
        &VerificationMethodType::Ed25519VerificationKey2020,
        &new_key,
        &relationships,
    );

    // 3. Issue credential
    let credential_hash = BytesN::from_array(&env, &[1u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");

    let credential_id = client.issue_credential(
        &owner,
        &subject,
        &CredentialType::MedicalLicense,
        &credential_hash,
        &credential_uri,
        &0u64,
    );

    // 4. Verify credential
    let status = client.verify_credential(&credential_id);
    assert!(matches!(status, CredentialStatus::Valid));

    // 5. Add service
    let service_id = String::from_str(&env, "#medical-records");
    let service_type = String::from_str(&env, "MedicalRecords");
    let endpoint = String::from_str(&env, "ipfs://QmRecords");

    client.add_service(&subject, &service_id, &service_type, &endpoint);

    // 6. Verify final state
    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.verification_methods.len(), 2);
    assert_eq!(did_doc.services.len(), 1);
    assert!(matches!(did_doc.status, DIDStatus::Active));
}

#[test]
fn test_multi_signature_recovery_workflow() {
    let (env, client, _owner) = create_test_contract();
    let subject = Address::generate(&env);

    create_test_did(&env, &client, &subject);

    // Setup multi-sig recovery with 3 guardians
    let guardian1 = Address::generate(&env);
    let guardian2 = Address::generate(&env);
    let guardian3 = Address::generate(&env);

    client.add_recovery_guardian(&subject, &guardian1, &1u32);
    client.add_recovery_guardian(&subject, &guardian2, &1u32);
    client.add_recovery_guardian(&subject, &guardian3, &1u32);

    // Set threshold to 2
    client.set_recovery_threshold(&subject, &2u32);

    // Initiate recovery
    let new_controller = Address::generate(&env);
    let new_key = BytesN::from_array(&env, &[99u8; 32]);

    let request_id = client.initiate_recovery(&guardian1, &subject, &new_controller, &new_key);

    // Second guardian approves
    client.approve_recovery(&guardian2, &request_id);

    // Wait for timelock
    env.ledger().set_timestamp(100_000);

    // Execute recovery
    client.execute_recovery(&request_id);

    // Verify new controller
    let did_doc = client.resolve_did(&subject);
    assert_eq!(did_doc.controller, new_controller);
}

#[test]
fn test_credential_expiration_workflow() {
    let (env, client, owner) = create_test_contract();
    let subject = Address::generate(&env);

    env.ledger().set_timestamp(10_000);

    let credential_hash = BytesN::from_array(&env, &[1u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");
    let expiration = 20_000u64;

    let credential_id = client.issue_credential(
        &owner,
        &subject,
        &CredentialType::MedicalLicense,
        &credential_hash,
        &credential_uri,
        &expiration,
    );

    // Valid initially
    let status = client.verify_credential(&credential_id);
    assert!(matches!(status, CredentialStatus::Valid));

    // Still valid before expiration
    env.ledger().set_timestamp(19_000);
    let status = client.verify_credential(&credential_id);
    assert!(matches!(status, CredentialStatus::Valid));

    // Expired after expiration time
    env.ledger().set_timestamp(21_000);
    let status = client.verify_credential(&credential_id);
    assert!(matches!(status, CredentialStatus::Expired));
}

#[test]
fn test_multiple_subjects_isolation() {
    let (env, client, owner) = create_test_contract();

    let subject1 = Address::generate(&env);
    let subject2 = Address::generate(&env);

    // Create DIDs for both subjects
    create_test_did(&env, &client, &subject1);
    create_test_did(&env, &client, &subject2);

    // Issue credentials to both
    let cred_hash1 = BytesN::from_array(&env, &[1u8; 32]);
    let cred_hash2 = BytesN::from_array(&env, &[2u8; 32]);
    let credential_uri = String::from_str(&env, "ipfs://QmTest");

    client.issue_credential(
        &owner,
        &subject1,
        &CredentialType::MedicalLicense,
        &cred_hash1,
        &credential_uri,
        &0u64,
    );

    client.issue_credential(
        &owner,
        &subject2,
        &CredentialType::SpecialistCertification,
        &cred_hash2,
        &credential_uri,
        &0u64,
    );

    // Verify isolation
    let creds1 = client.get_subject_credentials(&subject1);
    let creds2 = client.get_subject_credentials(&subject2);

    assert_eq!(creds1.len(), 1);
    assert_eq!(creds2.len(), 1);
    assert_ne!(
        creds1.get(0).unwrap().credential_type,
        creds2.get(0).unwrap().credential_type
    );
}
