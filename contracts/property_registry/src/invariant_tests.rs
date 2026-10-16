//! State-machine invariants for `PropertyRegistry` (#1780).
//!
//! Each test drives a realistic sequence of operations and then asserts a
//! property that must hold regardless of the path taken. These complement the
//! example-based tests in `test.rs`: those check that documented flows work,
//! these check that *no* flow can break the register or the ownership split.
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Borrows the `Env` rather than owning it, because the generated client holds
/// a reference to it.
struct World<'a> {
    env: &'a Env,
    client: PropertyRegistryContractClient<'a>,
    registrar: Address,
    owner: Address,
    alice: Address,
    bob: Address,
    carol: Address,
}

fn world(env: &Env) -> World<'_> {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let registrar = Address::generate(env);
    let owner = Address::generate(env);
    let alice = Address::generate(env);
    let bob = Address::generate(env);
    let carol = Address::generate(env);

    let contract_id = env.register_contract(None, PropertyRegistryContract);
    let client = PropertyRegistryContractClient::new(env, &contract_id);
    client.initialize(&admin, &registrar);
    client.grant_role(&registrar, &REGISTRAR_ROLE);

    let id = String::from_str(env, "PROP-1");
    client.register_property(
        &id,
        &owner,
        &250_000_00,
        &String::from_str(env, "USD"),
        &String::from_str(env, "ipfs://deed"),
    );

    World {
        env,
        client,
        registrar,
        owner,
        alice,
        bob,
        carol,
    }
}

impl World<'_> {
    fn property_id(&self) -> String {
        String::from_str(self.env, "PROP-1")
    }

    fn currency(&self) -> String {
        String::from_str(self.env, "USD")
    }

    fn uri(&self) -> String {
        String::from_str(self.env, "ipfs://deed")
    }

    /// Total across the four tracked holders.
    fn tracked_total(&self) -> u128 {
        let id = self.property_id();
        self.client.ownership_of(&id, &self.owner)
            + self.client.ownership_of(&id, &self.alice)
            + self.client.ownership_of(&id, &self.bob)
            + self.client.ownership_of(&id, &self.carol)
    }

    /// A representative multi-party workload: fractionalisation, revaluation,
    /// dilution and a handover, including rejected operations.
    fn churn(&self) {
        let id = self.property_id();

        self.client
            .record_ownership(&id, &self.alice, &3_000);
        self.client.record_ownership(&id, &self.bob, &2_000);
        self.client.update_valuation(&id, &300_000_00);
        self.client.record_ownership(&id, &self.carol, &1_000);
        self.client.transfer_property(&id, &self.alice);

        // Rejected operations must leave nothing behind.
        assert!(self
            .client
            .try_record_ownership(&id, &self.bob, &u128::MAX)
            .is_err());
        assert!(self
            .client
            .try_register_property(&id, &self.owner, &1, &self.currency(), &self.uri())
            .is_err());
        assert!(self
            .client
            .try_transfer_property(&id, &self.property_id())
            .is_err());
    }
}

/// The ownership split always totals exactly 100%, no matter the path taken.
/// This is the core register invariant.
#[test]
fn invariant_shares_always_total_one_hundred_percent() {
    let env = Env::default();
    let w = world(&env);
    w.churn();

    assert_eq!(w.tracked_total(), BASIS_POINTS_SCALE);
}

/// No single share can exceed 100%.
#[test]
fn invariant_no_share_exceeds_one_hundred_percent() {
    let env = Env::default();
    let w = world(&env);
    w.churn();

    let id = w.property_id();
    for holder in [w.owner.clone(), w.alice.clone(), w.bob.clone(), w.carol.clone()] {
        assert!(w.client.ownership_of(&id, &holder) <= BASIS_POINTS_SCALE);
    }
}

/// The owner of record is always a holder: their position is never silently
/// dropped to zero without the owner of record moving too.
#[test]
fn invariant_owner_of_record_is_always_a_holder() {
    let env = Env::default();
    let w = world(&env);

    let id = w.property_id();
    let buyer = Address::generate(&env);

    w.client.record_ownership(&id, &w.alice, &5_000);
    w.client.transfer_property(&id, &buyer);

    let new_owner = w.client.get_property(&id).owner;
    assert!(w.client.ownership_of(&id, &new_owner) > 0);
}

/// `holder_count` never drifts: it is recomputed from share transitions, so a
/// repeated transfer to an existing holder must not double-count them.
#[test]
fn invariant_holder_count_tracks_distinct_holders() {
    let env = Env::default();
    let w = world(&env);
    let id = w.property_id();

    assert_eq!(w.client.get_property(&id).holder_count, 1);

    w.client.record_ownership(&id, &w.alice, &1_000);
    w.client.record_ownership(&id, &w.alice, &2_000);
    assert_eq!(w.client.get_property(&id).holder_count, 2);

    w.client.record_ownership(&id, &w.bob, &1_000);
    assert_eq!(w.client.get_property(&id).holder_count, 3);
}

/// Property count is monotone and never double-counts a re-registration.
#[test]
fn invariant_property_count_tracks_registrations() {
    let env = Env::default();
    let w = world(&env);

    assert_eq!(w.client.total_properties(), 1);

    let second = String::from_str(&env, "PROP-2");
    w.client
        .register_property(&second, &w.owner, &1, &w.currency(), &w.uri());
    assert_eq!(w.client.total_properties(), 2);

    assert!(w
        .client
        .try_register_property(&second, &w.owner, &1, &w.currency(), &w.uri())
        .is_err());
    assert_eq!(w.client.total_properties(), 2);
}

/// Revaluation never touches the ownership split.
#[test]
fn invariant_revaluation_does_not_disturb_ownership() {
    let env = Env::default();
    let w = world(&env);
    let id = w.property_id();

    w.client.record_ownership(&id, &w.alice, &4_000);
    let before = w.tracked_total();

    w.client.update_valuation(&id, &u128::MAX);
    w.client.update_valuation(&id, &1);

    assert_eq!(w.tracked_total(), before);
    assert_eq!(w.client.get_property(&id).valuation, 1);
}

/// Failed operations are inert: the split and the record are identical before
/// and after.
#[test]
fn invariant_rejected_calls_do_not_mutate_state() {
    let env = Env::default();
    let w = world(&env);
    let id = w.property_id();

    w.client.record_ownership(&id, &w.alice, &2_500);

    let total_before = w.tracked_total();
    let holders_before = w.client.get_property(&id).holder_count;
    let valuation_before = w.client.get_property(&id).valuation;

    assert!(w.client.try_record_ownership(&id, &w.bob, &u128::MAX).is_err());
    assert!(w.client.try_transfer_property(&id, &w.owner).is_err());
    assert!(w
        .client
        .try_update_valuation(&String::from_str(&env, "PROP-MISSING"), &1)
        .is_err());
    assert!(w
        .client
        .try_register_property(&id, &w.owner, &1, &w.currency(), &w.uri())
        .is_err());

    assert_eq!(w.tracked_total(), total_before);
    assert_eq!(w.client.get_property(&id).holder_count, holders_before);
    assert_eq!(w.client.get_property(&id).valuation, valuation_before);
}

/// Authority is conserved: there is always exactly one admin after a transfer.
#[test]
fn invariant_always_exactly_one_admin_after_transfer() {
    let env = Env::default();
    let w = world(&env);

    assert_eq!(w.client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);

    let successor = Address::generate(&env);
    w.client.transfer_admin(&successor);

    assert_eq!(w.client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);
    assert!(w.client.has_role(&successor, &DEFAULT_ADMIN_ROLE));
}

/// Withdrawing the registrar role stops all register amendments; restoring it
/// resumes exactly where the ledger left off.
#[test]
fn invariant_role_withdrawal_halts_amendments() {
    let env = Env::default();
    let w = world(&env);
    let id = w.property_id();

    w.client.update_valuation(&id, &275_000_00);

    w.client.revoke_role(&w.registrar, &REGISTRAR_ROLE);
    assert!(w.client.try_update_valuation(&id, &1).is_err());
    assert_eq!(w.client.get_property(&id).valuation, 275_000_00);

    w.client.grant_role(&w.registrar, &REGISTRAR_ROLE);
    w.client.update_valuation(&id, &1);
    assert_eq!(w.client.get_property(&id).valuation, 1);
}

/// Pausing is global: no role, however privileged, can move state while it holds.
#[test]
fn invariant_pause_is_total() {
    let env = Env::default();
    let w = world(&env);
    let id = w.property_id();

    w.client.record_ownership(&id, &w.alice, &5_000);
    w.client.pause();

    assert!(w.client.try_update_valuation(&id, &1).is_err());
    assert!(w
        .client
        .try_record_ownership(&id, &w.bob, &1)
        .is_err());
    assert!(w.client.try_transfer_property(&id, &w.bob).is_err());

    assert_eq!(w.tracked_total(), BASIS_POINTS_SCALE);
}