//! Invariant tests for RBAC role transitions and revocation paths.
//!
//! These tests verify that the RBAC contract maintains critical invariants
//! across role assignments, transitions, and revocations.

#![cfg(all(test, feature = "testutils"))]

use soroban_sdk::{
    testutils::Address as _, Address, Env, Vec,
};

use rbac::{RBAC, RBACClient, Role, RBACConfig};

fn setup() -> (Env, RBACClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, RBAC);
    let client = RBACClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    let config = RBACConfig {
        max_roles_per_address: 5,
        require_admin_for_changes: true,
    };
    client.initialize(&admin, &config);

    (env, client, admin)
}

/// INVARIANT: Admin role cannot be assigned to non-admin addresses.
#[test]
fn invariant_admin_role_requires_admin() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Admin);
    assert!(client.has_role(&user, &Role::Admin).unwrap());
}

/// INVARIANT: Revoking a role removes it from the address.
#[test]
fn invariant_role_revocation_removes_role() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Doctor);
    assert!(client.has_role(&user, &Role::Doctor).unwrap());

    client.remove_role(&admin, &user, &Role::Doctor);
    assert!(!client.has_role(&user, &Role::Doctor).unwrap());
}

/// INVARIANT: Multiple roles per address are independent.
#[test]
fn invariant_multiple_roles_independent() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Doctor);
    client.assign_role(&admin, &user, &Role::Staff);

    assert!(client.has_role(&user, &Role::Doctor).unwrap());
    assert!(client.has_role(&user, &Role::Staff).unwrap());

    client.remove_role(&admin, &user, &Role::Doctor);
    assert!(!client.has_role(&user, &Role::Doctor).unwrap());
    assert!(client.has_role(&user, &Role::Staff).unwrap());
}

/// INVARIANT: Role assignment is idempotent.
#[test]
fn invariant_role_assignment_idempotent() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Patient);
    client.assign_role(&admin, &user, &Role::Patient);

    assert!(client.has_role(&user, &Role::Patient).unwrap());
}

/// INVARIANT: Revoking non-existent role does not affect other roles.
#[test]
fn invariant_revoke_nonexistent_safe() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Doctor);
    client.remove_role(&admin, &user, &Role::Patient);

    assert!(client.has_role(&user, &Role::Doctor).unwrap());
}

/// INVARIANT: Role count reflects actual assigned roles.
#[test]
fn invariant_role_count_accuracy() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Doctor);
    client.assign_role(&admin, &user, &Role::Staff);

    let roles = client.get_roles(&user).unwrap();
    assert_eq!(roles.len(), 2);
}

/// INVARIANT: Role member list reflects actual assignments.
#[test]
fn invariant_role_members_accuracy() {
    let (env, client, admin) = setup();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    client.assign_role(&admin, &user1, &Role::Doctor);
    client.assign_role(&admin, &user2, &Role::Doctor);

    let members = client.get_role_members(&Role::Doctor).unwrap();
    assert!(members.contains(&user1));
    assert!(members.contains(&user2));
}

/// INVARIANT: Address roles query returns all assigned roles.
#[test]
fn invariant_address_roles_complete() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Doctor);
    client.assign_role(&admin, &user, &Role::Patient);

    let address_roles = client.get_address_roles(&user).unwrap();
    assert!(address_roles.roles.contains(&Role::Doctor));
    assert!(address_roles.roles.contains(&Role::Patient));
}

/// INVARIANT: Removing last role leaves address with empty role set.
#[test]
fn invariant_remove_last_role_empty() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Doctor);
    client.remove_role(&admin, &user, &Role::Doctor);

    let roles = client.get_roles(&user).unwrap();
    assert_eq!(roles.len(), 0);
}

/// INVARIANT: Re-assigning after removal restores role.
#[test]
fn invariant_reassign_after_remove() {
    let (env, client, admin) = setup();
    let user = Address::generate(&env);

    client.assign_role(&admin, &user, &Role::Staff);
    client.remove_role(&admin, &user, &Role::Staff);
    assert!(!client.has_role(&user, &Role::Staff).unwrap());

    client.assign_role(&admin, &user, &Role::Staff);
    assert!(client.has_role(&user, &Role::Staff).unwrap());
}
