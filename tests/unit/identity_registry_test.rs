#[cfg(test)]
mod tests {
    use identity_registry::{IdentityRegistryContract, IdentityRegistryContractClient};
    use soroban_sdk::testutils::{Address as _, Events};
    use soroban_sdk::{Address, Env, String, BytesN};

    fn create_contract() -> (Env, IdentityRegistryContractClient<'static>, Address) {
        let env = Env::default();
        let contract_id = env.register_contract(None, IdentityRegistryContract);
        let client = IdentityRegistryContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        
        // Initialize the contract
        client.initialize(&owner);
        
        (env, client, owner)
    }

    #[test]
    fn test_initialize_and_owner_is_verifier() {
        let (env, client, owner) = create_contract();
        
        // Owner should be a verifier by default
        assert!(client.is_verifier(&owner));
        
        // Owner should be retrievable
        assert_eq!(client.get_owner(), owner);
    }

    #[test]
    fn test_register_identity_hash() {
        let (env, client, _owner) = create_contract();
        let subject = Address::generate(&env);
        
        let hash = BytesN::from_array(&env, &[1; 32]);
        let meta = String::from_str(&env, "Healthcare Provider License #12345");
        
        // Register identity hash
        client.register_identity_hash(&hash, &subject, &meta);
        
        // Verify storage
        assert_eq!(client.get_identity_hash(&subject), Some(hash));
        assert_eq!(client.get_identity_meta(&subject), Some(meta.clone()));
        
        // Verify event emission
        let events = env.events().all();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].topics,
            ("IdentityRegistered",)
        );
    }

    #[test]
    fn test_add_and_remove_verifier() {
        let (env, client, owner) = create_contract();
        let new_verifier = Address::generate(&env);
        
        // Add verifier
        client.add_verifier(&new_verifier);
        assert!(client.is_verifier(&new_verifier));
        
        // Remove verifier
        client.remove_verifier(&new_verifier);
        assert!(!client.is_verifier(&new_verifier));
        
        // Verify events
        let events = env.events().all();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].topics, ("VerifierAdded",));
        assert_eq!(events[1].topics, ("VerifierRemoved",));
    }

    #[test]
    #[should_panic(expected = "Cannot remove owner as verifier")]
    fn test_cannot_remove_owner_as_verifier() {
        let (_env, client, owner) = create_contract();
        
        // Try to remove owner as verifier (should panic)
        client.remove_verifier(&owner);
    }

    #[test]
    fn test_attest_and_verify() {
        let (env, client, owner) = create_contract();
        let verifier = Address::generate(&env);
        let subject = Address::generate(&env);
        
        // Add verifier
        client.add_verifier(&verifier);
        
        // Create attestation
        let claim_hash = BytesN::from_array(&env, &[2; 32]);
        client.mock_all_auths().attest(&subject, &claim_hash);
        
        // Verify attestation
        assert!(client.is_attested(&subject, &claim_hash));
        
        // Check attestations list
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 1);
        assert_eq!(attestations.get(0).unwrap(), claim_hash);
    }

    #[test]
    #[should_panic(expected = "Caller is not a verifier")]
    fn test_attest_unauthorized() {
        let (env, client, _owner) = create_contract();
        let unauthorized = Address::generate(&env);
        let subject = Address::generate(&env);
        
        let claim_hash = BytesN::from_array(&env, &[3; 32]);
        
        // Try to attest without being a verifier (should panic)
        client.mock_all_auths().attest(&subject, &claim_hash);
    }

    #[test]
    fn test_revoke_attestation() {
        let (env, client, owner) = create_contract();
        let verifier = Address::generate(&env);
        let subject = Address::generate(&env);
        
        // Add verifier
        client.add_verifier(&verifier);
        
        // Create attestation
        let claim_hash = BytesN::from_array(&env, &[4; 32]);
        client.mock_all_auths().attest(&subject, &claim_hash);
        
        // Verify attestation exists
        assert!(client.is_attested(&subject, &claim_hash));
        
        // Revoke attestation
        client.mock_all_auths().revoke_attestation(&subject, &claim_hash);
        
        // Verify attestation is revoked
        assert!(!client.is_attested(&subject, &claim_hash));
        
        // Check attestations list (should be empty after revocation)
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Caller is not a verifier")]
    fn test_revoke_attestation_unauthorized() {
        let (env, client, _owner) = create_contract();
        let unauthorized = Address::generate(&env);
        let subject = Address::generate(&env);
        
        let claim_hash = BytesN::from_array(&env, &[5; 32]);
        
        // Try to revoke without being a verifier (should panic)
        client.mock_all_auths().revoke_attestation(&subject, &claim_hash);
    }

    #[test]
    fn test_multiple_attestations() {
        let (env, client, owner) = create_contract();
        let verifier1 = Address::generate(&env);
        let verifier2 = Address::generate(&env);
        let subject = Address::generate(&env);
        
        // Add verifiers
        client.add_verifier(&verifier1);
        client.add_verifier(&verifier2);
        
        // Create multiple attestations
        let claim_hash1 = BytesN::from_array(&env, &[6; 32]);
        let claim_hash2 = BytesN::from_array(&env, &[7; 32]);
        let claim_hash3 = BytesN::from_array(&env, &[8; 32]);
        
        client.mock_all_auths().attest(&subject, &claim_hash1);
        client.mock_all_auths().attest(&subject, &claim_hash2);
        client.mock_all_auths().attest(&subject, &claim_hash3);
        
        // Verify all attestations
        assert!(client.is_attested(&subject, &claim_hash1));
        assert!(client.is_attested(&subject, &claim_hash2));
        assert!(client.is_attested(&subject, &claim_hash3));
        
        // Check attestations list
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 3);
        
        // Revoke one attestation
        client.mock_all_auths().revoke_attestation(&subject, &claim_hash1);
        
        // Verify partial revocation
        assert!(!client.is_attested(&subject, &claim_hash1));
        assert!(client.is_attested(&subject, &claim_hash2));
        assert!(client.is_attested(&subject, &claim_hash3));
        
        // Check updated attestations list
        let attestations = client.get_attestations(&subject);
        assert_eq!(attestations.len(), 2);
    }

    #[test]
    fn test_identity_record_persistence() {
        let (env, client, _owner) = create_contract();
        let subject1 = Address::generate(&env);
        let subject2 = Address::generate(&env);
        
        let hash1 = BytesN::from_array(&env, &[9; 32]);
        let hash2 = BytesN::from_array(&env, &[10; 32]);
        let meta1 = String::from_str(&env, "Doctor License");
        let meta2 = String::from_str(&env, "Clinic Registration");
        
        // Register multiple identities
        client.register_identity_hash(&hash1, &subject1, &meta1);
        client.register_identity_hash(&hash2, &subject2, &meta2);
        
        // Verify both are stored correctly
        assert_eq!(client.get_identity_hash(&subject1), Some(hash1));
        assert_eq!(client.get_identity_meta(&subject1), Some(meta1));
        assert_eq!(client.get_identity_hash(&subject2), Some(hash2));
        assert_eq!(client.get_identity_meta(&subject2), Some(meta2));
        
        // Verify non-existent subject returns None
        let non_existent = Address::generate(&env);
        assert_eq!(client.get_identity_hash(&non_existent), None);
        assert_eq!(client.get_identity_meta(&non_existent), None);
    }
}