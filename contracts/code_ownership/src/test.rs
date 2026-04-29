#[cfg(test)]
mod tests {
    use crate::{CodeOwnership, CodeOwnershipClient};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        let result = client.initialize(&admin);
        assert_eq!(result, ());
    }

    #[test]
    #[should_panic]
    fn test_initialize_already_initialized() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        client.initialize(&admin);
        client.initialize(&admin);
    }

    #[test]
    fn test_register_module() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        client.initialize(&admin);

        let module_id = String::from_str(&env, "aml");
        let module_name = String::from_str(&env, "Anti-Money Laundering");
        let expertise = Vec::from_slice(&env, &[String::from_str(&env, "compliance")]);

        let result = client.register_module(
            &admin,
            &module_id,
            &module_name,
            &owner,
            &Vec::new(&env),
            &expertise,
        );

        assert_eq!(result, ());
    }

    #[test]
    #[should_panic]
    fn test_register_duplicate_module() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        client.initialize(&admin);

        let module_id = String::from_str(&env, "aml");
        let module_name = String::from_str(&env, "Anti-Money Laundering");
        let expertise = Vec::from_slice(&env, &[String::from_str(&env, "compliance")]);

        client.register_module(
            &admin,
            &module_id,
            &module_name,
            &owner,
            &Vec::new(&env),
            &expertise,
        );

        client.register_module(
            &admin,
            &module_id,
            &module_name,
            &owner,
            &Vec::new(&env),
            &expertise,
        );
    }

    #[test]
    fn test_update_module_ownership() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        client.initialize(&admin);

        let module_id = String::from_str(&env, "aml");
        let module_name = String::from_str(&env, "Anti-Money Laundering");
        let expertise = Vec::from_slice(&env, &[String::from_str(&env, "compliance")]);

        client.register_module(
            &admin,
            &module_id,
            &module_name,
            &owner1,
            &Vec::new(&env),
            &expertise,
        );

        let result = client.update_module_ownership(&admin, &module_id, &owner2, &Vec::new(&env));

        assert_eq!(result, ());

        let ownership = client.get_module_ownership(&module_id);
        assert_eq!(ownership.primary_owner, owner2);
    }

    #[test]
    fn test_configure_review_route() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let escalation_owner = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        client.initialize(&admin);

        let module_id = String::from_str(&env, "aml");
        let module_name = String::from_str(&env, "Anti-Money Laundering");
        let expertise = Vec::from_slice(&env, &[String::from_str(&env, "compliance")]);

        client.register_module(
            &admin,
            &module_id,
            &module_name,
            &owner,
            &Vec::new(&env),
            &expertise,
        );

        let result = client.configure_review_route(&admin, &module_id, &2, &5, &escalation_owner);

        assert_eq!(result, ());

        let route = client.get_review_route(&module_id);
        assert_eq!(route.required_reviewers, 2);
        assert_eq!(route.escalation_threshold, 5);
    }

    #[test]
    fn test_is_module_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let other = Address::generate(&env);
        let contract_id = env.register_contract(None, CodeOwnership);
        let client = CodeOwnershipClient::new(&env, &contract_id);

        client.initialize(&admin);

        let module_id = String::from_str(&env, "aml");
        let module_name = String::from_str(&env, "Anti-Money Laundering");
        let expertise = Vec::from_slice(&env, &[String::from_str(&env, "compliance")]);

        client.register_module(
            &admin,
            &module_id,
            &module_name,
            &owner,
            &Vec::new(&env),
            &expertise,
        );

        assert!(client.is_module_owner(&module_id, &owner));
        assert!(!client.is_module_owner(&module_id, &other));
    }
}
