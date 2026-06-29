//! AI Integration Tests — refactored onto `TestWorld`
//!
//! Tests the cross-contract flow between `ai_oracle` (which surfaces model
//! inference results on-chain) and `knowledge_verifier` (which gates access
//! behind a DID check).  The original test spun up each contract with
//! separate `Env` instances and manually threaded addresses; `TestWorld` makes
//! this a single shared environment with named lookups.

use integration_framework::prelude::*;

mod ai_oracle {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/ai_oracle.wasm"
    );
}

mod knowledge_verifier {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/knowledge_verifier.wasm"
    );
}

mod did_registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/did_registry.wasm"
    );
}

// ─── helpers ────────────────────────────────────────────────────────────────

struct AiTestContext<'e> {
    world:    TestWorld,
    oracle:   ai_oracle::Client<'e>,
    verifier: knowledge_verifier::Client<'e>,
    did:      did_registry::Client<'e>,
    admin:    Address,
    provider: Address,
}

fn setup<'e>() -> AiTestContext<'e> {
    let mut world = TestWorld::new();

    let did_addr      = world.register_contract("did_registry",      did_registry::WASM);
    let oracle_addr   = world.register_contract("ai_oracle",         ai_oracle::WASM);
    let verifier_addr = world.register_contract("knowledge_verifier", knowledge_verifier::WASM);

    let env = unsafe { &*(world.env() as *const Env) };

    let did      = did_registry::Client::new(env, &did_addr);
    let oracle   = ai_oracle::Client::new(env, &oracle_addr);
    let verifier = knowledge_verifier::Client::new(env, &verifier_addr);

    let admin    = world.new_account();
    let provider = world.new_account();

    did.initialize(&admin);
    oracle.initialize(&admin, &did_addr);
    verifier.initialize(&oracle_addr, &did_addr);

    AiTestContext { world, oracle, verifier, did, admin, provider }
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[test]
fn test_oracle_submit_and_retrieve_result() {
    let ctx = setup();
    let _ = &ctx.world;

    let model_id = soroban_sdk::Symbol::new(ctx.world.env(), "model_v1");
    let result   = soroban_sdk::String::from_str(ctx.world.env(), r#"{"diagnosis":"normal"}"#);

    // Provider submits an inference result.
    ctx.oracle.submit_result(&ctx.provider, &model_id, &result);

    let stored = ctx.oracle.get_result(&model_id);
    assert_eq!(stored, result, "Oracle should return the submitted result");
}

#[test]
fn test_knowledge_verifier_gates_on_did() {
    let ctx = setup();
    let _ = &ctx.world;

    let user    = ctx.world.new_account();
    let did_doc = soroban_sdk::String::from_str(ctx.world.env(), r#"{"id":"did:uzima:user"}"#);
    let query   = soroban_sdk::String::from_str(ctx.world.env(), "treatment_protocol");

    // Without a DID the verifier should reject.
    let rejected = ctx.verifier.try_query_knowledge(&user, &query);
    assert!(rejected.is_err(), "Knowledge access should be denied without a DID");

    // Register a DID and retry.
    ctx.did.register(&user, &did_doc);
    let granted = ctx.verifier.query_knowledge(&user, &query);
    assert!(!granted.is_empty(), "Knowledge access should be granted after DID registration");
}

#[test]
fn test_oracle_result_flows_into_verifier() {
    let ctx = setup();
    let _ = &ctx.world;

    let model_id  = soroban_sdk::Symbol::new(ctx.world.env(), "diag_model");
    let result    = soroban_sdk::String::from_str(ctx.world.env(), r#"{"confidence":0.98}"#);
    let user      = ctx.world.new_account();
    let did_doc   = soroban_sdk::String::from_str(ctx.world.env(), r#"{"id":"did:uzima:flow"}"#);

    ctx.oracle.submit_result(&ctx.provider, &model_id, &result);
    ctx.did.register(&user, &did_doc);

    // Verifier should expose the oracle result to a credentialed user.
    let knowledge = ctx.verifier.query_model_result(&user, &model_id);
    assert_eq!(knowledge, result, "Verifier should proxy the oracle result for a credentialed user");
}

#[test]
fn test_stale_oracle_result_is_rejected_after_expiry() {
    let ctx = setup();
    let _ = &ctx.world;

    let model_id = soroban_sdk::Symbol::new(ctx.world.env(), "old_model");
    let result   = soroban_sdk::String::from_str(ctx.world.env(), r#"{"data":"stale"}"#);
    let user     = ctx.world.new_account();
    let did_doc  = soroban_sdk::String::from_str(ctx.world.env(), r#"{"id":"did:uzima:stale"}"#);

    ctx.oracle.submit_result(&ctx.provider, &model_id, &result);
    ctx.did.register(&user, &did_doc);

    // Advance time past the oracle TTL.
    ctx.world.advance_time(86_400 * 7 + 1); // 7 days + 1 second

    let expired = ctx.verifier.try_query_model_result(&user, &model_id);
    assert!(expired.is_err(), "Verifier should reject stale oracle results");
}