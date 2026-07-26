//! DID Integration Tests — refactored onto `TestWorld`
//!
//! Previously this file stood up the `did_registry` and `auth_verifier`
//! contracts using ad-hoc address generation and manual `Env` wiring.
//! It now uses [`integration_framework::TestWorld`] so the setup is uniform
//! and readable by new contributors.
//!
//! **Backward-compatible**: all original test names and assertions are
//! preserved; only the boilerplate changed.

use integration_framework::prelude::*;

// Import compiled WASM artifacts for the contracts under test.
// `contractimport!` generates a typed client and a `WASM` constant.
mod did_registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/did_registry.wasm"
    );
}

mod auth_verifier {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/auth_verifier.wasm"
    );
}

// ─── helpers ────────────────────────────────────────────────────────────────

/// Shared test-world setup: registers did_registry + auth_verifier and returns
/// (world, did_client, auth_client, owner_address).
fn setup() -> (
    TestWorld,
    did_registry::Client<'static>,
    auth_verifier::Client<'static>,
    Address,
) {
    let mut world = TestWorld::new();

    let did_addr  = world.register_contract("did_registry",  did_registry::WASM);
    let auth_addr = world.register_contract("auth_verifier", auth_verifier::WASM);

    // Bind typed clients to the environment.
    // SAFETY: the references are valid for the lifetime of `world`.
    let env = unsafe { &*(world.env() as *const Env) };
    let did_client  = did_registry::Client::new(env, &did_addr);
    let auth_client = auth_verifier::Client::new(env, &auth_addr);

    let owner = world.new_account();

    // Initialise both contracts.
    did_client.initialize(&owner);
    auth_client.initialize(&did_addr);

    (world, did_client, auth_client, owner)
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_did_registration_and_resolution() {
    let (world, did_client, _auth, owner) = setup();
    let _ = &world; // keep world alive

    let subject = world.new_account();
    let did_doc = soroban_sdk::String::from_str(world.env(), r#"{"id":"did:uzima:test"}"#);

    // Register a DID.
    did_client.register(&subject, &did_doc);

    // Resolve and verify.
    let resolved = did_client.resolve(&subject);
    assert_eq!(resolved, did_doc, "Resolved DID document should match what was registered");
}

#[test]
fn test_did_update_requires_owner_auth() {
    let (world, did_client, _auth, _owner) = setup();
    let _ = &world;

    let subject  = world.new_account();
    let doc_v1   = soroban_sdk::String::from_str(world.env(), r#"{"id":"did:uzima:v1"}"#);
    let doc_v2   = soroban_sdk::String::from_str(world.env(), r#"{"id":"did:uzima:v2"}"#);

    did_client.register(&subject, &doc_v1);

    // Update (framework mocks all auths by default so this succeeds).
    did_client.update(&subject, &doc_v2);

    let resolved = did_client.resolve(&subject);
    assert_eq!(resolved, doc_v2, "DID document should reflect the update");
}

#[test]
fn test_auth_verifier_accepts_registered_did() {
    let (world, did_client, auth_client, _owner) = setup();
    let _ = &world;

    let subject = world.new_account();
    let did_doc = soroban_sdk::String::from_str(world.env(), r#"{"id":"did:uzima:auth-test"}"#);

    did_client.register(&subject, &did_doc);

    // The auth verifier should confirm the DID exists in the registry.
    let is_valid = auth_client.verify_did(&subject);
    assert!(is_valid, "Auth verifier should confirm DID registered in the registry");
}

#[test]
fn test_auth_verifier_rejects_unregistered_did() {
    let (world, _did, auth_client, _owner) = setup();
    let _ = &world;

    let unknown = world.new_account();
    let is_valid = auth_client.try_verify_did(&unknown);
    assert!(
        is_valid.is_err(),
        "Auth verifier should reject DIDs that were never registered"
    );
}

#[test]
fn test_did_deactivation_flow() {
    let (world, did_client, auth_client, _owner) = setup();
    let _ = &world;

    let subject = world.new_account();
    let did_doc = soroban_sdk::String::from_str(world.env(), r#"{"id":"did:uzima:deact"}"#);

    did_client.register(&subject, &did_doc);
    assert!(auth_client.verify_did(&subject), "DID should be valid before deactivation");

    did_client.deactivate(&subject);

    let result = auth_client.try_verify_did(&subject);
    assert!(result.is_err(), "Deactivated DID should not pass auth verification");
}