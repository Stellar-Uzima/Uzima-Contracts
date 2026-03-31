#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::{Address as AddressTestUtils, Signature};
    use soroban_sdk::{Address, Env, Vec};

    use crate::types::{Role, RBACConfig};
    use crate::RBAC;

    fn create_test_env() -> Env {
        Env::default()
    }

    fn setup_contract(env: &Env) -> Address {
        let admin = Address::random(env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };

        RBAC::initialize(env.clone(), admin.clone(), config);
        admin
    }

    #[test]
    fn test_initialize() {
        let env = create_test_env();
        let admin = Address::random(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };

        RBAC::initialize(env.clone(), admin.clone(), config.clone());

        let stored_config = RBAC::get_config(env.clone());
        assert_eq!(stored_config.emit_events, true);
        assert_eq!(stored_config.max_roles_per_address, 10);
    }

    #[test]
    #[should_panic(expected = "Contract already initialized")]
    fn test_initialize_twice_fails() {
        let env = create_test_env();
        let admin = Address::random(&env);
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 10,
        };

        RBAC::initialize(env.clone(), admin.clone(), config.clone());
        RBAC::initialize(env.clone(), admin.clone(), config.clone());
    }

    #[test]
    fn test_assign_role() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        let success = RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        assert_eq!(success, true);

        // Verify role was assigned
        let has_role = RBAC::has_role(env.clone(), user.clone(), Role::Doctor);
        assert_eq!(has_role, true);
    }

    #[test]
    fn test_assign_same_role_twice() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Assign role first time
        let success1 = RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        assert_eq!(success1, true);

        env.mock_auths(&[Signature::Invoker]);

        // Try to assign same role again
        let success2 = RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        assert_eq!(success2, false); // Should fail
    }

    #[test]
    fn test_remove_role() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Assign role
        RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);

        // Verify role exists
        assert_eq!(RBAC::has_role(env.clone(), user.clone(), Role::Doctor), true);

        env.mock_auths(&[Signature::Invoker]);

        // Remove role
        let success = RBAC::remove_role(env.clone(), user.clone(), Role::Doctor);
        assert_eq!(success, true);

        // Verify role no longer exists
        assert_eq!(RBAC::has_role(env.clone(), user.clone(), Role::Doctor), false);
    }

    #[test]
    fn test_remove_nonexistent_role() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Try to remove role that was never assigned
        let success = RBAC::remove_role(env.clone(), user.clone(), Role::Doctor);
        assert_eq!(success, false);
    }

    #[test]
    fn test_get_roles() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Assign multiple roles
        RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user.clone(), Role::Patient);
        RBAC::assign_role(env.clone(), user.clone(), Role::Staff);

        // Get all roles
        let roles = RBAC::get_roles(env.clone(), user.clone());

        assert_eq!(roles.len(), 3);
        assert!(roles.iter().any(|r| *r == Role::Doctor));
        assert!(roles.iter().any(|r| *r == Role::Patient));
        assert!(roles.iter().any(|r| *r == Role::Staff));
    }

    #[test]
    fn test_get_roles_empty() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        let roles = RBAC::get_roles(env.clone(), user.clone());
        assert_eq!(roles.len(), 0);
    }

    #[test]
    fn test_has_any_role() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user.clone(), Role::Patient);

        // Create vector with roles to check
        let mut roles_to_check = Vec::with_capacity(&env, 2);
        roles_to_check.push_back(Role::Admin);
        roles_to_check.push_back(Role::Doctor);

        // Should return true because user has Doctor role
        let has_any = RBAC::has_any_role(env.clone(), user.clone(), roles_to_check);
        assert_eq!(has_any, true);

        // Create vector with only Admin role
        let mut admin_only = Vec::with_capacity(&env, 1);
        admin_only.push_back(Role::Admin);

        // Should return false because user doesn't have Admin role
        let has_admin = RBAC::has_any_role(env.clone(), user.clone(), admin_only);
        assert_eq!(has_admin, false);
    }

    #[test]
    fn test_has_all_roles() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user.clone(), Role::Patient);

        // Create vector with roles user has
        let mut roles_user_has = Vec::with_capacity(&env, 2);
        roles_user_has.push_back(Role::Doctor);
        roles_user_has.push_back(Role::Patient);

        // Should return true
        let has_all = RBAC::has_all_roles(env.clone(), user.clone(), roles_user_has.clone());
        assert_eq!(has_all, true);

        // Create vector with one role user has and one they don't
        let mut mixed_roles = Vec::with_capacity(&env, 2);
        mixed_roles.push_back(Role::Doctor);
        mixed_roles.push_back(Role::Admin);

        // Should return false
        let has_all_mixed = RBAC::has_all_roles(env.clone(), user.clone(), mixed_roles);
        assert_eq!(has_all_mixed, false);
    }

    #[test]
    fn test_get_address_roles() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user.clone(), Role::Researcher);

        let address_roles = RBAC::get_address_roles(env.clone(), user.clone());

        assert_eq!(address_roles.address, user);
        assert_eq!(address_roles.role_count, 2);
        assert_eq!(address_roles.roles.len(), 2);
    }

    #[test]
    fn test_get_role_members() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user1 = Address::random(&env);
        let user2 = Address::random(&env);
        let user3 = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Assign Doctor role to multiple users
        RBAC::assign_role(env.clone(), user1.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user2.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user3.clone(), Role::Patient);

        // Get doctors
        let doctors = RBAC::get_role_members(env.clone(), Role::Doctor);
        assert_eq!(doctors.len(), 2);

        // Get patients
        let patients = RBAC::get_role_members(env.clone(), Role::Patient);
        assert_eq!(patients.len(), 1);
    }

    #[test]
    fn test_get_role_member_count() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user1 = Address::random(&env);
        let user2 = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        RBAC::assign_role(env.clone(), user1.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user2.clone(), Role::Doctor);

        let count = RBAC::get_role_member_count(env.clone(), Role::Doctor);
        assert_eq!(count, 2);

        let patient_count = RBAC::get_role_member_count(env.clone(), Role::Patient);
        assert_eq!(patient_count, 0);
    }

    #[test]
    fn test_is_doctor() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::is_doctor(env.clone(), user.clone()), false);

        RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);

        assert_eq!(RBAC::is_doctor(env.clone(), user.clone()), true);
    }

    #[test]
    fn test_is_patient() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::is_patient(env.clone(), user.clone()), false);

        RBAC::assign_role(env.clone(), user.clone(), Role::Patient);

        assert_eq!(RBAC::is_patient(env.clone(), user.clone()), true);
    }

    #[test]
    fn test_is_admin() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        // Admin shouldn't have Admin role yet (different address)
        assert_eq!(RBAC::is_admin(env.clone(), user.clone()), false);

        env.mock_auths(&[Signature::Invoker]);

        RBAC::assign_role(env.clone(), user.clone(), Role::Admin);

        assert_eq!(RBAC::is_admin(env.clone(), user.clone()), true);
    }

    #[test]
    fn test_is_staff() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::is_staff(env.clone(), user.clone()), false);

        RBAC::assign_role(env.clone(), user.clone(), Role::Staff);

        assert_eq!(RBAC::is_staff(env.clone(), user.clone()), true);
    }

    #[test]
    fn test_multiple_roles_and_removals() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Assign multiple roles
        RBAC::assign_role(env.clone(), user.clone(), Role::Doctor);
        RBAC::assign_role(env.clone(), user.clone(), Role::Patient);
        RBAC::assign_role(env.clone(), user.clone(), Role::Researcher);

        // Verify all roles exist
        assert_eq!(RBAC::get_roles(env.clone(), user.clone()).len(), 3);

        env.mock_auths(&[Signature::Invoker]);

        // Remove one role
        RBAC::remove_role(env.clone(), user.clone(), Role::Patient);

        // Verify 2 roles remain
        let remaining = RBAC::get_roles(env.clone(), user.clone());
        assert_eq!(remaining.len(), 2);
        assert!(!remaining.iter().any(|r| *r == Role::Patient));

        env.mock_auths(&[Signature::Invoker]);

        // Remove another
        RBAC::remove_role(env.clone(), user.clone(), Role::Doctor);

        let final_roles = RBAC::get_roles(env.clone(), user.clone());
        assert_eq!(final_roles.len(), 1);
        assert_eq!(final_roles.get_unchecked(0), Role::Researcher);
    }

    #[test]
    fn test_update_config() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let new_config = RBACConfig {
            emit_events: false,
            max_roles_per_address: 5,
        };

        env.mock_auths(&[Signature::Invoker]);

        RBAC::update_config(env.clone(), new_config.clone());

        let stored_config = RBAC::get_config(env.clone());
        assert_eq!(stored_config.emit_events, false);
        assert_eq!(stored_config.max_roles_per_address, 5);
    }

    #[test]
    fn test_max_roles_per_address() {
        let env = create_test_env();
        let admin = Address::random(&env);
        
        let config = RBACConfig {
            emit_events: true,
            max_roles_per_address: 2,
        };

        RBAC::initialize(env.clone(), admin.clone(), config);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Assign first role
        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Doctor), true);

        // Assign second role
        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Patient), true);

        // Try to assign third role (should fail due to max limit)
        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Staff), false);

        // Verify only 2 roles assigned
        assert_eq!(RBAC::get_roles(env.clone(), user.clone()).len(), 2);
    }

    #[test]
    fn test_all_role_types() {
        let env = create_test_env();
        let admin = setup_contract(&env);

        let user = Address::random(&env);

        env.mock_auths(&[Signature::Invoker]);

        // Test all role types can be assigned
        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Admin), true);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Doctor), true);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Patient), true);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Staff), true);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Insurer), true);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Researcher), true);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Auditor), true);

        env.mock_auths(&[Signature::Invoker]);

        assert_eq!(RBAC::assign_role(env.clone(), user.clone(), Role::Service), true);

        // Verify all roles assigned
        assert_eq!(RBAC::get_roles(env.clone(), user.clone()).len(), 8);
    }
}
