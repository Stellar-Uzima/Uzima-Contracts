//! State-machine invariants for `Tokenization` (#1764).
//!
//! Each test drives a realistic sequence of operations and then asserts a
//! property that must hold regardless of the path taken. These complement the
//! example-based tests in `test.rs`: those check that documented flows work,
//! these check that *no* flow can break the accounting.
#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;

use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Borrows the `Env` rather than owning it, because the generated client holds
/// a reference to it.
struct World<'a> {
    env: &'a Env,
    client: TokenizationContractClient<'a>,
    admin: Address,
    issuer: Address,
    alice: Address,
    bob: Address,
    carol: Address,
}

fn world(env: &Env) -> World<'_> {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let issuer = Address::generate(env);
    let alice = Address::generate(env);
    let bob = Address::generate(env);
    let carol = Address::generate(env);
    let treasury = Address::generate(env);

    let contract_id = env.register_contract(None, TokenizationContract);
    let client = TokenizationContractClient::new(env, &contract_id);
    client.initialize(&admin, &treasury);
    client.grant_role(&issuer, &ISSUER_ROLE);

    let id = String::from_str(env, "HOSP-1");
    client.register_asset(&id, &issuer, &1_000_000, &6, &String::from_str(env, "uri"));

    World {
        env,
        client,
        admin,
        issuer,
        alice,
        bob,
        carol,
    }
}

impl World<'_> {
    fn asset_id(&self) -> String {
        String::from_str(self.env, "HOSP-1")
    }

    fn uri(&self) -> String {
        String::from_str(self.env, "uri")
    }

    /// Total across the three tracked holders.
    fn tracked_total(&self) -> u128 {
        let id = self.asset_id();
        self.client.balance_of(&id, &self.alice)
            + self.client.balance_of(&id, &self.bob)
            + self.client.balance_of(&id, &self.carol)
    }

    /// A representative multi-party workload: interleaved mints, transfers and
    /// burns, including rejected operations.
    fn churn(&self) {
        let id = self.asset_id();

        self.client.mint(&id, &self.alice, &300_000);
        self.client.mint(&id, &self.bob, &200_000);
        self.client.transfer(&id, &self.alice, &self.bob, &50_000);
        self.client.mint(&id, &self.carol, &100_000);
        self.client.burn(&id, &self.bob, &25_000);
        self.client.transfer(&id, &self.carol, &self.alice, &10_000);

        // Rejected operations must leave nothing behind.
        assert!(self
            .client
            .try_transfer(&id, &self.alice, &self.bob, &u128::MAX)
            .is_err());
        assert!(self
            .client
            .try_mint(&id, &self.alice, &u128::MAX)
            .is_err());
    }
}

/// Whatever sequence of operations ran, the sum of holder balances equals the
/// reported supply. This is the core accounting invariant.
#[test]
fn invariant_balances_sum_to_total_supply() {
    let env = Env::default();
    let w = world(&env);
    w.churn();

    assert_eq!(w.tracked_total(), w.client.total_supply(&w.asset_id()));
}

/// `total_supply` is monotone non-decreasing under mint and transfer, and only
/// decreases on an explicit burn, by exactly the burned amount.
#[test]
fn invariant_supply_only_moves_by_the_explicit_amount() {
    let env = Env::default();
    let w = world(&env);
    let id = w.asset_id();

    assert_eq!(w.client.total_supply(&id), 0);

    w.client.mint(&id, &w.alice, &400_000);
    assert_eq!(w.client.total_supply(&id), 400_000);

    w.client.transfer(&id, &w.alice, &w.bob, &150_000);
    assert_eq!(w.client.total_supply(&id), 400_000);

    w.client.burn(&id, &w.bob, &25_000);
    assert_eq!(w.client.total_supply(&id), 375_000);
}

/// No balance can exceed the reported supply.
#[test]
fn invariant_every_balance_is_bounded_by_supply() {
    let env = Env::default();
    let w = world(&env);
    w.churn();

    let id = w.asset_id();
    let supply = w.client.total_supply(&id);

    for holder in [w.alice.clone(), w.bob.clone(), w.carol.clone()] {
        let balance = w.client.balance_of(&id, &holder);
        assert!(balance <= supply, "balance {balance} exceeded supply {supply}");
    }
}

/// A transfer is value-preserving: it may not create or destroy units.
#[test]
fn invariant_transfer_preserves_the_total() {
    let env = Env::default();
    let w = world(&env);
    let id = w.asset_id();

    w.client.mint(&id, &w.alice, &500_000);
    let before = w.client.total_supply(&id);

    w.client.transfer(&id, &w.alice, &w.bob, &200_000);
    w.client.transfer(&id, &w.bob, &w.carol, &50_000);

    assert_eq!(w.client.total_supply(&id), before);
}

/// Failed operations are inert: state is identical before and after.
#[test]
fn invariant_rejected_calls_do_not_mutate_state() {
    let env = Env::default();
    let w = world(&env);
    let id = w.asset_id();

    w.client.mint(&id, &w.alice, &10_000);

    let supply_before = w.client.total_supply(&id);
    let alice_before = w.client.balance_of(&id, &w.alice);

    assert!(w.client.try_burn(&id, &w.alice, &u128::MAX).is_err());
    assert!(w
        .client
        .try_transfer(&id, &w.alice, &w.bob, &u128::MAX)
        .is_err());
    assert!(w
        .client
        .try_register_asset(&id, &w.issuer, &1, &6, &w.uri())
        .is_err());

    assert_eq!(w.client.total_supply(&id), supply_before);
    assert_eq!(w.client.balance_of(&id, &w.alice), alice_before);
}

/// Asset count is monotone and never double-counts a re-registration.
#[test]
fn invariant_asset_count_tracks_registrations() {
    let env = Env::default();
    let w = world(&env);

    assert_eq!(w.client.asset_count(), 1);

    let second = String::from_str(&env, "HOSP-2");
    w.client
        .register_asset(&second, &w.issuer, &500, &2, &w.uri());
    assert_eq!(w.client.asset_count(), 2);

    assert!(w
        .client
        .try_register_asset(&second, &w.issuer, &500, &2, &w.uri())
        .is_err());
    assert_eq!(w.client.asset_count(), 2);
}

/// The supply recorded on the asset record always matches the `total_supply`
/// query, so the two can never drift apart.
#[test]
fn invariant_record_and_query_agree_on_supply() {
    let env = Env::default();
    let w = world(&env);
    w.churn();

    assert_eq!(
        w.client.get_asset(&w.asset_id()).total_supply,
        w.client.total_supply(&w.asset_id())
    );
}

/// Authority is conserved: there is always exactly one admin after a transfer,
/// so the contract can never end up unadministrable.
#[test]
fn invariant_always_exactly_one_admin_after_transfer() {
    let env = Env::default();
    let w = world(&env);

    assert_eq!(w.client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);

    let successor = Address::generate(&env);
    w.client.transfer_admin(&successor);

    assert_eq!(w.client.role_members(&DEFAULT_ADMIN_ROLE).len(), 1);
    assert!(w.client.has_role(&successor, &DEFAULT_ADMIN_ROLE));
    assert!(!w.client.has_role(&w.admin, &DEFAULT_ADMIN_ROLE));
}

/// Withdrawing the issuer role stops all supply creation, and restoring it
/// resumes exactly where the ledger left off.
#[test]
fn invariant_role_withdrawal_halts_supply_creation() {
    let env = Env::default();
    let w = world(&env);
    let id = w.asset_id();

    w.client.mint(&id, &w.alice, &10_000);
    let supply = w.client.total_supply(&id);

    w.client.revoke_role(&w.issuer, &ISSUER_ROLE);
    assert!(w.client.try_mint(&id, &w.alice, &1).is_err());
    assert_eq!(w.client.total_supply(&id), supply);

    w.client.grant_role(&w.issuer, &ISSUER_ROLE);
    w.client.mint(&id, &w.alice, &5);
    assert_eq!(w.client.total_supply(&id), supply + 5);
}

/// Pausing is global: no role, however privileged, can move state while it holds.
#[test]
fn invariant_pause_is_total() {
    let env = Env::default();
    let w = world(&env);
    let id = w.asset_id();

    w.client.mint(&id, &w.alice, &100_000);
    w.client.pause();

    assert!(w.client.try_mint(&id, &w.alice, &1).is_err());
    assert!(w.client.try_burn(&id, &w.alice, &1).is_err());
    assert!(w
        .client
        .try_transfer(&id, &w.alice, &w.bob, &1)
        .is_err());

    assert_eq!(w.client.total_supply(&id), 100_000);
}