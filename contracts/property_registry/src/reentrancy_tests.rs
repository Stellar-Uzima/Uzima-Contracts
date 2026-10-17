//! Reentrancy regression tests (issue #1772).
//!
//! These do not simulate reentrancy by calling the contract twice from a test
//! function — that would run sequentially and prove nothing. Instead
//! [`ReentryProbe`] is a real contract that calls back into the registry *while
//! its own invocation frame is still open*, which is what an attacker-controlled
//! `SorobanAuthorization` contract would do during `require_auth`.
//!
//! The second call is issued with `try_` so a refusal is observable as a value
//! rather than aborting the outer frame, which is what lets each test assert
//! *which* call was refused and what the ledger looks like afterwards.
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, Address, Env, String,
};

/// A contract that performs an outer operation and then attempts a second one
/// against the same registry before its own frame closes.
#[contract]
struct ReentryProbe;

#[contractimpl]
impl ReentryProbe {
    /// Transfer the property to `outer_to`, then re-enter with `inner_to`.
    ///
    /// Returns `(outer_succeeded, inner_was_refused)`.
    pub fn probe(
        env: Env,
        registry: Address,
        property_id: String,
        outer_to: Address,
        inner_to: Address,
    ) -> (bool, bool) {
        let client = PropertyRegistryContractClient::new(&env, &registry);

        let outer_succeeded = client.try_transfer_property(&property_id, &outer_to).is_ok();
        let inner_was_refused = client.try_transfer_property(&property_id, &inner_to).is_err();

        (outer_succeeded, inner_was_refused)
    }

    /// Record a share of `amount`, then re-enter with a smaller amount.
    ///
    /// Returns `(outer_succeeded, inner_was_refused)`.
    pub fn probe_ownership(
        env: Env,
        registry: Address,
        property_id: String,
        inner_holder: Address,
        amount: u128,
    ) -> (bool, bool) {
        let client = PropertyRegistryContractClient::new(&env, &registry);

        let outer_succeeded = client
            .try_record_ownership(&property_id, &inner_holder, &amount)
            .is_ok();
        let inner_was_refused = client
            .try_record_ownership(&property_id, &inner_holder, &1)
            .is_err();

        (outer_succeeded, inner_was_refused)
    }
}

macro_rules! setup {
    () => {{
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let registrar = Address::generate(&env);
        let owner = Address::generate(&env);

        let registry = env.register_contract(None, PropertyRegistryContract);
        let client = PropertyRegistryContractClient::new(&env, &registry);
        client.initialize(&admin, &registrar);

        let id = String::from_str(&env, "PROP-1");
        client.register_property(
            &id,
            &owner,
            &250_000_00,
            &String::from_str(&env, "USD"),
            &String::from_str(&env, "ipfs://deed"),
        );

        let probe_id = env.register_contract(None, ReentryProbe);
        let probe = ReentryProbeClient::new(&env, &probe_id);

        (env, registry, client, probe, owner, id)
    }};
}

// ---------------------------------------------------------------------------
// The guard
// ---------------------------------------------------------------------------

#[test]
fn guard_is_clear_on_a_freshly_initialized_registry() {
    let (_env, registry, _client, _probe, _owner, _id) = setup!();

    let view = PropertyRegistryContractClient::new(&_env, &registry);
    assert!(!view.reentrancy_locked());
}

#[test]
fn reentrant_transfer_property_is_refused() {
    let (env, registry, client, probe, _owner, id) = setup!();

    let outer_to = Address::generate(&env);
    let inner_to = Address::generate(&env);

    let (outer_ok, inner_refused) = probe.probe(&registry, &id, &outer_to, &inner_to);

    assert!(outer_ok, "the outer call should have succeeded");
    assert!(inner_refused, "the re-entrant call should have been refused");

    // The ledger shows exactly one transfer, not two.
    assert_eq!(client.get_property(&id).owner, outer_to);
    assert_eq!(client.ownership_of(&id, &outer_to), BASIS_POINTS_SCALE);
    assert_eq!(client.ownership_of(&id, &inner_to), 0);
}

#[test]
fn reentrant_record_ownership_is_refused() {
    let (env, registry, client, probe, owner, id) = setup!();

    let investor = Address::generate(&env);
    let (outer_ok, inner_refused) = probe.probe_ownership(&registry, &id, &investor, &2_500);

    assert!(outer_ok);
    assert!(inner_refused);

    // The outer amount landed once; the re-entrant basis point did not.
    assert_eq!(client.ownership_of(&id, &investor), 2_500);
    assert_eq!(client.ownership_of(&id, &owner), 7_500);
    assert_eq!(client.get_property(&id).holder_count, 2);
}

#[test]
fn guard_is_released_after_a_successful_guarded_call() {
    let (env, registry, client, _probe, _owner, id) = setup!();

    let buyer = Address::generate(&env);
    client.transfer_property(&id, &buyer);

    assert!(!PropertyRegistryContractClient::new(&env, &registry).reentrancy_locked());
}

#[test]
fn guard_is_released_when_a_guarded_call_fails() {
    // This is the assumption the whole design rests on: Soroban's transactional
    // execution rolls the guard flag back along with everything else when an
    // entry point returns `Err`. If it did not, the first rejected call would
    // brick the contract.
    let (env, registry, client, _probe, owner, id) = setup!();

    assert_eq!(
        client.try_record_ownership(&id, &owner, &1_000),
        Err(Ok(Error::InvalidArgument))
    );
    assert_eq!(
        client.try_transfer_property(&id, &owner),
        Err(Ok(Error::InvalidArgument))
    );

    assert!(!PropertyRegistryContractClient::new(&env, &registry).reentrancy_locked());

    // And a subsequent valid guarded call must still be possible.
    let buyer = Address::generate(&env);
    client.transfer_property(&id, &buyer);
    assert_eq!(client.get_property(&id).owner, buyer);
}

#[test]
fn guard_is_released_when_a_guarded_call_is_rejected_mid_flight() {
    // Rejection *after* the guard is taken — the over-commit path — must also
    // release, otherwise a single bad `record_ownership` would wedge the
    // registry for every property.
    let (env, registry, client, _probe, owner, id) = setup!();

    let investor = Address::generate(&env);
    client.record_ownership(&id, &investor, &6_000);

    assert_eq!(
        client.try_record_ownership(&id, &investor, &4_001),
        Err(Ok(Error::OwnershipExceedsShare))
    );
    assert!(!PropertyRegistryContractClient::new(&env, &registry).reentrancy_locked());

    // Still fully operational afterwards.
    client.record_ownership(&id, &investor, &1_000);
    assert_eq!(client.ownership_of(&id, &investor), 7_000);
    let _ = owner;
}

#[test]
fn two_sequential_guarded_calls_both_succeed() {
    // The guard must serialise, not deadlock.
    let (env, _registry, client, _probe, _owner, id) = setup!();

    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.transfer_property(&id, &first);
    client.transfer_property(&id, &second);

    assert_eq!(client.get_property(&id).owner, second);
    assert_eq!(client.ownership_of(&id, &first), 0);
    assert_eq!(client.ownership_of(&id, &second), BASIS_POINTS_SCALE);
}

#[test]
fn unguarded_entry_points_are_unaffected_by_the_guard() {
    // `update_valuation` touches one key and deliberately does not take the
    // guard; it must keep working regardless of the guard's state.
    let (env, registry, client, _probe, _owner, id) = setup!();

    client.update_valuation(&id, &1);
    client.update_valuation(&id, &2);

    assert_eq!(client.get_property(&id).valuation, 2);
    assert!(!PropertyRegistryContractClient::new(&env, &registry).reentrancy_locked());
}