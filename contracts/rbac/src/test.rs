#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{vec, Address, Env};

    use crate::types::{RBACConfig, Role};
    use crate::{RBACClient, RBAC};

    fn setup_contract(env: &Env) -> (RBACClient<'_>, Address) {
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };
        env.mock_all_auths();
        client.initialize(&admin, &config);
        (client, admin)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };

        env.mock_all_auths();
        client.initialize(&admin, &config);

        let stored_config = client.get_config();
        assert!(stored_config.emit_events);
        assert_eq!(stored_config.max_roles_per_address, 10);
    }

    #[test]
    #[should_panic]
    fn test_initialize_twice_fails() {
        let env = Env::default();
        let (client, admin) = setup_contract(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };
        client.initialize(&admin, &config);
    }

    #[test]
    fn test_assign_role() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        let success = client.assign_role(&user, &Role::Doctor);
        assert!(success);

        let has_role = client.has_role(&user, &Role::Doctor);
        assert!(has_role);
    }

    #[test]
    fn test_assign_same_role_twice() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        let success1 = client.assign_role(&user, &Role::Doctor);
        assert!(success1);

        let success2 = client.assign_role(&user, &Role::Doctor);
        assert!(!success2);
    }

    #[test]
    fn test_remove_role() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        assert!(client.has_role(&user, &Role::Doctor));

        let success = client.remove_role(&user, &Role::Doctor);
        assert!(success);

        assert!(!client.has_role(&user, &Role::Doctor));
    }

    #[test]
    fn test_remove_nonexistent_role() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        let success = client.remove_role(&user, &Role::Doctor);
        assert!(!success);
    }

    #[test]
    fn test_get_roles() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        client.assign_role(&user, &Role::Patient);
        client.assign_role(&user, &Role::Staff);

        let roles = client.get_roles(&user);

        assert_eq!(roles.len(), 3);
        assert!(roles.iter().any(|r| r == Role::Doctor));
        assert!(roles.iter().any(|r| r == Role::Patient));
        assert!(roles.iter().any(|r| r == Role::Staff));
    }

    #[test]
    fn test_get_roles_empty() {
        let env = Env::default();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        let roles = client.get_roles(&user);
        assert_eq!(roles.len(), 0);
    }

    #[test]
    fn test_has_any_role() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        client.assign_role(&user, &Role::Patient);

        let roles_to_check = vec![&env, Role::Admin, Role::Doctor];
        let has_any = client.has_any_role(&user, &roles_to_check);
        assert!(has_any);

        let admin_only = vec![&env, Role::Admin];
        let has_admin = client.has_any_role(&user, &admin_only);
        assert!(!has_admin);
    }

    #[test]
    fn test_has_all_roles() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        client.assign_role(&user, &Role::Patient);

        let roles_user_has = vec![&env, Role::Doctor, Role::Patient];
        let has_all = client.has_all_roles(&user, &roles_user_has);
        assert!(has_all);

        let mixed_roles = vec![&env, Role::Doctor, Role::Admin];
        let has_all_mixed = client.has_all_roles(&user, &mixed_roles);
        assert!(!has_all_mixed);
    }

    #[test]
    fn test_get_address_roles() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        client.assign_role(&user, &Role::Researcher);

        let address_roles = client.get_address_roles(&user);

        assert_eq!(address_roles.address, user);
        assert_eq!(address_roles.role_count, 2);
        assert_eq!(address_roles.roles.len(), 2);
    }

    #[test]
    fn test_get_role_members() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);

        client.assign_role(&user1, &Role::Doctor);
        client.assign_role(&user2, &Role::Doctor);
        client.assign_role(&user3, &Role::Patient);

        let doctors = client.get_role_members(&Role::Doctor);
        assert_eq!(doctors.len(), 2);

        let patients = client.get_role_members(&Role::Patient);
        assert_eq!(patients.len(), 1);
    }

    #[test]
    fn test_get_role_member_count() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.assign_role(&user1, &Role::Doctor);
        client.assign_role(&user2, &Role::Doctor);

        let count = client.get_role_member_count(&Role::Doctor);
        assert_eq!(count, 2);

        let patient_count = client.get_role_member_count(&Role::Patient);
        assert_eq!(patient_count, 0);
    }

    #[test]
    fn test_is_doctor() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        assert!(!client.is_doctor(&user));

        client.assign_role(&user, &Role::Doctor);

        assert!(client.is_doctor(&user));
    }

    #[test]
    fn test_is_patient() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        assert!(!client.is_patient(&user));

        client.assign_role(&user, &Role::Patient);

        assert!(client.is_patient(&user));
    }

    #[test]
    fn test_is_admin() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        assert!(!client.is_admin(&user));

        client.assign_role(&user, &Role::Admin);

        assert!(client.is_admin(&user));
    }

    #[test]
    fn test_is_staff() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        assert!(!client.is_staff(&user));

        client.assign_role(&user, &Role::Staff);

        assert!(client.is_staff(&user));
    }

    #[test]
    fn test_multiple_roles_and_removals() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        client.assign_role(&user, &Role::Patient);
        client.assign_role(&user, &Role::Researcher);

        assert_eq!(client.get_roles(&user).len(), 3);

        client.remove_role(&user, &Role::Patient);

        let remaining = client.get_roles(&user);
        assert_eq!(remaining.len(), 2);
        assert!(!remaining.iter().any(|r| r == Role::Patient));

        client.remove_role(&user, &Role::Doctor);

        let final_roles = client.get_roles(&user);
        assert_eq!(final_roles.len(), 1);
        assert_eq!(final_roles.get_unchecked(0), Role::Researcher);
    }

    #[test]
    fn test_update_config() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let new_config = RBACConfig {
            emit_events: false,
            max_roles_per_address: 5,
        };

        client.update_config(&new_config);

        let stored_config = client.get_config();
        assert!(!stored_config.emit_events);
        assert_eq!(stored_config.max_roles_per_address, 5);
    }

    #[test]
    fn test_max_roles_per_address() {
        let env = Env::default();
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 2,
        };

        env.mock_all_auths();
        client.initialize(&admin, &config);

        let user = Address::generate(&env);

        assert!(client.assign_role(&user, &Role::Doctor));
        assert!(client.assign_role(&user, &Role::Patient));
        assert!(!client.assign_role(&user, &Role::Staff));

        assert_eq!(client.get_roles(&user).len(), 2);
    }

    #[test]
    fn test_all_role_types() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin) = setup_contract(&env);

        let user = Address::generate(&env);

        assert!(client.assign_role(&user, &Role::Admin));
        assert!(client.assign_role(&user, &Role::Doctor));
        assert!(client.assign_role(&user, &Role::Patient));
        assert!(client.assign_role(&user, &Role::Staff));
        assert!(client.assign_role(&user, &Role::Insurer));
        assert!(client.assign_role(&user, &Role::Researcher));
        assert!(client.assign_role(&user, &Role::Auditor));
        assert!(client.assign_role(&user, &Role::Service));

        assert_eq!(client.get_roles(&user).len(), 8);
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(crate::errors::Error::Unauthorized as u32, 100);
        assert_eq!(crate::errors::Error::NotInitialized as u32, 300);
        assert_eq!(crate::errors::Error::AlreadyInitialized as u32, 301);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        use crate::errors::{get_suggestion, Error};
        use soroban_sdk::symbol_short;
        assert_eq!(
            get_suggestion(Error::Unauthorized),
            symbol_short!("CHK_AUTH")
        );
        assert_eq!(
            get_suggestion(Error::NotInitialized),
            symbol_short!("INIT_CTR")
        );
        assert_eq!(
            get_suggestion(Error::AlreadyInitialized),
            symbol_short!("ALREADY")
        );
    }
}
