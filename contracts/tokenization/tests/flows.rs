//! End-to-end flows against the deployed `Tokenization` contract interface.
//!
//! These live outside the crate (`tests/` rather than `src/`) so they exercise
//! only what `lib.rs` re-exports. If a flow here stops compiling, the public
//! surface of the contract has regressed even though the unit tests still pass.

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use tokenization::{
    TokenizationContract, TokenizationContractClient, DEFAULT_ADMIN_ROLE, ISSUER_ROLE,
};

fn sid(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

fn deploy(env: &Env) -> TokenizationContractClient<'_> {
    let contract_id = env.register_contract(None, TokenizationContract);
    TokenizationContractClient::new(env, &contract_id)
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

/// Full issuance lifecycle: register, mint to three holders, move units around,
/// burn, and reconcile the accounting at the end.
#[test]
fn issuance_lifecycle_reconciles() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let treasury = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &treasury);
    client.grant_role(&issuer, &ISSUER_ROLE);

    let id = sid(&env, "HOSP-2026-001");
    client.register_asset(&id, &issuer, &1_000_000, &6, &sid(&env, "ipfs://hospital"));

    client.mint(&id, &alice, &400_000);
    client.mint(&id, &bob, &350_000);
    client.mint(&id, &carol, &250_000);

    client.transfer(&id, &alice, &bob, &100_000);
    client.transfer(&id, &carol, &alice, &25_000);
    client.burn(&id, &bob, &50_000);

    let alice_final = client.balance_of(&id, &alice);
    let bob_final = client.balance_of(&id, &bob);
    let carol_final = client.balance_of(&id, &carol);
    let supply = client.total_supply(&id);

    assert_eq!(alice_final, 325_000);
    assert_eq!(bob_final, 400_000);
    assert_eq!(carol_final, 225_000);
    assert_eq!(alice_final + bob_final + carol_final, supply);
    assert_eq!(client.asset_count(), 1);
    assert_eq!(client.get_asset(&id).issuer, issuer);
}

/// Secondary transfer of administration, followed by normal operation under the
/// new admin.
#[test]
fn admin_rotation_hands_over_cleanly() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let successor = Address::generate(&env);
    let issuer = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &admin);
    client.grant_role(&issuer, &ISSUER_ROLE);

    client.transfer_admin(&successor);

    assert_eq!(client.admin(), successor);
    assert!(client.has_role(&successor, &DEFAULT_ADMIN_ROLE));
    assert!(!client.has_role(&admin, &DEFAULT_ADMIN_ROLE));

    // The new admin can still register assets.
    let id = sid(&env, "HOSP-2");
    client.register_asset(&id, &issuer, &10, &2, &sid(&env, "ipfs://clinic"));
    assert_eq!(client.asset_count(), 1);
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

/// Every rejection the happy path can hit, asserted end to end.
#[test]
fn rejected_operations_leave_ledger_untouched() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let stranger = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &admin);

    let id = sid(&env, "HOSP-1");
    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "ipfs://x"));

    // The issuer holds no role yet.
    assert!(client.try_mint(&id, &stranger, &1).is_err());
    assert_eq!(client.total_supply(&id), 0);

    client.grant_role(&issuer, &ISSUER_ROLE);
    client.mint(&id, &stranger, &100);

    let supply_before = client.total_supply(&id);
    let holder_before = client.balance_of(&id, &stranger);

    // Over-transfer, over-burn and duplicate registration.
    assert!(client.try_transfer(&id, &stranger, &admin, &101).is_err());
    assert!(client.try_burn(&id, &stranger, &101).is_err());
    assert!(client
        .try_register_asset(&id, &issuer, &1_000, &6, &sid(&env, "ipfs://x"))
        .is_err());

    assert_eq!(client.total_supply(&id), supply_before);
    assert_eq!(client.balance_of(&id, &stranger), holder_before);
}

/// An issuer whose role has been withdrawn can no longer create supply, and the
/// already-issued supply is untouched.
#[test]
fn issuer_role_withdrawal_is_enforced_end_to_end() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let holder = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &admin);
    client.grant_role(&issuer, &ISSUER_ROLE);

    let id = sid(&env, "HOSP-1");
    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "ipfs://x"));
    client.mint(&id, &holder, &500);

    client.revoke_role(&issuer, &ISSUER_ROLE);

    assert!(client.try_mint(&id, &holder, &1).is_err());
    assert!(client.try_burn(&id, &holder, &1).is_err());
    assert_eq!(client.balance_of(&id, &holder), 500);
    assert_eq!(client.total_supply(&id), 500);
}

/// Pausing halts every state change for every role until it is lifted.
#[test]
fn pause_halts_all_flows_until_lifted() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let alice = Address::generate(&env);

    let client = deploy(&env);
    client.initialize(&admin, &admin);
    client.grant_role(&issuer, &ISSUER_ROLE);

    let id = sid(&env, "HOSP-1");
    client.register_asset(&id, &issuer, &1_000, &6, &sid(&env, "ipfs://x"));
    client.mint(&id, &alice, &200);

    client.pause();
    assert!(client.try_mint(&id, &alice, &1).is_err());
    assert!(client.try_burn(&id, &alice, &1).is_err());
    assert!(client.try_transfer(&id, &alice, &admin, &1).is_err());
    assert_eq!(client.balance_of(&id, &alice), 200);

    client.unpause();
    client.mint(&id, &alice, &1);
    assert_eq!(client.balance_of(&id, &alice), 201);
}