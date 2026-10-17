//! End-to-end flows against the deployed `PropertyRegistry` contract interface.
//!
//! These live outside the crate (`tests/` rather than `src/`) so they exercise
//! only what `lib.rs` re-exports. If a flow here stops compiling, the public
//! surface of the contract has regressed even though the unit tests still pass.

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use property_registry::{
    PropertyRegistryContract, PropertyRegistryContractClient, BASIS_POINTS_SCALE,
    DEFAULT_ADMIN_ROLE, REGISTRAR_ROLE,
};

fn sid(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

fn deploy(env: &Env) -> PropertyRegistryContractClient<'_> {
    let contract_id = env.register_contract(None, PropertyRegistryContract);
    PropertyRegistryContractClient::new(env, &contract_id)
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

/// Full register lifecycle: register, fractionalise, revalue, dilute further,
/// hand over, and reconcile the ownership split at the end.
#[test]
fn register_lifecycle_reconciles() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registrar = Address::generate(&env);
    let owner = Address::generate(&env);
    let investor_a = Address::generate(&env);
    let investor_b = Address::generate(&env);
    let successor = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &registrar);
    client.grant_role(&registrar, &REGISTRAR_ROLE);

    let id = sid(&env, "PROP-2026-001");
    client.register_property(
        &id,
        &owner,
        &250_000_00,
        &sid(&env, "USD"),
        &sid(&env, "ipfs://deed"),
    );

    client.record_ownership(&id, &investor_a, &2_500);
    client.update_valuation(&id, &300_000_00);
    client.record_ownership(&id, &investor_b, &1_500);
    client.transfer_property(&id, &successor);

    let split = client.ownership_of(&id, &owner)
        + client.ownership_of(&id, &investor_a)
        + client.ownership_of(&id, &investor_b)
        + client.ownership_of(&id, &successor);

    assert_eq!(split, BASIS_POINTS_SCALE);
    assert_eq!(client.get_property(&id).owner, successor);
    assert_eq!(client.ownership_of(&id, &successor), 6_000);
    assert_eq!(client.ownership_of(&id, &owner), 6_000);
    assert_eq!(client.get_property(&id).valuation, 300_000_00);
    assert_eq!(client.total_properties(), 1);
}

/// Secondary transfer of administration, followed by normal operation under the
/// new admin.
#[test]
fn admin_rotation_hands_over_cleanly() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let successor = Address::generate(&env);
    let registrar = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &registrar);
    client.grant_role(&registrar, &REGISTRAR_ROLE);

    client.transfer_admin(&successor);

    assert_eq!(client.admin(), successor);
    assert!(client.has_role(&successor, &DEFAULT_ADMIN_ROLE));
    assert!(!client.has_role(&admin, &DEFAULT_ADMIN_ROLE));
    assert_eq!(client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);

    // The new admin can still rotate the registrar and pause.
    let next_registrar = Address::generate(&env);
    client.set_registrar(&next_registrar);
    assert_eq!(client.registrar(), next_registrar);

    client.pause();
    assert!(client.paused());
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

/// Every rejection the happy path can hit, asserted end to end.
#[test]
fn rejected_operations_leave_the_register_untouched() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registrar = Address::generate(&env);
    let owner = Address::generate(&env);
    let investor = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &registrar);
    client.grant_role(&registrar, &REGISTRAR_ROLE);

    let id = sid(&env, "PROP-1");
    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));

    let split_before = client.ownership_of(&id, &owner);
    let holders_before = client.get_property(&id).holder_count;

    // Over-commitment, self-transfer, owner-as-holder and duplicate registration.
    assert!(client.try_record_ownership(&id, &investor, &10_001).is_err());
    assert!(client.try_transfer_property(&id, &owner).is_err());
    assert!(client.try_record_ownership(&id, &owner, &1_000).is_err());
    assert!(client
        .try_register_property(&id, &owner, &1, &sid(&env, "USD"), &sid(&env, "uri"))
        .is_err());

    assert_eq!(client.ownership_of(&id, &owner), split_before);
    assert_eq!(client.get_property(&id).holder_count, holders_before);
}

/// A registrar whose role has been withdrawn can no longer amend the register,
/// and the recorded property is untouched.
#[test]
fn registrar_role_withdrawal_is_enforced_end_to_end() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registrar = Address::generate(&env);
    let owner = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &registrar);
    client.grant_role(&registrar, &REGISTRAR_ROLE);

    let id = sid(&env, "PROP-1");
    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.update_valuation(&id, &250);

    client.revoke_role(&registrar, &REGISTRAR_ROLE);

    assert!(client.try_update_valuation(&id, &999).is_err());
    assert!(client
        .try_register_property(
            &sid(&env, "PROP-2"),
            &owner,
            &1,
            &sid(&env, "USD"),
            &sid(&env, "uri")
        )
        .is_err());
    assert_eq!(client.get_property(&id).valuation, 250);
    assert_eq!(client.total_properties(), 1);
}

/// Pausing halts every state change for every role until it is lifted.
#[test]
fn pause_halts_all_flows_until_lifted() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registrar = Address::generate(&env);
    let owner = Address::generate(&env);
    let investor = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &registrar);
    client.grant_role(&registrar, &REGISTRAR_ROLE);

    let id = sid(&env, "PROP-1");
    client.register_property(&id, &owner, &1_000, &sid(&env, "USD"), &sid(&env, "uri"));
    client.record_ownership(&id, &investor, &2_500);

    client.pause();
    assert!(client.try_update_valuation(&id, &1).is_err());
    assert!(client.try_record_ownership(&id, &investor, &1).is_err());
    assert!(client.try_transfer_property(&id, &investor).is_err());
    assert_eq!(client.ownership_of(&id, &investor), 2_500);

    client.unpause();
    client.update_valuation(&id, &1);
    assert_eq!(client.get_property(&id).valuation, 1);
}