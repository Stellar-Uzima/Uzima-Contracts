#[cfg(test)]
mod test {
    use crate::{
        AccessCondition, AccessControl, DataType, GranularPermissions, PatientConsentToken,
        PatientConsentTokenClient, PermissionLevel,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, Map, String, Vec};

    #[test]
    fn test_initialize_and_add_issuer() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        assert!(client.is_issuer(&issuer));
    }

    #[test]
    fn test_mint_consent() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");

        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        assert_eq!(token_id, 0);
        assert_eq!(client.owner_of(&token_id), patient);
        assert!(!client.is_revoked(&token_id));
    }

    #[test]
    fn test_revoke_consent() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "research");

        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
        client.revoke_consent(&token_id);

        assert!(client.is_revoked(&token_id));
        assert!(!client.is_valid(&token_id));
    }

    #[test]
    #[should_panic]
    fn test_transfer_revoked_fails() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");

        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
        client.revoke_consent(&token_id);
        client.transfer(&patient, &recipient, &token_id);
    }

    #[test]
    fn test_update_metadata() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");

        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        let new_uri = String::from_str(&env, "ipfs://QmYyy...");
        client.update_consent(&token_id, &new_uri);

        let metadata = client.get_metadata(&token_id);
        assert_eq!(metadata.version, 2);
        assert_eq!(metadata.metadata_uri, new_uri);
    }

    // Advanced feature tests

    #[test]
    fn test_granular_permissions() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");
        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        // Set granular permissions
        let mut permissions_map = Map::new(&env);
        permissions_map.set(DataType::LabResults, PermissionLevel::Read);
        permissions_map.set(DataType::MedicalHistory, PermissionLevel::Write);

        let permissions = GranularPermissions {
            permissions: permissions_map,
        };

        client.set_granular_permissions(&patient, &token_id, &permissions);

        let retrieved_permissions = client.get_granular_permissions(&token_id).unwrap();
        assert_eq!(
            retrieved_permissions
                .permissions
                .get(DataType::LabResults)
                .unwrap(),
            PermissionLevel::Read
        );
    }

    #[test]
    fn test_access_controls() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");
        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        // Set access controls with time window
        let current_time = env.ledger().timestamp();
        let mut conditions = Vec::new(&env);
        conditions.push_back(AccessCondition::TimeWindow(
            current_time,
            current_time + 86400, // 1 day
        ));

        let access_control = AccessControl {
            conditions,
            max_access_count: 10,
            current_access_count: 0,
            last_access_timestamp: 0,
        };

        client.set_access_controls(&token_id, &access_control);

        let requester = Address::generate(&env);
        let allowed = client.check_access_allowed(&token_id, &requester).unwrap();
        assert!(allowed);
    }

    #[test]
    fn test_delegation() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);
        let delegate = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");
        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        // Create permissions for delegation
        let mut permissions_map = Map::new(&env);
        permissions_map.set(DataType::LabResults, PermissionLevel::Read);
        let permissions = GranularPermissions {
            permissions: permissions_map,
        };

        let expiry = env.ledger().timestamp() + 86400; // 1 day
        client.delegate_consent(&token_id, &delegate, &permissions, &expiry);

        let delegations = client.get_delegations(&token_id);
        assert_eq!(delegations.len(), 1);
        assert_eq!(delegations.get(0).unwrap().delegate, delegate);
    }

    #[test]
    fn test_emergency_override() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);
        let emergency_auth = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);
        client.add_emergency_authority(&emergency_auth);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");
        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        let reason = String::from_str(&env, "Life-threatening emergency");
        let override_id = client
            .emergency_override(&emergency_auth, &token_id, &reason, &0)
            .unwrap();

        assert!(override_id >= 0);
    }

    #[test]
    fn test_dynamic_consent_update() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");
        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        // Enable dynamic updates
        client.enable_dynamic_updates(&token_id);

        let new_uri = String::from_str(&env, "ipfs://QmZzz...");
        let change_summary = String::from_str(&env, "Updated treatment plan");
        client.update_consent_dynamic(&patient, &token_id, &new_uri, &change_summary);

        let version_history = client.get_version_history(&token_id);
        assert_eq!(version_history.len(), 1);
        assert_eq!(version_history.get(0).unwrap().version, 1);
    }

    #[test]
    fn test_analytics() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");
        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        let analytics = client.get_analytics();
        assert_eq!(analytics.total_consents, 1);
        assert_eq!(analytics.active_consents, 1);
    }

    #[test]
    fn test_consent_report() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PatientConsentToken);
        let client = PatientConsentTokenClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.add_issuer(&issuer);

        let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
        let consent_type = String::from_str(&env, "treatment");
        let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);

        let report = client.generate_consent_report(&patient);
        assert_eq!(report.len(), 1);
        assert_eq!(report.get(0).unwrap(), token_id);
    }

    // ── Property-Based Tests (Issue #832) ─────────────────────────

    // Property 1: Token counter monotonicity
    #[test]
    fn proptest_token_counter_monotonicity() {
        use proptest::proptest;
        proptest!(|(token_count in 1usize..=50)| {
            let env = Env::default();
            let contract_id = env.register_contract(None, PatientConsentToken);
            let client = PatientConsentTokenClient::new(&env, &contract_id);

            let admin = Address::generate(&env);
            let issuer = Address::generate(&env);
            let patient = Address::generate(&env);

            client.initialize(&admin);
            client.add_issuer(&issuer);

            let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
            let consent_type = String::from_str(&env, "treatment");
            
            for i in 0..token_count {
                let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
                prop_assert_eq!(token_id as usize, i,
                    "Token ID must equal iteration index (0-indexed)");
            }
        });
    }

    // Property 2: Can't revoke token twice
    #[test]
    fn proptest_revoke_idempotency() {
        use proptest::proptest;
        proptest!(|(seed in 1u64..=1000)| {
            let env = Env::default();
            let contract_id = env.register_contract(None, PatientConsentToken);
            let client = PatientConsentTokenClient::new(&env, &contract_id);

            let admin = Address::generate(&env);
            let issuer = Address::generate(&env);
            let patient = Address::generate(&env);

            client.initialize(&admin);
            client.add_issuer(&issuer);

            let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
            let consent_type = String::from_str(&env, "treatment");
            let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
            
            client.revoke_consent(&token_id);
            
            // Second revoke should panic or fail (idempotent protection)
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                client.revoke_consent(&token_id);
            }));
            prop_assert!(result.is_err(), "Revoking already revoked token should fail at seed {}", seed);
        });
    }

    // Property 3: Revoked tokens are not queryable
    #[test]
    fn proptest_revoked_token_not_queryable() {
        use proptest::proptest;
        proptest!(|(seed in 1u64..=1000)| {
            let env = Env::default();
            let contract_id = env.register_contract(None, PatientConsentToken);
            let client = PatientConsentTokenClient::new(&env, &contract_id);

            let admin = Address::generate(&env);
            let issuer = Address::generate(&env);
            let patient = Address::generate(&env);

            client.initialize(&admin);
            client.add_issuer(&issuer);

            let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
            let consent_type = String::from_str(&env, "treatment");
            let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
            
            // Before revoke - should be valid
            let valid_before = client.is_valid(&token_id);
            prop_assert!(valid_before, "Token should be valid before revoke at seed {}", seed);
            
            client.revoke_consent(&token_id);
            
            // After revoke - should not be valid
            let valid_after = client.is_valid(&token_id);
            prop_assert!(!valid_after, "Revoked token should not be valid at seed {}", seed);
            
            let is_revoked = client.is_revoked(&token_id);
            prop_assert!(is_revoked, "Token should be marked as revoked at seed {}", seed);
        });
    }

    // Property 4: Token ownership is persistent
    #[test]
    fn proptest_token_ownership_persistent() {
        use proptest::proptest;
        proptest!(|(check_count in 1usize..=20)| {
            let env = Env::default();
            let contract_id = env.register_contract(None, PatientConsentToken);
            let client = PatientConsentTokenClient::new(&env, &contract_id);

            let admin = Address::generate(&env);
            let issuer = Address::generate(&env);
            let patient = Address::generate(&env);

            client.initialize(&admin);
            client.add_issuer(&issuer);

            let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
            let consent_type = String::from_str(&env, "treatment");
            let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
            
            // Multiple checks should always return the same owner
            for _ in 0..check_count {
                let owner = client.owner_of(&token_id);
                prop_assert_eq!(owner, patient,
                    "Token owner must remain the same across multiple queries");
            }
        });
    }

    // Property 5: Version history increases monotonically
    #[test]
    fn proptest_version_history_monotonicity() {
        use proptest::proptest;
        proptest!(|(update_count in 1usize..=20)| {
            let env = Env::default();
            let contract_id = env.register_contract(None, PatientConsentToken);
            let client = PatientConsentTokenClient::new(&env, &contract_id);

            let admin = Address::generate(&env);
            let issuer = Address::generate(&env);
            let patient = Address::generate(&env);

            client.initialize(&admin);
            client.add_issuer(&issuer);

            let metadata_uri = String::from_str(&env, "ipfs://QmXxx...");
            let consent_type = String::from_str(&env, "treatment");
            let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
            
            let mut prev_version = 0u32;
            for i in 0..update_count {
                let new_uri = String::from_str(&env, 
                    &format!("ipfs://version{}", i));
                client.update_metadata(&token_id, &new_uri, &consent_type);
                
                let updated_consent = client.get_consent(&token_id);
                let version = updated_consent.version;
                prop_assert!(version > prev_version,
                    "Version must increase with each update (prev: {}, current: {})", 
                    prev_version, version);
                prev_version = version;
            }
        });
    }

    // Property 6: Multiple tokens for same patient
    #[test]
    fn proptest_multiple_tokens_same_patient() {
        use proptest::proptest;
        proptest!(|(token_count in 1usize..=30)| {
            let env = Env::default();
            let contract_id = env.register_contract(None, PatientConsentToken);
            let client = PatientConsentTokenClient::new(&env, &contract_id);

            let admin = Address::generate(&env);
            let issuer = Address::generate(&env);
            let patient = Address::generate(&env);

            client.initialize(&admin);
            client.add_issuer(&issuer);

            let mut token_ids = Vec::new();
            for i in 0..token_count {
                let metadata_uri = String::from_str(&env, 
                    &format!("ipfs://QmXxx{}", i));
                let consent_type = String::from_str(&env, "treatment");
                let token_id = client.mint_consent(&issuer, &patient, &metadata_uri, &consent_type, &0);
                token_ids.push(token_id);
            }

            // All tokens should have same owner
            let analytics = client.get_analytics();
            prop_assert_eq!(analytics.total_consents as usize, token_count,
                "Total consents must equal number of minted tokens");
            
            // Verify each token owner
            for token_id in token_ids {
                let owner = client.owner_of(&token_id);
                prop_assert_eq!(owner, patient,
                    "All tokens must have same owner");
            }
        });
    }
}
