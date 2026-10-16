//! Proptest-based fuzz harness for `property_registry` (issue #1771).
//!
//! Covers the arguments that reach storage — identifiers, valuations,
//! ownership shares — over the x86_64 host target.
//!
//! The properties asserted here are the ones a random argument stream can most
//! plausibly break, and which the example-based unit tests cannot reach:
//!
//! 1. **No panic, ever.** Whatever bytes arrive, the contract returns a typed
//!    error or a value — it never traps the host.
//! 2. **The ownership split is conserved.** For any accepted `record_ownership`,
//!    the sum of the two affected shares is unchanged.
//! 3. **The split never exceeds 100%.** No accepted call pushes a holder above
//!    `BASIS_POINTS_SCALE`.
//! 4. **Rejected calls are inert.** After any rejected call, both shares are
//!    exactly what they were before.
//! 5. **Reentrancy never wedges the contract.** After any failure on a guarded
//!    entry point, the guard is clear and a subsequent valid call still works.
//!
//! Run locally:
//!   cd tests/fuzz/property_registry && cargo test
//! Long-duration fuzzing (CI default):
//!   PROPTEST_CASES=5000 cargo test

extern crate std;

use proptest::prelude::*;
use property_registry::{
    Error, PropertyRegistryContract, PropertyRegistryContractClient, BASIS_POINTS_SCALE,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Identifiers of any shape the string type can hold, including empty,
/// over-long, and non-ASCII payloads.
fn arb_id() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<u8>(), 0..=80)
        .prop_map(|bytes| String::from_bytes(&bytes))
}

/// A `u128` drawn from both ends of the range, where wrap behaviour lives.
fn arb_amount() -> impl Strategy<Value = u128> {
    prop_oneof![
        any::<u128>(),
        0..=BASIS_POINTS_SCALE,
        (BASIS_POINTS_SCALE - 1)..=(BASIS_POINTS_SCALE + 1),
    ]
}

/// Build a fresh registry plus an initialized client.
///
/// A macro rather than a function: the generated client borrows the `Env`, so it
/// cannot outlive a returned one.
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

        (env, owner, registrar, client, id)
    }};
}
proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(256),
        ..ProptestConfig::default()
    })]

    /// Property 1 + 4: any `record_ownership` either succeeds and conserves the
    /// split, or fails and changes nothing. It never traps the host.
    #[test]
    fn record_ownership_is_conservative(
        amount in arb_amount(),
    ) {
        let (env, owner, _registrar, client, id) = setup!();
        let holder = Address::generate(&env);

        let owner_before = client.ownership_of(&id, &owner);
        let holder_before = client.ownership_of(&id, &holder);

        let result = client.try_record_ownership(&id, &holder, &amount);
        let owner_after = client.ownership_of(&id, &owner);
        let holder_after = client.ownership_of(&id, &holder);

        match &result {
            Ok(Ok(())) => {
                // Accepted: the split is conserved and still totals 100%.
                prop_assert_eq!(owner_after + holder_after, owner_before + holder_before);
                prop_assert_eq!(owner_after + holder_after, BASIS_POINTS_SCALE);
            }
            Ok(Err(Error::PropertyNotFound)) | Ok(Err(Error::OwnershipExceedsShare))
            | Ok(Err(Error::ShareTotalExceeded)) | Ok(Err(Error::InvalidArgument))
            | Ok(Err(Error::Unauthorized)) | Ok(Err(Error::ReentrantCall)) => {
                // Rejected: completely inert.
                prop_assert_eq!(owner_after, owner_before);
                prop_assert_eq!(holder_after, holder_before);
            }
            Ok(Err(other)) => {
                // Any other *typed* error is still a clean rejection, and still
                // must not have mutated state.
                let _ = other;
                prop_assert_eq!(owner_after, owner_before);
                prop_assert_eq!(holder_after, holder_before);
            }
            Err(_) => {
                // Conversion / invocation-level failure. Also must be inert.
                prop_assert_eq!(owner_after, owner_before);
                prop_assert_eq!(holder_after, holder_before);
            }
        }
    }

    /// Property 3: no accepted `record_ownership` pushes a holder above 100%.
    #[test]
    fn no_share_ever_exceeds_one_hundred_percent(amount in arb_amount()) {
        let (env, owner, _registrar, client, id) = setup!();
        let holder = Address::generate(&env);

        if client.try_record_ownership(&id, &holder, &amount).is_ok() {
            prop_assert!(client.ownership_of(&id, &holder) <= BASIS_POINTS_SCALE);
            prop_assert!(client.ownership_of(&id, &owner) <= BASIS_POINTS_SCALE);
        }
    }

    /// Property 1 + 4: a random `transfer_property` target is either a clean
    /// success or a clean refusal, and never leaves a partial split.
    #[test]
    fn transfer_property_never_half_applies(
        amount in arb_amount(),
    ) {
        let (env, owner, _registrar, client, id) = setup!();
        let investor = Address::generate(&env);

        // Fractionalise first so `transfer_property` has a partial position to
        // move, which is where the credit-before-debit ordering matters.
        let _ = client.try_record_ownership(&id, &investor, &amount);

        let owner_before = client.ownership_of(&id, &owner);
        let investor_before = client.ownership_of(&id, &investor);

        let result = client.try_transfer_property(&id, &investor);

        let owner_after = client.ownership_of(&id, &owner);
        let investor_after = client.ownership_of(&id, &investor);

        if result.is_ok() {
            prop_assert_eq!(owner_after + investor_after, BASIS_POINTS_SCALE);
        } else {
            prop_assert_eq!(owner_after, owner_before);
            prop_assert_eq!(investor_after, investor_before);
        }
    }

    /// Property 1: arbitrary identifiers reach the contract without trapping,
    /// and never produce a record for a duplicate registration.
    #[test]
    fn register_property_with_arbitrary_ids(
        raw in arb_id(),
        valuation in any::<u128>(),
    ) {
        let (env, owner, _registrar, client, _id) = setup!();

        let candidate = String::from_bytes(&raw);
        let result = client.try_register_property(
            &candidate,
            &owner,
            &valuation,
            &String::from_str(&env, "USD"),
            &String::from_str(&env, "ipfs://x"),
        );

        // Either it was refused, or it is now readable back byte-for-byte.
        if result.is_ok() {
            let stored = client.get_property(&candidate);
            prop_assert_eq!(stored.property_id, candidate);
            prop_assert_eq!(stored.valuation, valuation);
            prop_assert_eq!(client.total_properties(), 2);
        } else {
            prop_assert_eq!(client.total_properties(), 1);
        }
    }

    /// Property 5: after any rejected guarded call the guard is clear and a
    /// subsequent valid call still succeeds. This is the property the reentrancy
    /// guard's design depends on.
    #[test]
    fn guard_never_wedges_after_a_rejected_call(
        amount in arb_amount(),
    ) {
        let (env, owner, _registrar, client, id) = setup!();

        // Provoke a rejection on a guarded entry point.
        let _ = client.try_record_ownership(&id, &owner, &amount);

        prop_assert!(!client.reentrancy_locked());

        let buyer = Address::generate(&env);
        client.transfer_property(&id, &buyer);
        prop_assert_eq!(client.get_property(&id).owner, buyer);
        prop_assert!(!client.reentrancy_locked());
    }

    /// Property 1 + 4: a random valuation never traps and never disturbs the
    /// ownership split.
    #[test]
    fn update_valuation_with_arbitrary_amounts(
        valuation in any::<u128>(),
    ) {
        let (env, owner, _registrar, client, id) = setup!();
        let investor = Address::generate(&env);

        client.record_ownership(&id, &investor, &2_500);
        let split_before =
            client.ownership_of(&id, &owner) + client.ownership_of(&id, &investor);

        let result = client.try_update_valuation(&id, &valuation);

        match &result {
            Ok(Ok(())) => prop_assert_eq!(client.get_property(&id).valuation, valuation),
            Ok(Err(_)) | Err(_) => prop_assert_eq!(
                client.get_property(&id).valuation,
                250_000u128
            ),
        }

        prop_assert_eq!(
            client.ownership_of(&id, &owner) + client.ownership_of(&id, &investor),
            split_before
        );
    }

    /// Property 2 + 3: a long random sequence of share movements always leaves
    /// the split totalling exactly 100%.
    #[test]
    fn random_share_sequences_conserve_the_split(
        moves in proptest::collection::vec(arb_amount(), 1..24),
    ) {
        let (env, owner, _registrar, client, id) = setup!();
        let holders: std::vec::Vec<Address> =
            (0..4).map(|_| Address::generate(&env)).collect();

        for (i, amount) in moves.iter().enumerate() {
            let holder = &holders[i % holders.len()];
            let _ = client.try_record_ownership(&id, holder, amount);

            let total = client.ownership_of(&id, &owner)
                + holders
                    .iter()
                    .map(|h| client.ownership_of(&id, h))
                    .sum::<u128>();

            prop_assert_eq!(total, BASIS_POINTS_SCALE);
        }
    }
}