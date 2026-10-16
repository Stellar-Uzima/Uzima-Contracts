#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Build a fresh registry plus an initialized client bound to a local `Env`.
///
/// This has to be a macro rather than a function: the generated client borrows
/// the `Env`, so it cannot outlive a returned `Env`.
macro_rules! setup {
    () => {{
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let registrar = Address::generate(&env);
        let owner = Address::generate(&env);

        let contract_id = env.register_contract(None, PropertyRegistryContract);
        let client = PropertyRegistryContractClient::new(&env, &contract_id);
        client.initialize(&admin, &registrar);

        (env, client, admin, registrar, owner)
    }};
}

fn sid(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

#[test]
fn test_initialize_sets_admin_and_registrar() {
    let (env, client, admin, registrar, _owner) = setup!();
    let next = Address::generate(&env);

    client.set_registrar(&next);

    assert_eq!(client.admin(), admin);
    assert_eq!(client.registrar(), next);
    assert!(!client.paused());
}

#[test]
fn test_initialize_can_only_run_once() {
    let (_env, client, admin, registrar, _owner) = setup!();

    assert_eq!(
        client.try_initialize(&admin, &registrar),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn test_register_property_seeds_full_ownership() {
    let (env, client, _admin, registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(
        &id,
        &owner,
        &250_000_00,
        &sid(&env, "USD"),
        &sid(&env, "ipfs://deed"),
    );

    assert_eq!(client.total_properties(), 1);
    assert_eq!(client.ownership_of(&id, &owner), 10_000);

    let property = client.get_property(&id);
    assert_eq!(property.property_id, id);
    assert_eq!(property.owner, owner);
    assert_eq!(property.valuation, 250_000_00);
}

#[test]
fn test_register_property_rejects_duplicate_id() {
    let (env, client, _admin, registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(&id, &owner, &1, &sid(&env, "USD"), &sid(&env, "uri"));

    assert_eq!(
        client.try_register_property(&id, &owner, &1, &sid(&env, "USD"), &sid(&env, "uri")),
        Err(Ok(Error::PropertyAlreadyExists))
    );
}

#[test]
fn test_update_valuation_changes_record() {
    let (env, client, _admin, registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.update_valuation(&id, &175);

    assert_eq!(client.get_property(&id).valuation, 175);
}

#[test]
fn test_update_valuation_requires_existing_property() {
    let (env, client, _admin, _registrar, _owner) = setup!();
    let id = sid(&env, "MISSING");

    assert_eq!(
        client.try_update_valuation(&id, &1),
        Err(Ok(Error::PropertyNotFound))
    );
}

#[test]
fn test_transfer_property_moves_full_share() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let buyer = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.transfer_property(&id, &buyer);

    assert_eq!(client.get_property(&id).owner, buyer);
    assert_eq!(client.ownership_of(&id, &owner), 0);
    assert_eq!(client.ownership_of(&id, &buyer), 10_000);
}

#[test]
fn test_record_ownership_sets_share() {
    let (env, client, _admin, registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let investor = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.record_ownership(&id, &investor, &2_500);

    assert_eq!(client.ownership_of(&id, &investor), 2_500);
}

#[test]
fn test_pause_blocks_state_changes() {
    let (env, client, _admin, registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.pause();

    assert!(client.paused());
    assert_eq!(
        client.try_update_valuation(&id, &2),
        Err(Ok(Error::Unauthorized))
    );

    client.unpause();
    client.update_valuation(&id, &2);
    assert_eq!(client.get_property(&id).valuation, 2);
}

#[test]
fn test_transfer_admin_rotates_admin() {
    let (env, client, _admin, _registrar, _owner) = setup!();
    let next = Address::generate(&env);

    client.transfer_admin(&next);

    assert_eq!(client.admin(), next);
}

#[test]
fn test_ownership_of_unknown_holder_is_zero() {
    let (env, client, _admin, registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));

    assert_eq!(client.ownership_of(&id, &Address::generate(&env)), 0);
}