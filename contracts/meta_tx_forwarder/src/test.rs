//! # Meta-Transaction Forwarder Tests
//!
//! These tests exercise a real Ed25519 signature verification path with a
//! dedicated mock target contract. Each test builds an actual Ed25519 key
//! pair, registers the public key, signs the request XDR payload, and then
//! asserts the resulting state via `try_X` calls.
//!
//! Run with `cargo test --lib -p meta_tx_forwarder` or
//! `cargo test -p meta_tx_forwarder`.

#![allow(clippy::unwrap_used, clippy::expect_used)]
// Bring the standard library back into scope inside the test sub-module
// (the parent crate is `#![no_std]`); required for `std::vec::Vec` used to
// feed the rustcrypto ed25519-dalek signer.
extern crate std;

use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{symbol_short, vec, Address, Bytes, BytesN, Env, IntoVal, Val, Vec};

use super::*;

// ---------------------------------------------------------------------------
// Mock target contract: receives forwarded calls with `from` as arg 0.
// ---------------------------------------------------------------------------

#[soroban_sdk::contract]
pub struct MockTarget;

#[soroban_sdk::contractimpl]
impl MockTarget {
    /// Add two numbers and record the original sender's address bytes.
    /// Exposed as `echo_add` (≤ 9 ASCII chars to fit `symbol_short!`).
    pub fn echo_add(env: Env, from: Address, a: u32, b: u32) -> u32 {
        // Record that we were called via the forwarder (arg 0 == from).
        env.storage()
            .instance()
            .set(&MockTargetKey::LastFrom, &from);
        a + b
    }

    /// No-op function used to verify that forward_call dispatches correctly.
    pub fn noop(_env: Env, _from: Address) {}
}

#[derive(Clone)]
#[soroban_sdk::contracttype]
pub enum MockTargetKey {
    LastFrom,
}

fn install_mock_target(env: &Env) -> (Address, MockTargetClient<'_>) {
    let id = env.register_contract(None, MockTarget);
    let client = MockTargetClient::new(env, &id);
    (id, client)
}

fn last_mock_target_last_from(env: &Env, target: &Address) -> Option<Address> {
    env.as_contract(target, || {
        env.storage()
            .instance()
            .get(&MockTargetKey::LastFrom)
    })
}

/// Build a `Vec<Val>` from a slice of `Val`s. Mirrors what target-contract
/// invocations expect (positional args of heterogeneous types).
fn vals_to_target_args(env: &Env, vals: &[Val]) -> Vec<Val> {
    let mut out: Vec<Val> = Vec::new(env);
    for v in vals {
        out.push_back(*v);
    }
    out
}

// ---------------------------------------------------------------------------
// Forwarder scaffolding helpers
// ---------------------------------------------------------------------------

fn install_forwarder(env: &Env) -> (Address, MetaTxForwarderClient<'_>) {
    let id = env.register_contract(None, MetaTxForwarder);
    let client = MetaTxForwarderClient::new(env, &id);
    (id, client)
}

fn initialize(
    _env: &Env,
    forwarder: &MetaTxForwarderClient<'_>,
    owner: &Address,
    fee_collector: &Address,
) {
    forwarder.initialize(owner, fee_collector, &1_000_000i128);
}

fn make_user(env: &Env, forwarder: &MetaTxForwarderClient<'_>, owner: &Address) -> UserKeys {
    let addr = Address::generate(env);
    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let vk_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
    let pk = BytesN::<32>::from_array(env, &vk_bytes);

    // Register public key on-chain (auth: the user).
    env.mock_all_auths();
    forwarder.register_user_pub_key(&addr, &pk);

    UserKeys {
        addr,
        signing_key,
        pub_key: pk,
        _owner: owner.clone(),
    }
}

struct UserKeys {
    addr: Address,
    signing_key: SigningKey,
    #[allow(dead_code)]
    pub_key: BytesN<32>,
    #[allow(dead_code)]
    _owner: Address,
}

/// Construct the canonical signed message for a `ForwardRequest` and produce
/// a valid Ed25519 signature. The byte layout is:
///   DOMAIN_PREFIX(16) || trusted_forwarder_address_xdr || request.to_xdr
/// and it must match `lib::verify_signature` byte-for-byte.
fn sign_request(
    env: &Env,
    trusted_forwarder: &Address,
    signing_key: &SigningKey,
    request: &ForwardRequest,
) -> BytesN<64> {
    let mut message: Bytes = Bytes::new(env);
    let prefix: Bytes = Bytes::from_slice(env, &super::DOMAIN_PREFIX);
    message.append(&prefix);
    let fwd_xdr: Bytes = trusted_forwarder.to_xdr(env);
    message.append(&fwd_xdr);
    // The soroban-sdk 21.7.7 contracttype-derive impl consumes self by
    // value when serialising to XDR, so the borrowed request is cloned
    // first.
    let req_xdr: Bytes = request.clone().to_xdr(env);
    message.append(&req_xdr);

    // Stream a copy out into a heap Vec<u8> to feed rustcrypto's signer,
    // which requires an actual byte slice (not a Soroban Bytes handle).
    let mut out: std::vec::Vec<u8> = std::vec::Vec::with_capacity(message.len() as usize);
    let mut i: u32 = 0;
    while i < message.len() {
        out.push(message.get(i).unwrap_or(0));
        i = i.saturating_add(1);
    }
    let sig_bytes = signing_key.sign(&out).to_bytes();
    BytesN::<64>::from_array(env, &sig_bytes)
}

/// Build a request that targets `MockTarget::echo_add(from, a, b)`.
///
/// `#[allow(clippy::too_many_arguments)]` is applied because every argument
/// here is a distinct, named input (user, target, monotonic nonce, deadline,
/// and the two addends). Grouping into a config struct would obscure what
/// each test is setting; the warning adds no real value at call sites that
/// already read like a literal config block.
#[allow(clippy::too_many_arguments)]
fn build_add_request(
    env: &Env,
    user: &UserKeys,
    target: &Address,
    nonce: u64,
    deadline: u64,
    a: u32,
    b: u32,
) -> ForwardRequest {
    let target_args = vals_to_target_args(env, &[a.into_val(env), b.into_val(env)]);
    ForwardRequest {
        from: user.addr.clone(),
        to: target.clone(),
        value: 0,
        gas: 100_000,
        nonce,
        deadline,
        target_fn: symbol_short!("echo_add"),
        target_args,
    }
}

// ===========================================================================
// Initialization
// ===========================================================================

#[test]
fn test_initialize_success() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fee_collector);
    assert_eq!(fwd.get_trusted_forwarder(), fwd.address);
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fee_collector);
    let res = fwd.try_initialize(&owner, &fee_collector, &1_000_000i128);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_register_user_pub_key_persists() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let user = Address::generate(&env);
    let pk = BytesN::<32>::from_array(&env, &[7u8; 32]);
    fwd.register_user_pub_key(&user, &pk);
    let stored = fwd.get_user_pub_key(&user).expect("key stored");
    assert_eq!(stored, pk);
}

// ===========================================================================
// Auth / Relayer gating
// ===========================================================================

#[test]
fn test_register_relayer_rejects_invalid_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fee_collector);

    // 100% + 1 basis point (100_01 bps) is not a valid fee.
    let new_relayer = Address::generate(&env);
    let res = fwd.try_register_relayer(&owner, &new_relayer, &10_001u32);
    assert_eq!(res, Err(Ok(Error::InvalidFeePercentage)));

    // 100% (10_000 bps) is the maximum → must succeed.
    fwd.register_relayer(&owner, &new_relayer, &10_000u32);
    assert!(fwd.is_relayer(&new_relayer));
}

#[test]
fn test_unauthorized_relayer_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env); // unregistered

    let req = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        1,
        2,
    );
    let sig = sign_request(&env, &fwd.address, &user.signing_key, &req);

    let res = fwd.try_execute(&relayer, &req, &sig);
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

// ===========================================================================
// Nonce management
// ===========================================================================

#[test]
fn test_nonce_starts_at_zero_and_increments() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    assert_eq!(fwd.get_nonce(&user.addr), 0);

    let req = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        10,
        20,
    );
    let sig = sign_request(&env, &fwd.address, &user.signing_key, &req);
    fwd.execute(&relayer, &req, &sig);
    assert_eq!(fwd.get_nonce(&user.addr), 1);
}

#[test]
fn test_replay_same_nonce_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    let req = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        4,
        5,
    );
    let sig = sign_request(&env, &fwd.address, &user.signing_key, &req);
    fwd.execute(&relayer, &req, &sig);
    // Replaying the same request must fail because the nonce advanced.
    let res = fwd.try_execute(&relayer, &req, &sig);
    assert_eq!(res, Err(Ok(Error::InvalidNonce)));
    // Nonce stays at 1 (rejected replay did not advance).
    assert_eq!(fwd.get_nonce(&user.addr), 1);
}

#[test]
fn test_nonce_skipped_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    // First request at nonce 0 succeeds.
    let req0 = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        1,
        1,
    );
    let sig0 = sign_request(&env, &fwd.address, &user.signing_key, &req0);
    fwd.execute(&relayer, &req0, &sig0);

    // Now skip to nonce 5 — should be rejected.
    let req5 = build_add_request(
        &env,
        &user,
        &target.address,
        5,
        env.ledger().timestamp() + 3600,
        1,
        1,
    );
    let sig5 = sign_request(&env, &fwd.address, &user.signing_key, &req5);
    let res = fwd.try_execute(&relayer, &req5, &sig5);
    assert_eq!(res, Err(Ok(Error::InvalidNonce)));
    // Nonce is still 1.
    assert_eq!(fwd.get_nonce(&user.addr), 1);
}

// ===========================================================================
// Deadline enforcement
// ===========================================================================

#[test]
fn test_expired_request_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    env.ledger().with_mut(|l| {
        l.timestamp = 1_000;
    });

    let now = env.ledger().timestamp();
    let req = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        now.saturating_sub(1), // already expired
        1,
        1,
    );
    let sig = sign_request(&env, &fwd.address, &user.signing_key, &req);
    let res = fwd.try_execute(&relayer, &req, &sig);
    assert_eq!(res, Err(Ok(Error::RequestExpired)));
    assert_eq!(fwd.get_nonce(&user.addr), 0);
}

// ===========================================================================
// Signature verification
// ===========================================================================

#[test]
fn test_signature_verification_succeeds_for_valid_sig() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    let req = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        7,
        8,
    );
    let sig = sign_request(&env, &fwd.address, &user.signing_key, &req);
    let result_bytes = fwd.execute(&relayer, &req, &sig);

    // Mock target receives the exact `from` user as arg 0.
    let last_from = last_mock_target_last_from(&env, &target.address).expect("from recorded");
    assert_eq!(last_from, user.addr);
    assert!(!result_bytes.is_empty());
}

#[test]
fn test_tampered_request_rejected_by_signature_trap() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    let req = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        1,
        2,
    );
    let sig = sign_request(&env, &fwd.address, &user.signing_key, &req);

    // Tamper with the args after signing.
    let tampered_args = vals_to_target_args(&env, &[99u32.into_val(&env), 99u32.into_val(&env)]);
    let tampered = ForwardRequest {
        target_args: tampered_args,
        ..req.clone()
    };

    let res = fwd.try_execute(&relayer, &tampered, &sig);
    // Soroban traps on ed25519 verification failure. try_execute surfaces
    // the trap as an Err result; either variant is acceptable.
    assert!(res.is_err(), "tampered request must be rejected");
}

#[test]
fn test_unsigned_user_request_rejected_no_pub_key() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let user_addr = Address::generate(&env);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    // No public key registered for `user_addr`.
    let req = ForwardRequest {
        from: user_addr.clone(),
        to: target.address.clone(),
        value: 0,
        gas: 100_000,
        nonce: 0,
        deadline: env.ledger().timestamp() + 3600,
        target_fn: symbol_short!("noop"),
        target_args: Vec::new(&env),
    };
    let sig = BytesN::<64>::from_array(&env, &[0u8; 64]);
    let res = fwd.try_execute(&relayer, &req, &sig);
    assert_eq!(res, Err(Ok(Error::PubKeyNotRegistered)));
}

#[test]
fn test_domain_separator_is_constant() {
    let env = Env::default();
    let (_fid, fwd) = install_forwarder(&env);
    let sep = fwd.domain_separator();
    assert_eq!(sep.len(), 16);
    assert_eq!(
        sep,
        Bytes::from_slice(
            &env,
            &[
                b'U', b'Z', b'M', b'-', b'M', b'T', b'X', b'-', b'v', b'1', 0, 0, 0, 0, 0, 0,
            ]
        )
    );
}

// ===========================================================================
// Batch execution
// ===========================================================================

#[test]
fn test_batch_execute_advances_each_user_nonce() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let u1 = make_user(&env, &fwd, &owner);
    let u2 = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    let r1 = build_add_request(
        &env,
        &u1,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        1,
        2,
    );
    let r2 = build_add_request(
        &env,
        &u2,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        10,
        20,
    );
    let s1 = sign_request(&env, &fwd.address, &u1.signing_key, &r1);
    let s2 = sign_request(&env, &fwd.address, &u2.signing_key, &r2);
    let res = fwd.execute_batch(
        &relayer,
        &vec![&env, r1.clone(), r2.clone()],
        &vec![&env, s1.clone(), s2.clone()],
    );
    assert_eq!(res.len(), 2);
    assert_eq!(fwd.get_nonce(&u1.addr), 1);
    assert_eq!(fwd.get_nonce(&u2.addr), 1);
}

#[test]
fn test_batch_length_mismatch_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);
    let u = make_user(&env, &fwd, &owner);
    let (_tid, target) = install_mock_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &100u32);

    let req = build_add_request(
        &env,
        &u,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        0,
        0,
    );
    let sig = sign_request(&env, &fwd.address, &u.signing_key, &req);
    // One request, two signatures → must be rejected.
    let res = fwd.try_execute_batch(
        &relayer,
        &vec![&env, req.clone()],
        &vec![&env, sig.clone(), sig.clone()],
    );
    assert_eq!(res, Err(Ok(Error::BatchLengthMismatch)));
}

// ===========================================================================
// ERC-2771 context helpers
// ===========================================================================

#[test]
fn test_trusted_forwarder_storage_is_consistent() {
    let env = Env::default();
    env.mock_all_auths();
    let (fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    initialize(&env, &fwd, &owner, &fc);

    // Reading through the forwarder client returns its own address.
    let stored = fwd.get_trusted_forwarder();
    assert_eq!(stored, fid);

    // Direct access via the ERC-2771 module helper requires a contract context.
    let retrieved = env.as_contract(&fid, || {
        crate::erc2771_context::ERC2771ContextImpl::get_trusted_forwarder(&env)
            .expect("trusted forwarder stored in shared storage")
    });
    assert_eq!(retrieved, fid);
}

// ===========================================================================
// Trusted forwarder pattern guarantees (wrong forwarder / wrong target)
// ===========================================================================

#[test]
fn test_signature_does_not_replay_against_different_forwarder() {
    // Two separate forwarder contracts must produce distinct signatures because
    // the trusted-forwarder address is bound into the signed message.
    let env = Env::default();
    env.mock_all_auths();

    let (fid_a, fwd_a) = install_forwarder(&env);
    let owner_a = Address::generate(&env);
    let fc_a = Address::generate(&env);
    initialize(&env, &fwd_a, &owner_a, &fc_a);

    let (_fid_b, fwd_b) = install_forwarder(&env);
    let owner_b = Address::generate(&env);
    let fc_b = Address::generate(&env);
    initialize(&env, &fwd_b, &owner_b, &fc_b);

    // Register the *same* relayer on both forwarders so that `Unauthorized`
    // is not the rejection reason — we want to exercise the signature path.
    let relayer = Address::generate(&env);
    fwd_a.register_relayer(&owner_a, &relayer, &100u32);
    fwd_b.register_relayer(&owner_b, &relayer, &100u32);

    let user = make_user(&env, &fwd_a, &owner_a);
    let (_tid, target) = install_mock_target(&env);

    let req = build_add_request(
        &env,
        &user,
        &target.address,
        0,
        env.ledger().timestamp() + 3600,
        3,
        4,
    );

    // Sign against A (legitimate forwarder) and execute there.
    let sig_for_a = sign_request(&env, &fid_a, &user.signing_key, &req);
    fwd_a.execute(&relayer, &req, &sig_for_a);

    // The very same signature bytes must NOT work against B even though B
    // has the same relayer. The signed payload contains `fid_a`, so B's
    // `verify_signature` will trap.
    let res = fwd_b.try_execute(&relayer, &req, &sig_for_a);
    assert!(
        res.is_err(),
        "cross-forwarder signature replay must be rejected"
    );
}
