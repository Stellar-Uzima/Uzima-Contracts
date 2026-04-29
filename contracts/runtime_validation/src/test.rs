#[cfg(test)]
mod tests {
    use crate::{RuntimeValidation, RuntimeValidationClient, ViolationType};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        let result = client.initialize(&admin);
        assert_eq!(result, ());
    }

    #[test]
    fn test_register_invariant() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "balance_check");
        let description = String::from_str(&env, "Balance must be non-negative");

        let result = client.register_invariant(&admin, &check_id, &description, &3);
        assert_eq!(result, ());
    }

    #[test]
    #[should_panic]
    fn test_register_invariant_invalid_severity() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "balance_check");
        let description = String::from_str(&env, "Balance must be non-negative");

        client.register_invariant(&admin, &check_id, &description, &5);
    }

    #[test]
    fn test_verify_invariant() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "balance_check");
        let description = String::from_str(&env, "Balance must be non-negative");

        client.register_invariant(&admin, &check_id, &description, &3);

        let result = client.verify_invariant(&check_id, &100, &0, &1000);
        assert!(result);
    }

    #[test]
    fn test_verify_invariant_violation() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "balance_check");
        let description = String::from_str(&env, "Balance must be non-negative");

        client.register_invariant(&admin, &check_id, &description, &3);

        let result = client.verify_invariant(&check_id, &-100, &0, &1000);
        assert!(!result);
    }

    #[test]
    fn test_register_state_check() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "state_check");
        let description = String::from_str(&env, "State must be active");
        let expected_state = String::from_str(&env, "active");

        let result = client.register_state_check(&admin, &check_id, &description, &expected_state);
        assert_eq!(result, ());
    }

    #[test]
    fn test_verify_state_consistency() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "state_check");
        let description = String::from_str(&env, "State must be active");
        let expected_state = String::from_str(&env, "active");

        client.register_state_check(&admin, &check_id, &description, &expected_state);

        let current_state = String::from_str(&env, "active");
        let result = client.verify_state_consistency(&check_id, &current_state);
        assert!(result);
    }

    #[test]
    fn test_register_permission_check() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "admin_check");
        let description = String::from_str(&env, "Must be admin");
        let required_role = String::from_str(&env, "admin");

        let result =
            client.register_permission_check(&admin, &check_id, &description, &required_role);
        assert_eq!(result, ());
    }

    #[test]
    fn test_verify_permission() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "admin_check");
        let description = String::from_str(&env, "Must be admin");
        let required_role = String::from_str(&env, "admin");

        client.register_permission_check(&admin, &check_id, &description, &required_role);

        let user_role = String::from_str(&env, "admin");
        let result = client.verify_permission(&check_id, &user_role);
        assert!(result);
    }

    #[test]
    fn test_register_resource_tracker() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let tracker_id = String::from_str(&env, "memory_tracker");
        let resource_type = String::from_str(&env, "memory");

        let result = client.register_resource_tracker(&admin, &tracker_id, &resource_type, &1000);
        assert_eq!(result, ());
    }

    #[test]
    fn test_update_resource_usage() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let tracker_id = String::from_str(&env, "memory_tracker");
        let resource_type = String::from_str(&env, "memory");

        client.register_resource_tracker(&admin, &tracker_id, &resource_type, &1000);

        let result = client.update_resource_usage(&tracker_id, &100);
        assert_eq!(result, ());
    }

    #[test]
    fn test_report_violation() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let reporter = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "test_check");
        let details = String::from_str(&env, "Test violation");

        let result = client.report_violation(
            &reporter,
            &check_id,
            &ViolationType::InvariantViolation,
            &details,
        );
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_violation_count() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let reporter = Address::generate(&env);
        let contract_id = env.register_contract(None, RuntimeValidation);
        let client = RuntimeValidationClient::new(&env, &contract_id);

        client.initialize(&admin);

        let check_id = String::from_str(&env, "test_check");
        let details = String::from_str(&env, "Test violation");

        client.report_violation(
            &reporter,
            &check_id,
            &ViolationType::InvariantViolation,
            &details,
        );

        let count = client.get_violation_count();
        assert_eq!(count, 1);
    }
}
