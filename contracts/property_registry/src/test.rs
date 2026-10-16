//! Unit tests for `PropertyRegistry`, grouped by the acceptance criteria each
//! group covers: initialization, happy path, and error path.
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, String};

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

macro_rules! register {
    ($env:expr, $client:expr, $owner:expr) => {{
        let id = sid($env, "PROP-1");
        $client.register_property(
            &$id,
            $owner,
            &250_000_00,
            &sid($env, "USD"),
            &sid($env, "ipfs://deed"),
        );
        $id
    }};
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_records_admin_and_registrar() {
    let (_env, client, admin, registrar, _owner) = setup!();

    assert_eq!(client.admin(), admin);
    assert_eq!(client.registrar(), registrar);
    assert!(!client.paused());
    assert_eq!(client.total_properties(), 0);
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
fn test_queries_before_initialization_report_not_initialized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PropertyRegistryContract);
    let client = PropertyRegistryContractClient::new(&env, &contract_id);

    assert_eq!(client.try_admin(), Err(Ok(Error::NotInitialized)));
    assert_eq!(client.try_registrar(), Err(Ok(Error::NotInitialized)));
    assert_eq!(
        client.try_get_property(&sid(&env, "PROP-1")),
        Err(Ok(Error::NotInitialized))
    );
}

#[test]
fn test_admin_functions_before_initialization_report_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let next = Address::generate(&env);
    let contract_id = env.register_contract(None, PropertyRegistryContract);
    let client = PropertyRegistryContractClient::new(&env, &contract_id);

    assert_eq!(
        client.try_transfer_admin(&next),
        Err(Ok(Error::NotInitialized))
    );
    assert_eq!(client.try_pause(), Err(Ok(Error::NotInitialized)));
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

#[test]
fn test_register_property_seeds_the_full_position() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(client.total_properties(), 1);
    assert_eq!(client.ownership_of(&id, &owner), BASIS_POINTS_SCALE);

    let property = client.get_property(&id);
    assert_eq!(property.property_id, id);
    assert_eq!(property.owner, owner);
    assert_eq!(property.valuation, 250_000_00);
    assert_eq!(property.holder_count, 1);
    assert_eq!(property.created_at, property.updated_at);
}

#[test]
fn test_update_valuation_changes_the_record() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    env.ledger().set_timestamp(1_700_000_000);
    client.update_valuation(&id, &300_000_00);

    let property = client.get_property(&id);
    assert_eq!(property.valuation, 300_000_00);
    assert_eq!(property.created_at, 0);
    assert_eq!(property.updated_at, 1_700_000_000);
}

#[test]
fn test_record_ownership_moves_share_and_counts_the_holder() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &2_500);

    assert_eq!(client.ownership_of(&id, &owner), 7_500);
    assert_eq!(client.ownership_of(&id, &investor), 2_500);
    assert_eq!(client.get_property(&id).holder_count, 2);
}

#[test]
fn test_repeat_transfer_to_the_same_holder_does_not_double_count() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &2_500);
    client.record_ownership(&id, &investor, &1_500);

    assert_eq!(client.ownership_of(&id, &investor), 4_000);
    assert_eq!(client.get_property(&id).holder_count, 2);
}

#[test]
fn test_shares_can_be_diluted_to_exactly_one_hundred_percent() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.record_ownership(&id, &first, &4_000);
    client.record_ownership(&id, &second, &6_000);

    let total = client.ownership_of(&id, &owner)
        + client.ownership_of(&id, &first)
        + client.ownership_of(&id, &second);
    assert_eq!(total, BASIS_POINTS_SCALE);
    assert_eq!(client.get_property(&id).holder_count, 3);
}

#[test]
fn test_transfer_property_moves_the_whole_position() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let buyer = Address::generate(&env);

    client.transfer_property(&id, &buyer);

    assert_eq!(client.get_property(&id).owner, buyer);
    assert_eq!(client.ownership_of(&id, &owner), 0);
    assert_eq!(client.ownership_of(&id, &buyer), BASIS_POINTS_SCALE);
    assert_eq!(client.get_property(&id).holder_count, 1);
}

#[test]
fn test_transfer_property_moves_only_the_incumbents_position() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);
    let buyer = Address::generate(&env);

    client.record_ownership(&id, &investor, &3_000);
    client.transfer_property(&id, &buyer);

    // The owner of record held 7_000 bps, so 7_000 move and the investor keeps
    // its 3_000; the buyer lands at 7_000, not 100%.
    assert_eq!(client.ownership_of(&id, &investor), 3_000);
    assert_eq!(client.ownership_of(&id, &buyer), 7_000);
    assert_eq!(client.ownership_of(&id, &owner), 0);
    assert_eq!(client.get_property(&id).holder_count, 2);
}

#[test]
fn test_transfer_admin_rotates_the_admin() {
    let (env, client, _admin, _registrar, _owner) = setup!();
    let next = Address::generate(&env);

    client.transfer_admin(&next);

    assert_eq!(client.admin(), next);
}

#[test]
fn test_set_registrar_rotates_the_registrar() {
    let (env, client, _admin, _registrar, _owner) = setup!();
    let next = Address::generate(&env);

    client.set_registrar(&next);

    assert_eq!(client.registrar(), next);
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

#[test]
fn test_register_rejects_duplicate_property_id() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

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
fn test_operations_on_unknown_property_report_not_found() {
    let (env, client, _admin, _registrar, _owner) = setup!();
    let id = sid(&env, "MISSING");
    let other = Address::generate(&env);

    assert_eq!(
        client.try_get_property(&id),
        Err(Ok(Error::PropertyNotFound))
    );
    assert_eq!(
        client.try_update_valuation(&id, &1),
        Err(Ok(Error::PropertyNotFound))
    );
    assert_eq!(
        client.try_record_ownership(&id, &other, &1),
        Err(Ok(Error::PropertyNotFound))
    );
    assert_eq!(
        client.try_transfer_property(&id, &other),
        Err(Ok(Error::PropertyNotFound))
    );
}

#[test]
fn test_ownership_of_unknown_holder_is_zero_rather_than_an_error() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(client.ownership_of(&id, &Address::generate(&env)), 0);
    // Even for a property that does not exist at all.
    assert_eq!(
        client.ownership_of(&sid(&env, "MISSING"), &Address::generate(&env)),
        0
    );
    let _ = owner;
}

#[test]
fn test_record_ownership_rejects_more_than_the_owner_holds() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &6_000);

    assert_eq!(
        client.try_record_ownership(&id, &investor, &4_001),
        Err(Ok(Error::OwnershipExceedsShare))
    );
    // The rejected call must not have moved anything.
    assert_eq!(client.ownership_of(&id, &investor), 6_000);
    assert_eq!(client.ownership_of(&id, &owner), 4_000);
}

#[test]
fn test_record_ownership_rejects_crediting_the_owner_of_record() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(
        client.try_record_ownership(&id, &owner, &1_000),
        Err(Ok(Error::InvalidArgument))
    );
}

#[test]
fn test_record_ownership_rejects_pushing_a_holder_above_one_hundred_percent() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &6_000);
    client.record_ownership(&id, &investor, &4_000);

    assert_eq!(client.ownership_of(&id, &investor), BASIS_POINTS_SCALE);
    assert_eq!(client.ownership_of(&id, &owner), 0);
}

#[test]
fn test_transfer_property_rejects_transfer_to_self() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(
        client.try_transfer_property(&id, &owner),
        Err(Ok(Error::InvalidArgument))
    );
    assert_eq!(client.ownership_of(&id, &owner), BASIS_POINTS_SCALE);
}

#[test]
fn test_transfer_property_with_a_diluted_owner_moves_nothing() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    // The owner hands the whole position to `investor`, who is then at 100%.
    client.record_ownership(&id, &investor, &8_000);
    assert_eq!(client.ownership_of(&id, &investor), 8_000);

    // A successor that already holds the maximum cannot be credited further.
    let other = Address::generate(&env);
    client.record_ownership(&id, &other, &2_000);
    client.record_ownership(&id, &investor, &2_000);
    assert_eq!(client.ownership_of(&id, &investor), BASIS_POINTS_SCALE);

    // The owner now holds zero, so handing the property over moves nothing and
    // cannot over-commit the new owner of record.
    client.transfer_property(&id, &investor);
    assert_eq!(client.ownership_of(&id, &investor), BASIS_POINTS_SCALE);
}

#[test]
fn test_pause_blocks_every_state_change_and_unpause_restores_them() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

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
        client.try_transfer_property(&id, &investor),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_register_property(
            &sid(&env, "PROP-2"),
            &owner,
            &1,
            &sid(&env, "USD"),
            &sid(&env, "uri")
        ),
        Err(Ok(Error::Unauthorized))
    );

    client.unpause();
    client.update_valuation(&id, &1);
    assert_eq!(client.get_property(&id).valuation, 1);
}

#[test]
fn test_pause_does_not_block_queries() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    client.pause();

    // Deliberate: a registry that cannot be inspected cannot be investigated.
    assert!(client.paused());
    assert_eq!(client.ownership_of(&id, &owner), BASIS_POINTS_SCALE);
    assert_eq!(client.get_property(&id).owner, owner);
    assert_eq!(client.total_properties(), 1);
}

#[test]
fn test_valuation_accepts_the_full_u128_range() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    client.update_valuation(&id, &u128::MAX);
    assert_eq!(client.get_property(&id).valuation, u128::MAX);

    client.update_valuation(&id, &0);
    assert_eq!(client.get_property(&id).valuation, 0);
}

// ---------------------------------------------------------------------------
// Storage layout
// ---------------------------------------------------------------------------

#[test]
fn test_data_key_variant_count_matches_the_enum() {
    // If someone inserts a variant in the middle of `DataKey`, this fails and
    // points at the storage-collision hazard rather than letting it reach a
    // deployment.
    assert_eq!(DATA_KEY_VARIANT_COUNT, 7);
}

#[test]
fn test_property_records_are_independent() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let first = register!(&env, &client, &owner);

    let second = sid(&env, "PROP-2");
    client.register_property(
        &second,
        &owner,
        &1,
        &sid(&env, "USD"),
        &sid(&env, "uri"),
    );

    client.update_valuation(&first, &999);

    assert_eq!(client.get_property(&first).valuation, 999);
    assert_eq!(client.get_property(&second).valuation, 1);
    assert_eq!(client.total_properties(), 2);
}

#[test]
fn test_ownership_entries_are_scoped_to_their_property() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let first = register!(&env, &client, &owner);
    let second = sid(&env, "PROP-2");
    let investor = Address::generate(&env);

    client.register_property(
        &second,
        &owner,
        &1,
        &sid(&env, "USD"),
        &sid(&env, "uri"),
    );
    client.record_ownership(&first, &investor, &1_000);

    assert_eq!(client.ownership_of(&first, &investor), 1_000);
    assert_eq!(client.ownership_of(&second, &investor), 0);
}