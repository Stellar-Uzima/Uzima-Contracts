#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, Vec};

    use crate::types::{RBACConfig, Role};
    use crate::{RBACClient, RBAC};

    fn create_test_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    fn setup_contract(env: &Env) -> (RBACClient, Address) {
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };
        client.initialize(&admin, &config);
        (client, admin)
    }

    #[test]
    fn test_initialize() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };

        client.initialize(&admin, &config);

        let stored_config = client.get_config();
        assert_eq!(stored_config.emit_events, true);
        assert_eq!(stored_config.max_roles_per_address, 10);
    }

    #[test]
    #[should_panic(expected = "Contract already initialized")]
    fn test_initialize_twice_fails() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };

        client.initialize(&admin, &config);
        client.initialize(&admin, &config);
    }

    #[test]
    fn test_assign_role() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        let success = client.assign_role(&user, &Role::Doctor);
        assert_eq!(success, true);

        let has_role = client.has_role(&user, &Role::Doctor);
        assert_eq!(has_role, true);
    }

    #[test]
    fn test_assign_same_role_twice() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        let success1 = client.assign_role(&user, &Role::Doctor);
        assert_eq!(success1, true);

        let success2 = client.assign_role(&user, &Role::Doctor);
        assert_eq!(success2, false);
    }

    #[test]
    fn test_remove_role() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        assert_eq!(client.has_role(&user, &Role::Doctor), true);

        let success = client.remove_role(&user, &Role::Doctor);
        assert_eq!(success, true);
        assert_eq!(client.has_role(&user, &Role::Doctor), false);
    }

    #[test]
    fn test_remove_nonexistent_role() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        let success = client.remove_role(&user, &Role::Doctor);
        assert_eq!(success, false);
    }

    #[test]
    fn test_get_roles() {
        let env = create_test_env();
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
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        let roles = client.get_roles(&user);
        assert_eq!(roles.len(), 0);
    }

    #[test]
    fn test_has_any_role() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        client.assign_role(&user, &Role::Patient);

        let mut roles_to_check = Vec::new(&env);
        roles_to_check.push_back(Role::Admin);
        roles_to_check.push_back(Role::Doctor);
        assert_eq!(client.has_any_role(&user, &roles_to_check), true);

        let mut admin_only = Vec::new(&env);
        admin_only.push_back(Role::Admin);
        assert_eq!(client.has_any_role(&user, &admin_only), false);
    }

    #[test]
    fn test_has_all_roles() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        client.assign_role(&user, &Role::Doctor);
        client.assign_role(&user, &Role::Patient);

        let mut roles_user_has = Vec::new(&env);
        roles_user_has.push_back(Role::Doctor);
        roles_user_has.push_back(Role::Patient);
        assert_eq!(client.has_all_roles(&user, &roles_user_has), true);

        let mut mixed_roles = Vec::new(&env);
        mixed_roles.push_back(Role::Doctor);
        mixed_roles.push_back(Role::Admin);
        assert_eq!(client.has_all_roles(&user, &mixed_roles), false);
    }

    #[test]
    fn test_get_address_roles() {
        let env = create_test_env();
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
        let env = create_test_env();
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
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        client.assign_role(&user1, &Role::Doctor);
        client.assign_role(&user2, &Role::Doctor);

        assert_eq!(client.get_role_member_count(&Role::Doctor), 2);
        assert_eq!(client.get_role_member_count(&Role::Patient), 0);
    }

    #[test]
    fn test_is_doctor() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        assert_eq!(client.is_doctor(&user), false);
        client.assign_role(&user, &Role::Doctor);
        assert_eq!(client.is_doctor(&user), true);
    }

    #[test]
    fn test_is_patient() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        assert_eq!(client.is_patient(&user), false);
        client.assign_role(&user, &Role::Patient);
        assert_eq!(client.is_patient(&user), true);
    }

    #[test]
    fn test_is_admin() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        assert_eq!(client.is_admin(&user), false);
        client.assign_role(&user, &Role::Admin);
        assert_eq!(client.is_admin(&user), true);
    }

    #[test]
    fn test_is_staff() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        assert_eq!(client.is_staff(&user), false);
        client.assign_role(&user, &Role::Staff);
        assert_eq!(client.is_staff(&user), true);
    }

    #[test]
    fn test_multiple_roles_and_removals() {
        let env = create_test_env();
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
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);

        let new_config = RBACConfig {
            emit_events: false,
            max_roles_per_address: 5,
        };

        client.update_config(&new_config);

        let stored_config = client.get_config();
        assert_eq!(stored_config.emit_events, false);
        assert_eq!(stored_config.max_roles_per_address, 5);
    }

    #[test]
    fn test_max_roles_per_address() {
        let env = create_test_env();
        let contract_id = env.register_contract(None, RBAC);
        let client = RBACClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 2,
        };
        client.initialize(&admin, &config);

        let user = Address::generate(&env);

        assert_eq!(client.assign_role(&user, &Role::Doctor), true);
        assert_eq!(client.assign_role(&user, &Role::Patient), true);
        assert_eq!(client.assign_role(&user, &Role::Staff), false);
        assert_eq!(client.get_roles(&user).len(), 2);
    }

    #[test]
    fn test_all_role_types() {
        let env = create_test_env();
        let (client, _admin) = setup_contract(&env);
        let user = Address::generate(&env);

        assert_eq!(client.assign_role(&user, &Role::Admin), true);
        assert_eq!(client.assign_role(&user, &Role::Doctor), true);
        assert_eq!(client.assign_role(&user, &Role::Patient), true);
        assert_eq!(client.assign_role(&user, &Role::Staff), true);
        assert_eq!(client.assign_role(&user, &Role::Insurer), true);
        assert_eq!(client.assign_role(&user, &Role::Researcher), true);
        assert_eq!(client.assign_role(&user, &Role::Auditor), true);
        assert_eq!(client.assign_role(&user, &Role::Service), true);

        assert_eq!(client.get_roles(&user).len(), 8);
    }
}
