//! Meta-Transaction Integration Tests — refactored onto `TestWorld`
//!
//! Meta-transactions allow a *relayer* to submit a signed user transaction on
//! the user's behalf, paying the fee while the user's intent is verified
//! on-chain.  These tests exercise the `meta_tx_relay` contract in combination
//! with `did_registry` (for signer identity) and `medical_records_stub` (as a
//! target contract whose functions get invoked via the relayer).

use integration_framework::prelude::*;

mod meta_tx_relay {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/meta_tx_relay.wasm"
    );
}

mod did_registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/did_registry.wasm"
    );
}

mod medical_records_stub {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/medical_records.wasm"
    );
}

// ─── helpers ────────────────────────────────────────────────────────────────

struct MetaTxContext<'e> {
    world:   TestWorld,
    relay:   meta_tx_relay::Client<'e>,
    did:     did_registry::Client<'e>,
    records: medical_records_stub::Client<'e>,
    admin:   Address,
    user:    Address,
    relayer: Address,
}

fn setup<'e>() -> MetaTxContext<'e> {
    let mut world = TestWorld::new();

    let did_addr     = world.register_contract("did_registry",     did_registry::WASM);
    let relay_addr   = world.register_contract("meta_tx_relay",    meta_tx_relay::WASM);
    let records_addr = world.register_contract("medical_records",  medical_records_stub::WASM);

    let env = unsafe { &*(world.env() as *const Env) };

    let relay   = meta_tx_relay::Client::new(env, &relay_addr);
    let did     = did_registry::Client::new(env, &did_addr);
    let records = medical_records_stub::Client::new(env, &records_addr);

    let admin   = world.new_account();
    let user    = world.new_account();
    let relayer = world.new_account();

    did.initialize(&admin);
    relay.initialize(&admin, &did_addr);
    records.initialize(&admin);

    // Register the user's DID so the relay can verify their identity.
    let did_doc = soroban_sdk::String::from_str(
        env,
        r#"{"id":"did:uzima:meta-user","controller":"did:uzima:meta-user"}"#,
    );
    did.register(&user, &did_doc);

    MetaTxContext { world, relay, did, records, admin, user, relayer }
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_relayer_submits_valid_meta_tx_on_behalf_of_user() {
    let ctx = setup();
    let _ = &ctx.world;

    let nonce     = 1u64;
    let target    = ctx.world.address_of("medical_records");
    let fn_name   = soroban_sdk::Symbol::new(ctx.world.env(), "add_record");
    let payload   = soroban_sdk::Bytes::from_slice(ctx.world.env(), b"encrypted-health-data");

    // The relay contract validates the DID, checks nonce, then dispatches.
    ctx.relay.relay(
        &ctx.relayer,
        &ctx.user,
        &nonce,
        &target,
        &fn_name,
        &payload,
    );

    // Verify the record was actually written.
    let stored = ctx.records.get_latest_record(&ctx.user);
    assert_eq!(stored, payload, "Medical record should be stored after relay dispatch");
}

#[test]
fn test_replay_attack_is_rejected() {
    let ctx = setup();
    let _ = &ctx.world;

    let nonce   = 1u64;
    let target  = ctx.world.address_of("medical_records");
    let fn_name = soroban_sdk::Symbol::new(ctx.world.env(), "add_record");
    let payload = soroban_sdk::Bytes::from_slice(ctx.world.env(), b"replay-payload");

    // First submission succeeds.
    ctx.relay.relay(&ctx.relayer, &ctx.user, &nonce, &target, &fn_name, &payload);

    // Resubmitting the same nonce must fail.
    let replay = ctx.relay.try_relay(
        &ctx.relayer,
        &ctx.user,
        &nonce,   // same nonce
        &target,
        &fn_name,
        &payload,
    );
    assert!(replay.is_err(), "Relay should reject a replayed nonce");
}

#[test]
fn test_relay_rejects_unregistered_did() {
    let ctx = setup();
    let _ = &ctx.world;

    let unknown = ctx.world.new_account(); // no DID registered
    let nonce   = 1u64;
    let target  = ctx.world.address_of("medical_records");
    let fn_name = soroban_sdk::Symbol::new(ctx.world.env(), "add_record");
    let payload = soroban_sdk::Bytes::from_slice(ctx.world.env(), b"bad-actor-data");

    let result = ctx.relay.try_relay(
        &ctx.relayer,
        &unknown,
        &nonce,
        &target,
        &fn_name,
        &payload,
    );
    assert!(
        result.is_err(),
        "Relay should reject meta-tx from a user without a registered DID"
    );
}

#[test]
fn test_nonce_increments_correctly_across_multiple_relays() {
    let ctx = setup();
    let _ = &ctx.world;

    let target  = ctx.world.address_of("medical_records");
    let fn_name = soroban_sdk::Symbol::new(ctx.world.env(), "add_record");

    for nonce in 1u64..=5 {
        let payload = soroban_sdk::Bytes::from_slice(
            ctx.world.env(),
            alloc::format!("record-{}", nonce).as_bytes(),
        );
        ctx.relay.relay(&ctx.relayer, &ctx.user, &nonce, &target, &fn_name, &payload);
    }

    let current_nonce = ctx.relay.get_nonce(&ctx.user);
    assert_eq!(current_nonce, 5u64, "Nonce should reflect the number of successful relays");
}

#[test]
fn test_relay_respects_out_of_order_nonce_rejection() {
    let ctx = setup();
    let _ = &ctx.world;

    let target  = ctx.world.address_of("medical_records");
    let fn_name = soroban_sdk::Symbol::new(ctx.world.env(), "add_record");
    let payload = soroban_sdk::Bytes::from_slice(ctx.world.env(), b"ooo");

    // Submit nonce 1 first.
    ctx.relay.relay(&ctx.relayer, &ctx.user, &1u64, &target, &fn_name, &payload);

    // Attempt to submit nonce 3 (skipping 2) should be rejected.
    let result = ctx.relay.try_relay(&ctx.relayer, &ctx.user, &3u64, &target, &fn_name, &payload);
    assert!(result.is_err(), "Relay should reject out-of-order nonces");
}

// ─── allow alloc in no_std context ──────────────────────────────────────────
extern crate alloc;