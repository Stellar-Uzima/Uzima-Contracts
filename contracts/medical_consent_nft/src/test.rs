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
}
