#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol};

macro_rules! setup {
    ($registrar_has_role:expr) => {{
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let registrar = Address::generate(&env);
        let owner = Address::generate(&env);

        let contract_id = env.register_contract(None, PropertyRegistryContract);
        let client = PropertyRegistryContractClient::new(&env, &contract_id);
        client.initialize(&admin, &registrar);
        if $registrar_has_role {
            client.grant_role(&registrar, &REGISTRAR_ROLE);
        }

        let id = sid(&env, "PROP-1");
        client.register_property(&id, &owner, &250_000_00, &sid(&env, "USD"), &sid(&env, "ipfs://deed"));

        (env, contract_id, client, admin, registrar, owner, id)
    }};
}

fn sid(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

/// Was `who` the address that had to authorize `function` on `contract_id`?
///
/// `Symbol::new` rather than `symbol_short!` because several entry-point names
/// exceed the 9-character short-symbol limit.
fn authorized(env: &Env, who: &Address, contract_id: &Address, function: &str) -> bool {
    let expected = Symbol::new(env, function);
    env.auths()
        .iter()
        .any(|record| record.0 == *who && record.1 == *contract_id && record.2 == expected)
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_grants_the_deployer_the_admin_role() {
    let (_env, _cid, client, admin, _registrar, _owner, _id) = setup!(false);

    assert!(client.has_role(&admin, &DEFAULT_ADMIN_ROLE));
    assert_eq!(client.admin(), admin);
    assert_eq!(client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);
}

#[test]
fn test_initialize_can_only_run_once() {
    let (_env, _cid, client, admin, registrar, _owner, _id) = setup!(false);

    assert_eq!(
        client.try_initialize(&admin, &registrar),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_is_gated_by_the_deployer() {
    let env = Env::default();
    env.mock_all_auths();

    let deployer = Address::generate(&env);
    let contract_id = env.register_contract(None, PropertyRegistryContract);
    let client = PropertyRegistryContractClient::new(&env, &contract_id);

    client.initialize(&deployer, &deployer);

    assert!(authorized(&env, &deployer, &contract_id, "initialize"));
}

// ---------------------------------------------------------------------------
// #1779 — access control audit
// ---------------------------------------------------------------------------

#[test]
fn test_register_property_is_gated_by_the_registrar() {
    let (env, contract_id, client, _admin, registrar, _owner, _id) = setup!(true);

    client.register_property(
        &sid(&env, "PROP-2"),
        &Address::generate(&env),
        &100,
        &sid(&env, "USD"),
        &sid(&env, "uri"),
    );

    assert!(authorized(
        &env,
        &registrar,
        &contract_id,
        "register_property"
    ));
}

#[test]
fn test_registrar_slot_without_the_role_cannot_amend_the_register() {
    let (env, _cid, client, _admin, _registrar, _owner, _id) = setup!(false);

    // `initialize` recorded this address as the registrar, but no role was
    // granted, so every registrar-gated write is refused.
    assert_eq!(
        client.try_update_valuation(&sid(&env, "PROP-1"), &1),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn test_role_checks_reject_addresses_without_the_role() {
    let (env, _cid, client, _admin, registrar, _owner, _id) = setup!(false);
    let stranger = Address::generate(&env);

    assert!(!client.has_role(&stranger, &DEFAULT_ADMIN_ROLE));
    assert!(!client.has_role(&registrar, &REGISTRAR_ROLE));
}

#[test]
fn test_update_valuation_is_gated_by_the_registrar() {
    let (env, contract_id, client, _admin, registrar, _owner, id) = setup!(true);

    client.update_valuation(&id, &300_000_00);

    assert!(authorized(&env, &registrar, &contract_id, "update_valuation"));
}

#[test]
fn test_record_ownership_needs_both_the_role_and_the_owner() {
    let (env, contract_id, client, _admin, _registrar, owner, id) = setup!(true);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &2_500);

    // Both signatures are demanded: the registrar to amend the register, the
    // owner of record to have their position debited.
    assert!(authorized(&env, &owner, &contract_id, "record_ownership"));
}

#[test]
fn test_transfer_property_is_gated_by_the_owner_of_record() {
    let (env, contract_id, client, _admin, _registrar, owner, id) = setup!(true);
    let buyer = Address::generate(&env);

    client.transfer_property(&id, &buyer);

    assert!(authorized(
        &env,
        &owner,
        &contract_id,
        "transfer_property"
    ));
}

#[test]
fn test_revoke_removes_the_role_and_stops_amendments() {
    let (env, _cid, client, _admin, registrar, _owner, id) = setup!(true);

    client.revoke_role(&registrar, &REGISTRAR_ROLE);

    assert!(!client.has_role(&registrar, &REGISTRAR_ROLE));
    assert_eq!(client.role_members(&REGISTRAR_ROLE).len(), 0);
    assert_eq!(
        client.try_update_valuation(&id, &1),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn test_a_registrar_without_the_role_cannot_escalate_itself() {
    let (env, _cid, client, _admin, _registrar, _owner, _id) = setup!(false);
    let intruder = Address::generate(&env);

    client.grant_role(&intruder, &REGISTRAR_ROLE);
    assert!(!client.has_role(&intruder, &DEFAULT_ADMIN_ROLE));
}

#[test]
fn test_last_admin_cannot_be_revoked() {
    let (_env, _cid, client, admin, _registrar, _owner, _id) = setup!(false);

    assert_eq!(
        client.try_revoke_role(&admin, &DEFAULT_ADMIN_ROLE),
        Err(Ok(Error::CannotRevokeLastAdmin))
    );
}

#[test]
fn test_second_admin_allows_the_first_to_step_down() {
    let (env, _cid, client, admin, _registrar, _owner, _id) = setup!(false);
    let successor = Address::generate(&env);

    client.grant_role(&successor, &DEFAULT_ADMIN_ROLE);
    client.revoke_role(&admin, &DEFAULT_ADMIN_ROLE);

    assert!(!client.has_role(&admin, &DEFAULT_ADMIN_ROLE));
    assert!(client.has_role(&successor, &DEFAULT_ADMIN_ROLE));
}

#[test]
fn test_transfer_admin_rotates_the_role_not_just_the_mirror() {
    let (env, _cid, client, _admin, _registrar, _owner, _id) = setup!(false);
    let next = Address::generate(&env);

    client.transfer_admin(&next);

    assert_eq!(client.admin(), next);
    assert!(client.has_role(&next, &DEFAULT_ADMIN_ROLE));
    assert!(!client.has_role(&Address::generate(&env), &DEFAULT_ADMIN_ROLE));
    assert_eq!(client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);
}

#[test]
fn test_renounce_clears_every_role() {
    let (env, _cid, client, _admin, registrar, _owner, _id) = setup!(true);

    client.renounce_role();

    assert!(!client.has_role(&registrar, &REGISTRAR_ROLE));
    assert_eq!(client.role_members(&REGISTRAR_ROLE).len(), 0);
}

#[test]
fn test_pause_blocks_every_state_change_regardless_of_role() {
    let (env, _cid, client, _admin, _registrar, owner, id) = setup!(true);
    let investor = Address::generate(&env);
    let buyer = Address::generate(&env);

    client.pause();

    assert!(client.paused());
    assert_eq!(
        client.try_update_valuation(&id, &1),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_record_ownership(&id, &investor, &1),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_transfer_property(&id, &buyer),
        Err(Ok(Error::Unauthorized))
    );

    client.unpause();
    client.update_valuation(&id, &1);
}

// ---------------------------------------------------------------------------
// Behaviour
// ---------------------------------------------------------------------------

#[test]
fn test_register_seeds_full_ownership() {
    let (env, _cid, client, _admin, _registrar, owner, id) = setup!(true);

    assert_eq!(client.total_properties(), 1);
    assert_eq!(client.ownership_of(&id, &owner), BASIS_POINTS_SCALE);
    assert_eq!(client.get_property(&id).holder_count, 1);
}

#[test]
fn test_register_rejects_duplicate_id() {
    let (env, _cid, client, _admin, registrar, owner, id) = setup!(true);

    assert_eq!(
        client.try_register_property(
            &id,
            &owner,
            &1,
            &sid(&env, "USD"),
            &sid(&env, "uri")
        ),
        Err(Ok(Error::PropertyAlreadyExists))
    );
    assert_eq!(client.total_properties(), 1);
}

#[test]
fn test_record_ownership_moves_share_and_counts_the_holder() {
    let (env, _cid, client, _admin, _registrar, owner, id) = setup!(true);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &2_500);

    assert_eq!(client.ownership_of(&id, &owner), 7_500);
    assert_eq!(client.ownership_of(&id, &investor), 2_500);
    assert_eq!(client.get_property(&id).holder_count, 2);

    client.record_ownership(&id, &investor, &1_000);
    assert_eq!(client.get_property(&id).holder_count, 2);
}

#[test]
fn test_record_ownership_rejects_more_than_the_owner_holds() {
    let (env, _cid, client, _admin, _registrar, owner, id) = setup!(true);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &6_000);

    assert_eq!(
        client.try_record_ownership(&id, &investor, &4_001),
        Err(Ok(Error::OwnershipExceedsShare))
    );
    assert_eq!(client.ownership_of(&id, &investor), 6_000);
}

#[test]
fn test_transfer_property_moves_the_whole_position() {
    let (env, _cid, client, _admin, _registrar, owner, id) = setup!(true);
    let buyer = Address::generate(&env);

    client.transfer_property(&id, &buyer);

    assert_eq!(client.get_property(&id).owner, buyer);
    assert_eq!(client.ownership_of(&id, &owner), 0);
    assert_eq!(client.ownership_of(&id, &buyer), BASIS_POINTS_SCALE);
    assert_eq!(client.get_property(&id).holder_count, 1);
}

#[test]
fn test_transfer_property_rejects_transfer_to_self() {
    let (env, _cid, client, _admin, _registrar, owner, id) = setup!(true);

    assert_eq!(
        client.try_transfer_property(&id, &owner),
        Err(Ok(Error::InvalidArgument))
    );
}

#[test]
fn test_queries_fail_for_unknown_property() {
    let (env, _cid, client, _admin, _registrar, _owner, _id) = setup!(true);
    let id = sid(&env, "MISSING");

    assert_eq!(client.try_get_property(&id), Err(Ok(Error::PropertyNotFound)));
}

#[test]
fn test_ownership_of_unknown_holder_is_zero() {
    let (env, _cid, client, _admin, _registrar, _owner, id) = setup!(true);

    assert_eq!(client.ownership_of(&id, &Address::generate(&env)), 0);
}