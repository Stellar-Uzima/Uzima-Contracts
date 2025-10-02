#[cfg(test)]
mod test {
    use crate::{PatientConsentToken, PatientConsentTokenClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

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

        let token_id = client.mint_consent(&patient, &metadata_uri, &consent_type, &0);

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

        let token_id = client.mint_consent(&patient, &metadata_uri, &consent_type, &0);
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

        let token_id = client.mint_consent(&patient, &metadata_uri, &consent_type, &0);
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

        let token_id = client.mint_consent(&patient, &metadata_uri, &consent_type, &0);

        let new_uri = String::from_str(&env, "ipfs://QmYyy...");
        client.update_consent(&token_id, &new_uri);

        let metadata = client.get_metadata(&token_id);
        assert_eq!(metadata.version, 2);
        assert_eq!(metadata.metadata_uri, new_uri);
    }
}
