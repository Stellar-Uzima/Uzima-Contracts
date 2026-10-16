//! Regression tests for issue #1765 (integer overflow guards) and
//! #1766 (storage gap / upgrade-safety markers).
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String};

macro_rules! setup {
    () => {{
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let issuer = Address::generate(&env);
        let treasury = Address::generate(&env);

        let contract_id = env.register_contract(None, TokenizationContract);
        let client = TokenizationContractClient::new(&env, &contract_id);
        client.initialize(&admin, &treasury);

        (env, client, admin, issuer)
    }};
}

fn sid(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

// ---------------------------------------------------------------------------
// #1765 — integer overflow / underflow guards
// ---------------------------------------------------------------------------

#[test]
fn mint_rejects_supply_overflow() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "SATURATED");
    let holder = Address::generate(&env);

    client.register_asset(&id, &issuer, &u128::MAX, &6, &sid(&env, "uri"));
    client.mint(&id, &holder, &u128::MAX);

    assert_eq!(
        client.try_mint(&id, &holder, &1),
        Err(Ok(Error::ArithmeticOverflow))
    );
}

#[test]
fn mint_rejects_amount_that_would_wrap_supply() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "SATURATED");
    let holder = Address::generate(&env);

    // A small registered supply plus a near-`u128::MAX` mint wraps the running
    // total even though the holder's own balance would still fit.
    client.register_asset(&id, &issuer, &10, &6, &sid(&env, "uri"));

    assert_eq!(
        client.try_mint(&id, &holder, &u128::MAX),
        Err(Ok(Error::ArithmeticOverflow))
    );
    assert_eq!(client.balance_of(&id, &holder), 0);
    assert_eq!(client.total_supply(&id), 10);
}

#[test]
fn burn_rejects_amount_above_balance() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let holder = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &holder, &100);

    assert_eq!(
        client.try_burn(&id, &holder, &101),
        Err(Ok(Error::InsufficientBalance))
    );
}

#[test]
fn burn_leaves_state_untouched_on_rejection() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let holder = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &holder, &100);

    assert!(client.try_burn(&id, &holder, &1_000).is_err());
    assert_eq!(client.balance_of(&id, &holder), 100);
    assert_eq!(client.total_supply(&id), 100);
}

#[test]
fn transfer_rejects_amount_above_balance() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &50);

    assert_eq!(
        client.try_transfer(&id, &alice, &bob, &51),
        Err(Ok(Error::InsufficientBalance))
    );
    assert_eq!(client.balance_of(&id, &alice), 50);
    assert_eq!(client.balance_of(&id, &bob), 0);
}

#[test]
fn transfer_preserves_total_supply_across_both_legs() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &u128::MAX - 10);
    client.transfer(&id, &alice, &bob, &5);

    assert_eq!(client.total_supply(&id), u128::MAX - 10);
}

#[test]
fn self_transfer_credits_and_debits_the_same_holder() {
    let (env, client, _admin, issuer) = setup!();
    let id = sid(&env, "HOSP-1");
    let alice = Address::generate(&env);

    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "uri"));
    client.mint(&id, &alice, &100);
    client.transfer(&id, &alice, &alice, &40);

    // The debit and credit cancel out; the guard must not let the intermediate
    // subtraction leave the holder's balance wedged at the debited value.
    assert_eq!(client.balance_of(&id, &alice), 100);
    assert_eq!(client.total_supply(&id), 100);
}

// ---------------------------------------------------------------------------
// #1766 — storage gap / upgrade-safety markers
// ---------------------------------------------------------------------------

#[test]
fn initialize_stamps_storage_schema_version() {
    let (_env, client, _admin, _issuer) = setup!();

    assert_eq!(client.storage_version(), STORAGE_SCHEMA_VERSION);
}

#[test]
fn storage_version_is_zero_before_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TokenizationContract);
    let client = TokenizationContractClient::new(&env, &contract_id);

    assert_eq!(client.storage_version(), 0);
}

#[test]
fn reserved_gap_slots_are_reported_and_bounded() {
    let (_env, client, _admin, _issuer) = setup!();

    assert_eq!(client.storage_gap_slots(), STORAGE_GAP_SLOTS);
    assert!(STORAGE_GAP_SLOTS > 0);
}

#[test]
fn gap_slot_predicate_covers_exactly_the_reserved_range() {
    let (_env, client, _admin, _issuer) = setup!();
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