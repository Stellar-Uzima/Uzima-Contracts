#[cfg(test)]
mod delegation_tests {
    use soroban_sdk::{
        testutils::Address as _,
        Address, Env, Symbol,
    };
    // Adjust this import to match your actual contract module path
    use crate::{AccessControlContract, AccessControlContractClient};

    // Helper: role symbols matching your contract's role definitions
    fn role_admin(env: &Env)     -> Symbol { Symbol::new(env, "admin")     }
    fn role_sub_admin(env: &Env) -> Symbol { Symbol::new(env, "sub_admin") }
    fn role_viewer(env: &Env)    -> Symbol { Symbol::new(env, "viewer")    }

    fn setup(env: &Env) -> (AccessControlContractClient, Address, Address, Address) {
        let contract_id = env.register_contract(None, AccessControlContract);
        let client      = AccessControlContractClient::new(env, &contract_id);

        let admin     = Address::generate(env);
        let sub_admin = Address::generate(env);
        let viewer    = Address::generate(env);

        env.mock_all_auths();
        client.initialize(&admin);

        (client, admin, sub_admin, viewer)
    }

    // ── Test 1: Admin can delegate sub-admin role ────────────────────────────

    #[test]
    fn test_admin_delegates_to_sub_admin() {
        let env = Env::default();
        let (client, admin, sub_admin, _) = setup(&env);

        env.mock_all_auths();
        client.grant_role(&admin, &sub_admin, &role_sub_admin(&env));

        assert!(
            client.has_role(&sub_admin, &role_sub_admin(&env)),
            "Sub-admin should have sub_admin role after delegation"
        );
    }

    // ── Test 2: Delegated roles cannot exceed parent permissions ─────────────

    #[test]
    fn test_delegated_role_cannot_exceed_parent_permissions() {
        let env = Env::default();
        let (client, admin, sub_admin, viewer) = setup(&env);

        env.mock_all_auths();
        // Admin grants sub_admin to sub_admin
        client.grant_role(&admin, &sub_admin, &role_sub_admin(&env));

        // sub_admin should NOT be able to grant admin role to viewer —
        // that would exceed their own permission level
        let result = std::panic::catch_unwind(|| {
            client.grant_role(&sub_admin, &viewer, &role_admin(&env));
        });
        assert!(
            result.is_err(),
            "Sub-admin must not be able to grant roles above their own level"
        );

        // But sub_admin CAN grant viewer (below their level)
        client.grant_role(&sub_admin, &viewer, &role_viewer(&env));
        assert!(client.has_role(&viewer, &role_viewer(&env)));
    }

    // ── Test 3: Role revocation is immediate ─────────────────────────────────

    #[test]
    fn test_role_revocation_is_immediate() {
        let env = Env::default();
        let (client, admin, sub_admin, _) = setup(&env);

        env.mock_all_auths();
        client.grant_role(&admin, &sub_admin, &role_sub_admin(&env));
        assert!(client.has_role(&sub_admin, &role_sub_admin(&env)));

        client.revoke_role(&admin, &sub_admin, &role_sub_admin(&env));
        assert!(
            !client.has_role(&sub_admin, &role_sub_admin(&env)),
            "Role should be removed immediately after revocation"
        );
    }

    // ── Test 4: Revocation cascades to delegated roles ───────────────────────

    #[test]
    fn test_revocation_cascades_to_delegated_roles() {
        let env = Env::default();
        let (client, admin, sub_admin, viewer) = setup(&env);

        env.mock_all_auths();
        // admin → sub_admin → viewer
        client.grant_role(&admin,     &sub_admin, &role_sub_admin(&env));
        client.grant_role(&sub_admin, &viewer,    &role_viewer(&env));

        // Revoke sub_admin's role — viewer's delegated role should also cascade
        client.revoke_role(&admin, &sub_admin, &role_sub_admin(&env));

        assert!(
            !client.has_role(&sub_admin, &role_sub_admin(&env)),
            "sub_admin role must be revoked"
        );
        // Depending on your contract's cascade implementation:
        assert!(
            !client.has_role(&viewer, &role_viewer(&env)),
            "viewer role delegated by sub_admin must also be revoked"
        );
    }

    // ── Test 5: Role hierarchy is enforced ───────────────────────────────────

    #[test]
    fn test_role_hierarchy_enforcement() {
        let env = Env::default();
        let (client, _, _, viewer) = setup(&env);

        // viewer should not be able to grant any role
        let result = std::panic::catch_unwind(|| {
            let other = Address::generate(&env);
            client.grant_role(&viewer, &other, &role_viewer(&env));
        });
        assert!(result.is_err(), "Viewer must not be able to grant roles");
    }

    // ── Test 6: Unauthorized role grant is rejected ───────────────────────────

    #[test]
    fn test_unauthorized_grant_rejected() {
        let env = Env::default();
        let (client, _, _, _) = setup(&env);

        let random  = Address::generate(&env);
        let target  = Address::generate(&env);

        let result = std::panic::catch_unwind(|| {
            client.grant_role(&random, &target, &role_sub_admin(&env));
        });
        assert!(result.is_err(), "Random address must not be able to grant roles");
    }
}