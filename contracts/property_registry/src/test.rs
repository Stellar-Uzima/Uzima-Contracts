#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String};

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

/// Register a valid property, returning its id.
macro_rules! register {
    ($env:expr, $client:expr, $owner:expr) => {{
        let id = sid($env, "PROP-1");
        $client.register_property(
            &id,
            $owner,
            &250_000_00,
            &sid($env, "USD"),
            &sid($env, "ipfs://deed"),
        );
        id
    }};
}

/// Count events published under the given topic symbols.
fn count_events(env: &Env, topics: &[soroban_sdk::Symbol]) -> usize {
    env.events()
        .all()
        .iter()
        .filter(|event| {
            topics.len() <= event.topics.len()
                && topics
                    .iter()
                    .enumerate()
                    .all(|(i, expected)| event.topics[i] == expected.clone().into())
        })
        .count()
}

// ---------------------------------------------------------------------------
// Initialization and administration
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin_and_registrar() {
    let (env, client, admin, _registrar, _owner) = setup!();
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
fn test_transfer_admin_rotates_admin() {
    let (env, client, _admin, _registrar, _owner) = setup!();
    let next = Address::generate(&env);

    client.transfer_admin(&next);

    assert_eq!(client.admin(), next);
}

#[test]
fn test_pause_blocks_state_changes_and_resumes() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

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

// ---------------------------------------------------------------------------
// Input validation (#1775)
// ---------------------------------------------------------------------------

#[test]
fn test_register_rejects_empty_property_id() {
    let (env, client, _admin, _registrar, owner) = setup!();

    assert_eq!(
        client.try_register_property(
            &sid(&env, ""),
            &owner,
            &100,
            &sid(&env, "USD"),
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidPropertyId))
    );
}

#[test]
fn test_register_rejects_characters_outside_the_alphabet() {
    let (env, client, _admin, _registrar, owner) = setup!();

    assert_eq!(
        client.try_register_property(
            &sid(&env, "PROP/001"),
            &owner,
            &100,
            &sid(&env, "USD"),
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidPropertyId))
    );
}

#[test]
fn test_register_rejects_zero_valuation() {
    let (env, client, _admin, _registrar, owner) = setup!();

    assert_eq!(
        client.try_register_property(
            &sid(&env, "PROP-1"),
            &owner,
            &0,
            &sid(&env, "USD"),
            &sid(&env, "uri")
        ),
        Err(Ok(Error::ZeroValue))
    );
}

#[test]
fn test_register_rejects_non_alphabetic_currency() {
    let (env, client, _admin, _registrar, owner) = setup!();

    assert_eq!(
        client.try_register_property(
            &sid(&env, "PROP-1"),
            &owner,
            &100,
            &sid(&env, "US1"),
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidCurrency))
    );
}

#[test]
fn test_register_rejects_empty_metadata_uri() {
    let (env, client, _admin, _registrar, owner) = setup!();

    assert_eq!(
        client.try_register_property(
            &sid(&env, "PROP-1"),
            &owner,
            &100,
            &sid(&env, "USD"),
            &sid(&env, "")
        ),
        Err(Ok(Error::InvalidMetadataUri))
    );
}

#[test]
fn test_update_valuation_rejects_zero() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(
        client.try_update_valuation(&id, &0),
        Err(Ok(Error::ZeroValue))
    );
}

#[test]
fn test_record_ownership_rejects_zero_share() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(
        client.try_record_ownership(&id, &Address::generate(&env), &0),
        Err(Ok(Error::ZeroValue))
    );
}

#[test]
fn test_record_ownership_rejects_a_share_above_one_hundred_percent() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    assert_eq!(
        client.try_record_ownership(&id, &investor, &(BASIS_POINTS_SCALE + 1)),
        Err(Ok(Error::ShareTooLarge))
    );
}

#[test]
fn test_record_ownership_rejects_the_owner_as_holder() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(
        client.try_record_ownership(&id, &owner, &1_000),
        Err(Ok(Error::HolderIsOwner))
    );
}

#[test]
fn test_transfer_rejects_transfer_to_self() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(
        client.try_transfer_property(&id, &owner),
        Err(Ok(Error::TransferToSelf))
    );
}

#[test]
fn test_validation_runs_before_authorization_is_consumed() {
    // A malformed call must be rejected on the argument, not on auth, so the
    // error a caller sees cannot be used to probe the registrar set.
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let registrar = Address::generate(&env);
    let contract_id = env.register_contract(None, PropertyRegistryContract);
    let client = PropertyRegistryContractClient::new(&env, &contract_id);
    client.initialize(&admin, &registrar);

    assert_eq!(
        client.try_register_property(
            &sid(&env, "PROP/001"),
            &registrar,
            &1,
            &sid(&env, "USD"),
            &sid(&env, "uri")
        ),
        Err(Ok(Error::InvalidPropertyId))
    );
    assert_eq!(client.total_properties(), 0);
}

// ---------------------------------------------------------------------------
// Typed custom errors (#1776)
// ---------------------------------------------------------------------------

#[test]
fn test_share_bound_and_balance_shortfall_are_distinguishable() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &6_000);

    // Both are "you asked for too much", but the absolute bound and the
    // available balance are different facts with different discriminants.
    assert_eq!(
        client.try_record_ownership(&id, &investor, &(BASIS_POINTS_SCALE + 1)),
        Err(Ok(Error::ShareTooLarge))
    );
    assert_eq!(
        client.try_record_ownership(&id, &investor, &4_001),
        Err(Ok(Error::OwnershipExceedsShare))
    );
}

#[test]
fn test_shares_never_sum_above_one_hundred_percent() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.record_ownership(&id, &first, &4_000);
    client.record_ownership(&id, &second, &6_000);

    assert_eq!(client.ownership_of(&id, &owner), 0);
    assert!(client.try_record_ownership(&id, &second, &1).is_err());

    let total = client.ownership_of(&id, &owner)
        + client.ownership_of(&id, &first)
        + client.ownership_of(&id, &second);
    assert_eq!(total, BASIS_POINTS_SCALE);
}

#[test]
fn test_error_variants_map_to_stable_discriminants() {
    // These numbers are part of the contract interface: clients match on them.
    assert_eq!(Error::InvalidPropertyId as u32, 6);
    assert_eq!(Error::InvalidCurrency as u32, 7);
    assert_eq!(Error::InvalidMetadataUri as u32, 8);
    assert_eq!(Error::ZeroValue as u32, 9);
    assert_eq!(Error::HolderIsOwner as u32, 10);
    assert_eq!(Error::TransferToSelf as u32, 11);
    assert_eq!(Error::ShareTotalExceeded as u32, 12);
    assert_eq!(Error::OwnershipExceedsShare as u32, 13);
    assert_eq!(Error::ShareTooLarge as u32, 14);
    assert_eq!(Error::ArithmeticOverflow as u32, 15);
    assert_eq!(Error::ArithmeticUnderflow as u32, 16);
}

// ---------------------------------------------------------------------------
// Events (#1773)
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_emits_initialized_event() {
    let (_env, client, _admin, _registrar, _owner) = setup!();

    assert_eq!(count_events(&_env, &[symbol_short!("init")]), 1);
    assert!(client.total_properties() == 0);
}

#[test]
fn test_admin_and_registrar_changes_emit_events() {
    let (env, client, _admin, _registrar, _owner) = setup!();
    let next_admin = Address::generate(&env);
    let next_registrar = Address::generate(&env);

    client.transfer_admin(&next_admin);
    client.set_registrar(&next_registrar);

    assert_eq!(count_events(&env, &[symbol_short!("adm_xfer")]), 1);
    assert_eq!(count_events(&env, &[symbol_short!("registrar")]), 1);
}

#[test]
fn test_pause_and_unpause_each_emit_one_event() {
    let (env, client, _admin, _registrar, _owner) = setup!();

    client.pause();
    assert_eq!(count_events(&env, &[symbol_short!("pause")]), 1);

    client.unpause();
    assert_eq!(count_events(&env, &[symbol_short!("pause")]), 2);
}

#[test]
fn test_property_lifecycle_emits_one_event_per_state_change() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);
    let buyer = Address::generate(&env);

    client.update_valuation(&id, &300_000_00);
    client.record_ownership(&id, &investor, &2_500);
    client.transfer_property(&id, &buyer);

    assert_eq!(
        count_events(&env, &[symbol_short!("prop"), symbol_short!("reg")]),
        1
    );
    assert_eq!(
        count_events(&env, &[symbol_short!("prop"), symbol_short!("value")]),
        1
    );
    assert_eq!(
        count_events(&env, &[symbol_short!("prop"), symbol_short!("share")]),
        1
    );
    assert_eq!(
        count_events(&env, &[symbol_short!("prop"), symbol_short!("xfer")]),
        1
    );
}

#[test]
fn test_rejected_calls_emit_no_events() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);
    let before = env.events().all().len();

    assert!(client.try_update_valuation(&id, &0).is_err());
    assert!(client.try_transfer_property(&id, &owner).is_err());
    assert!(client
        .try_record_ownership(&id, &owner, &1_000)
        .is_err());

    assert_eq!(env.events().all().len(), before);
}

// ---------------------------------------------------------------------------
// Behaviour
// ---------------------------------------------------------------------------

#[test]
fn test_register_seeds_full_ownership() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(client.total_properties(), 1);
    assert_eq!(client.ownership_of(&id, &owner), BASIS_POINTS_SCALE);
    assert_eq!(client.ownership_percent(&id, &owner), 100);
    assert_eq!(client.get_property(&id).holder_count, 1);
}

#[test]
fn test_register_rejects_duplicate_id() {
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
fn test_update_valuation_changes_record() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    client.update_valuation(&id, &300_000_00);

    assert_eq!(client.get_property(&id).valuation, 300_000_00);
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
fn test_record_ownership_moves_share_and_counts_the_holder() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);
    let investor = Address::generate(&env);

    client.record_ownership(&id, &investor, &2_500);

    assert_eq!(client.ownership_of(&id, &owner), 7_500);
    assert_eq!(client.ownership_of(&id, &investor), 2_500);
    assert_eq!(client.ownership_percent(&id, &investor), 25);
    assert_eq!(client.get_property(&id).holder_count, 2);

    // A repeat transfer to the same holder must not double-count them.
    client.record_ownership(&id, &investor, &1_000);
    assert_eq!(client.get_property(&id).holder_count, 2);
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
fn test_ownership_of_unknown_holder_is_zero() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = register!(&env, &client, &owner);

    assert_eq!(client.ownership_of(&id, &Address::generate(&env)), 0);
}