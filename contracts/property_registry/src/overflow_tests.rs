//! Regression tests for issue #1781 (integer overflow guards) and
//! #1782 (storage gap / upgrade-safety markers).
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String};

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

// ---------------------------------------------------------------------------
// #1781 — integer overflow / underflow guards
// ---------------------------------------------------------------------------

#[test]
fn record_ownership_rejects_more_than_the_owner_holds() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let investor = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));

    // Above 100% in absolute terms.
    assert_eq!(
        client.try_record_ownership(&id, &investor, &(BASIS_POINTS_SCALE + 1)),
        Err(Ok(Error::ShareTotalExceeded))
    );

    // Within 100% in absolute terms, but more than the owner of record still
    // holds. These are two different failure reasons and two different errors.
    client.record_ownership(&id, &investor, &5_000);
    assert_eq!(
        client.try_record_ownership(&id, &investor, &5_001),
        Err(Ok(Error::OwnershipExceedsShare))
    );
}

#[test]
fn record_ownership_rejects_an_amount_near_u128_max() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let investor = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));

    // The share bound, not an accidental wrap, is what rejects this.
    assert!(client
        .try_record_ownership(&id, &investor, &u128::MAX)
        .is_err());
    assert_eq!(client.ownership_of(&id, &investor), 0);
}

#[test]
fn record_ownership_leaves_the_split_untouched_on_rejection() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let investor = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.record_ownership(&id, &investor, &2_500);

    let owner_before = client.ownership_of(&id, &owner);
    let investor_before = client.ownership_of(&id, &investor);

    assert!(client.try_record_ownership(&id, &investor, &u128::MAX).is_err());

    assert_eq!(client.ownership_of(&id, &owner), owner_before);
    assert_eq!(client.ownership_of(&id, &investor), investor_before);
    assert_eq!(owner_before + investor_before, BASIS_POINTS_SCALE);
}

#[test]
fn shares_never_sum_above_one_hundred_percent() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.record_ownership(&id, &first, &4_000);
    client.record_ownership(&id, &second, &6_000);

    // The owner of record has been fully diluted; one basis point more is out.
    assert_eq!(client.ownership_of(&id, &owner), 0);
    assert!(client.try_record_ownership(&id, &second, &1).is_err());

    let total = client.ownership_of(&id, &owner)
        + client.ownership_of(&id, &first)
        + client.ownership_of(&id, &second);
    assert_eq!(total, BASIS_POINTS_SCALE);
}

#[test]
fn transfer_property_credits_before_it_debits() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let buyer = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    client.record_ownership(&id, &buyer, &2_500);

    // The incumbent now holds 7_500 bps; handing that to `buyer`, who already
    // holds 2_500, would land at exactly 10_000.
    client.transfer_property(&id, &buyer);

    assert_eq!(client.get_property(&id).owner, buyer);
    assert_eq!(client.ownership_of(&id, &owner), 0);
    assert_eq!(client.ownership_of(&id, &buyer), BASIS_POINTS_SCALE);
}

#[test]
fn transfer_property_rejects_transfer_to_self() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));

    assert_eq!(
        client.try_transfer_property(&id, &owner),
        Err(Ok(Error::InvalidArgument))
    );
}

#[test]
fn record_ownership_rejects_the_owner_of_record_as_holder() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));

    // Crediting the debit side would be a no-op written twice.
    assert_eq!(
        client.try_record_ownership(&id, &owner, &1_000),
        Err(Ok(Error::InvalidArgument))
    );
}

#[test]
fn holder_count_tracks_distinct_holders() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");
    let investor = Address::generate(&env);

    client.register_property(&id, &owner, &100, &sid(&env, "USD"), &sid(&env, "uri"));
    assert_eq!(client.get_property(&id).holder_count, 1);

    client.record_ownership(&id, &investor, &2_500);
    assert_eq!(client.get_property(&id).holder_count, 2);

    // A second transfer to the same holder must not double-count them.
    client.record_ownership(&id, &investor, &1_000);
    assert_eq!(client.get_property(&id).holder_count, 2);
}

#[test]
fn valuation_accepts_the_full_u128_range() {
    let (env, client, _admin, _registrar, owner) = setup!();
    let id = sid(&env, "PROP-1");

    client.register_property(&id, &owner, &0, &sid(&env, "USD"), &sid(&env, "uri"));
    client.update_valuation(&id, &u128::MAX);

    assert_eq!(client.get_property(&id).valuation, u128::MAX);
}

// ---------------------------------------------------------------------------
// #1782 — storage gap / upgrade-safety markers
// ---------------------------------------------------------------------------

#[test]
fn initialize_stamps_storage_schema_version() {
    let (_env, client, _admin, _registrar, _owner) = setup!();

    assert_eq!(client.storage_version(), STORAGE_SCHEMA_VERSION);
}

#[test]
fn storage_version_is_zero_before_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PropertyRegistryContract);
    let client = PropertyRegistryContractClient::new(&env, &contract_id);

    assert_eq!(client.storage_version(), 0);
}

#[test]
fn reserved_gap_slots_are_reported_and_bounded() {
    let (_env, client, _admin, _registrar, _owner) = setup!();

    assert_eq!(client.storage_gap_slots(), STORAGE_GAP_SLOTS);
    assert!(STORAGE_GAP_SLOTS > 0);
}

#[test]
fn gap_slot_predicate_covers_exactly_the_reserved_range() {
    let (_env, client, _admin, _registrar, _owner) = setup!();
    let slots = client.storage_gap_slots();

    for slot in 0..slots {
        assert!(
            client.is_reserved_gap_slot(&slot),
            "slot {slot} should be reserved"
        );
    }

    assert!(!client.is_reserved_gap_slot(&slots));
    assert!(!client.is_reserved_gap_slot(&(slots + 1)));
}