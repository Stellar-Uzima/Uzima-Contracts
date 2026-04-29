#[cfg(test)]
mod tests {
    use crate::{DeprecationFramework, DeprecationFrameworkClient, DeprecationPhase};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        let result = client.initialize(&admin);
        assert_eq!(result, ());
    }

    #[test]
    fn test_mark_for_deprecation() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        let result = client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );
        assert_eq!(result, ());
    }

    #[test]
    #[should_panic]
    fn test_mark_duplicate_deprecation() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );
    }

    #[test]
    fn test_set_sunset_timeline() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        let result =
            client.set_sunset_timeline(&admin, &contract_to_deprecate, &1000, &2000, &3000);
        assert_eq!(result, ());
    }

    #[test]
    #[should_panic]
    fn test_set_invalid_sunset_timeline() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        // Invalid: dates not in chronological order
        client.set_sunset_timeline(&admin, &contract_to_deprecate, &3000, &2000, &1000);
    }

    #[test]
    fn test_add_migration_guide() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        let guide_title = String::from_str(&env, "Migration Guide");
        let guide_content = String::from_str(&env, "Follow these steps...");
        let examples = Vec::new(&env);

        let result = client.add_migration_guide(
            &admin,
            &contract_to_deprecate,
            &guide_title,
            &guide_content,
            &examples,
        );
        assert_eq!(result, ());
    }

    #[test]
    fn test_update_deprecation_phase() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        let result = client.update_deprecation_phase(
            &admin,
            &contract_to_deprecate,
            &DeprecationPhase::Supported,
        );
        assert_eq!(result, ());
    }

    #[test]
    fn test_publish_user_communication() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        let message = String::from_str(&env, "This contract is deprecated");
        let comm_type = String::from_str(&env, "announcement");

        let result =
            client.publish_user_communication(&admin, &contract_to_deprecate, &message, &comm_type);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_create_removal_checklist() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        let mut items = Vec::new(&env);
        items.push_back(String::from_str(&env, "Migrate data"));
        items.push_back(String::from_str(&env, "Update documentation"));

        let result = client.create_removal_checklist(&admin, &contract_to_deprecate, &items);
        assert_eq!(result, ());
    }

    #[test]
    fn test_is_deprecated() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, DeprecationFramework);
        let client = DeprecationFrameworkClient::new(&env, &contract_id);

        client.initialize(&admin);

        let contract_to_deprecate = String::from_str(&env, "old_contract");
        let contract_name = String::from_str(&env, "Old Contract");
        let reason = String::from_str(&env, "Replaced by new version");

        assert!(!client.is_deprecated(&contract_to_deprecate));

        client.mark_for_deprecation(
            &admin,
            &contract_to_deprecate,
            &contract_name,
            &reason,
            &None,
        );

        assert!(client.is_deprecated(&contract_to_deprecate));
    }
}
