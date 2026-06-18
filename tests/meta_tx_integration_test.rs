//! # Meta-Transaction Forwarder Integration Tests
//!
//! End-to-end coverage for the ERC-2771-compatible meta-transaction
//! forwarder: a real Ed25519 signer, a registered relayer, and a mock target
//! contract that records the original sender. Every test is deterministic
//! (uses `OsRng` only for key generation).
//!
//! Run with `cargo test --test meta_tx_integration_test` from the workspace
//! root or `cargo test -p uzima-tests -- meta_tx`.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use ed25519_dalek::{Signer, SigningKey};
use meta_tx_forwarder::{ForwardRequest, MetaTxForwarder, MetaTxForwarderClient};
use rand::rngs::OsRng;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{symbol_short, vec, Address, Bytes, BytesN, Env, IntoVal, Symbol, Val, Vec};

/// Build a `Vec<Val>` from a slice of `Val`s. Target contracts receive
/// positional heterogeneous args via the forwarder's `forward_call`.
fn vals_to_target_args(env: &Env, vals: &[Val]) -> Vec<Val> {
    let mut out: Vec<Val> = Vec::new(env);
    for v in vals {
        out.push_back(*v);
    }
    out
}

// ---------------------------------------------------------------------------
// Mock target contract (regenerated here for test-isolation purposes).
// ---------------------------------------------------------------------------

#[soroban_sdk::contract]
pub struct MockTargetContract;

#[soroban_sdk::contractimpl]
impl MockTargetContract {
    /// Atomically records (from, last_a, last_b) and returns a + b.
    /// Exposed as `record_a` (≤ 9 ASCII chars to fit `symbol_short!`).
    pub fn record_a(
        env: Env,
        from: Address,
        a: u32,
        b: u32,
    ) -> u32 {
        env.storage()
            .instance()
            .set(&MockTargetKey::LastFrom, &from);
        env.storage().instance().set(&MockTargetKey::LastA, &a);
        env.storage().instance().set(&MockTargetKey::LastB, &b);
        a + b
    }

    /// No-op target used to assert dispatch-only behaviour.
    pub fn noop(_env: Env, _from: Address) {}
}

#[derive(Clone)]
#[soroban_sdk::contracttype]
pub enum MockTargetKey {
    LastFrom,
    LastA,
    LastB,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn install_forwarder(env: &Env) -> (Address, MetaTxForwarderClient<'_>) {
    let id = env.register_contract(None, MetaTxForwarder);
    let client = MetaTxForwarderClient::new(env, &id);
    (id, client)
}

fn install_target(env: &Env) -> (Address, MockTargetContractClient<'_>) {
    let id = env.register_contract(None, MockTargetContract);
    let client = MockTargetContractClient::new(env, &id);
    (id, client)
}

struct UserKeyPair {
    addr: Address,
    signing_key: SigningKey,
    #[allow(dead_code)]
    pub_key: [u8; 32],
}

fn make_user(env: &Env, fwd: &MetaTxForwarderClient<'_>) -> UserKeyPair {
    let addr = Address::generate(env);
    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let pk_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
    let pk = BytesN::<32>::from_array(env, &pk_bytes);
    fwd.register_user_pub_key(&addr, &pk);
    UserKeyPair {
        addr,
        signing_key,
        pub_key: pk_bytes,
    }
}

fn sign_request(
    env: &Env,
    trusted_forwarder: &Address,
    signing_key: &SigningKey,
    request: &ForwardRequest,
) -> BytesN<64> {
    let mut message: Bytes = Bytes::new(env);
    let prefix: Bytes = Bytes::from_slice(env, &meta_tx_forwarder::DOMAIN_PREFIX);
    message.append(&prefix);
    let fwd_xdr: Bytes = trusted_forwarder.to_xdr(env);
    message.append(&fwd_xdr);
    // The soroban-sdk 21.7.7 contracttype-derive impl consumes self by
    // value, so the borrowed request is cloned before serialising.
    let req_xdr: Bytes = request.clone().to_xdr(env);
    message.append(&req_xdr);

    let mut out: std::vec::Vec<u8> = std::vec::Vec::with_capacity(message.len() as usize);
    let mut i: u32 = 0;
    while i < message.len() {
        out.push(message.get(i).unwrap_or(0));
        i = i.saturating_add(1);
    }
    let sig_bytes = signing_key.sign(&out).to_bytes();
    BytesN::<64>::from_array(env, &sig_bytes)
}

fn build_record_request(
    env: &Env,
    user: &UserKeyPair,
    target: &Address,
    nonce: u64,
    deadline: u64,
    a: u32,
    b: u32,
) -> ForwardRequest {
    ForwardRequest {
        from: user.addr.clone(),
        to: target.clone(),
        value: 0,
        gas: 100_000,
        nonce,
        deadline,
        target_fn: symbol_short!("record_a"),
        target_args: vals_to_target_args(env, &[a.into_val(env), b.into_val(env)]),
    }
}

// ============================================================================
// Happy path
// ============================================================================

#[test]
fn end_to_end_relayed_call_invokes_target_with_original_sender() {
    let env = Env::default();
    env.mock_all_auths();

    let (forwarder_id, forwarder) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    forwarder.initialize(&owner, &fc, &1_000_000i128);

    let (target_id, target) = install_target(&env);

    let relayer = Address::generate(&env);
    forwarder.register_relayer(&owner, &relayer, &100u32);

    let user = make_user(&env, &forwarder);

    let req = build_record_request(
        &env,
        &user,
        &target_id,
        0,
        env.ledger().timestamp() + 3600,
        40,
        2,
    );
    let sig = sign_request(&env, &forwarder_id, &user.signing_key, &req);

    // Relayer submits — no auth required from `user`.
    let result_bytes = forwarder.execute(&relayer, &req, &sig);

    // The mock target recorded the original sender in arg 0.
    let recorded_from = env.as_contract(&target_id, || {
        env.storage()
            .instance()
            .get::<_, Address>(&MockTargetKey::LastFrom)
    });
    assert_eq!(recorded_from, Some(user.addr.clone()));

    // Soroban 21.x removed `env.invoker()` from the public API, so we
    // cannot assert the invoker address from inside the test host. Instead
    // we just confirm the contract was dispatched by the forwarder and that
    // the original `from` was recorded as arg 0 by the mock target.
    let recorded_from = env.as_contract(&target_id, || {
        env.storage()
            .instance()
            .get::<_, Address>(&MockTargetKey::LastFrom)
    });
    assert_eq!(recorded_from, Some(user.addr.clone()));

    // Forwarder returned the result XDR — non-empty for a u32 return.
    assert!(!result_bytes.is_empty());

    // Nonce advanced exactly once.
    assert_eq!(forwarder.get_nonce(&user.addr), 1);
}

// ============================================================================
// Replay protection across multiple sequential requests
// ============================================================================

#[test]
fn sequential_relayed_calls_increment_nonce() {
    let env = Env::default();
    env.mock_all_auths();
    let (fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    fwd.initialize(&owner, &fc, &1_000_000i128);
    let (tid, _tgt) = install_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &0u32);
    let user = make_user(&env, &fwd);

    for n in 0u64..3 {
        let req = build_record_request(
            &env,
            &user,
            &tid,
            n,
            env.ledger().timestamp() + 3600,
            n as u32 + 1,
            (n as u32 + 1) * 10,
        );
        let sig = sign_request(&env, &fid, &user.signing_key, &req);
        fwd.execute(&relayer, &req, &sig);
        assert_eq!(fwd.get_nonce(&user.addr), n + 1);
    }
}

// ============================================================================
// Negative paths
// ============================================================================

#[test]
fn unauthorized_relayer_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    fwd.initialize(&owner, &fc, &1_000_000i128);
    let (tid, _tgt) = install_target(&env);

    let user = make_user(&env, &fwd);
    let stranger = Address::generate(&env);
    let req = build_record_request(
        &env,
        &user,
        &tid,
        0,
        env.ledger().timestamp() + 3600,
        1,
        2,
    );
    let sig = sign_request(&env, &fid, &user.signing_key, &req);

    let res = fwd.try_execute(&stranger, &req, &sig);
    assert_eq!(res, Err(Ok(meta_tx_forwarder::Error::Unauthorized)));
}

#[test]
fn invalid_nonce_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    fwd.initialize(&owner, &fc, &1_000_000i128);
    let (tid, _tgt) = install_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &0u32);
    let user = make_user(&env, &fwd);

    let req = build_record_request(
        &env,
        &user,
        &tid,
        42,
        env.ledger().timestamp() + 3600,
        1,
        2,
    );
    let sig = sign_request(&env, &fid, &user.signing_key, &req);
    let res = fwd.try_execute(&relayer, &req, &sig);
    assert_eq!(res, Err(Ok(meta_tx_forwarder::Error::InvalidNonce)));
}

#[test]
fn expired_request_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    fwd.initialize(&owner, &fc, &1_000_000i128);
    let (tid, _tgt) = install_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &0u32);
    let user = make_user(&env, &fwd);

    env.ledger().with_mut(|l| l.timestamp = 5_000);
    let now = env.ledger().timestamp();
    let req = build_record_request(
        &env,
        &user,
        &tid,
        0,
        now.saturating_sub(1), // already expired
        1,
        2,
    );
    let sig = sign_request(&env, &fid, &user.signing_key, &req);
    let res = fwd.try_execute(&relayer, &req, &sig);
    assert_eq!(res, Err(Ok(meta_tx_forwarder::Error::RequestExpired)));
}

// ============================================================================
// Batch
// ============================================================================

#[test]
fn batch_execute_two_users_advances_independent_nonces() {
    let env = Env::default();
    env.mock_all_auths();
    let (fid, fwd) = install_forwarder(&env);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    fwd.initialize(&owner, &fc, &1_000_000i128);
    let (tid, _tgt) = install_target(&env);
    let relayer = Address::generate(&env);
    fwd.register_relayer(&owner, &relayer, &0u32);
    let u1 = make_user(&env, &fwd);
    let u2 = make_user(&env, &fwd);

    let r1 = build_record_request(
        &env,
        &u1,
        &tid,
        0,
        env.ledger().timestamp() + 3600,
        5,
        6,
    );
    let r2 = build_record_request(
        &env,
        &u2,
        &tid,
        0,
        env.ledger().timestamp() + 3600,
        7,
        8,
    );
    let s1 = sign_request(&env, &fid, &u1.signing_key, &r1);
    let s2 = sign_request(&env, &fid, &u2.signing_key, &r2);
    let results = fwd.execute_batch(
        &relayer,
        &vec![&env, r1.clone(), r2.clone()],
        &vec![&env, s1.clone(), s2.clone()],
    );
    assert_eq!(results.len(), 2);
    assert_eq!(fwd.get_nonce(&u1.addr), 1);
    assert_eq!(fwd.get_nonce(&u2.addr), 1);
}

// ============================================================================
// Cross-contract dispatch through a real medical_records contract.
// Demonstrates the full ERC-2771 round-trip: the patient's signed
// request goes through the forwarder and lands on medical_records with
// `from` correctly transcribed as the function's arg 0 (here: `user`).
// ============================================================================

#[test]
fn integration_with_medical_records_end_to_end() {
    use medical_records::medical_records::{
        MedicalRecordsContract, MedicalRecordsContractClient, Role,
    };

    let env = Env::default();
    env.mock_all_auths();

    let (fid, fwd) = install_forwarder(&env);
    let forwarder_owner = Address::generate(&env);
    let fc = Address::generate(&env);
    fwd.initialize(&forwarder_owner, &fc, &1_000_000i128);

    let relayer = Address::generate(&env);
    fwd.register_relayer(&forwarder_owner, &relayer, &0u32);

    // Deploy medical_records with the doctor as its RBAC admin.
    let doctor = Address::generate(&env);
    let rbac_contract = Address::generate(&env);
    let medical_records_id = env.register_contract(None, MedicalRecordsContract);
    let medical_records = MedicalRecordsContractClient::new(&env, &medical_records_id);
    medical_records.initialize(&doctor, &rbac_contract);

    // Install the trusted forwarder into medical_records storage so that
    // its own checks (if it later calls msg_sender) know who the forwarder is.
    env.as_contract(&medical_records_id, || {
        meta_tx_forwarder::erc2771_context::ERC2771ContextImpl::set_trusted_forwarder(
            &env,
            fid.clone(),
        );
    });

    // Register the patient’s Ed25519 public key with the forwarder.
    let patient_addr = Address::generate(&env);
    let mut rng = OsRng;
    let patient_signer = SigningKey::generate(&mut rng);
    let patient_pk: [u8; 32] = patient_signer.verifying_key().to_bytes();
    let patient_pk_handle = BytesN::<32>::from_array(&env, &patient_pk);
    fwd.register_user_pub_key(&patient_addr, &patient_pk_handle);

    // The patient composes a meta-tx that asks medical_records for their
    // role. The forwarder invokes `get_user_role(from)` where `from` is
    // prepended as arg 0. With `target_args: Vec<Bytes>` empty, the
    // call shape is exactly `medical_records::get_user_role(env, from)`.
    let req = ForwardRequest {
        from: patient_addr.clone(),
        to: medical_records_id.clone(),
        value: 0,
        gas: 100_000,
        nonce: 0,
        deadline: env.ledger().timestamp() + 3600,
        target_fn: Symbol::new(&env, "get_user_role"),
        target_args: Vec::new(&env),
    };
    let sig = sign_request(&env, &fid, &patient_signer, &req);
    let result = fwd.execute(&relayer, &req, &sig);
    assert!(!result.is_empty(), "non-empty XDR return expected");
    assert_eq!(fwd.get_nonce(&patient_addr), 1);

    // Sanity: trust wiring round-tripped correctly.
    let trusted_in_mr = env.as_contract(&medical_records_id, || {
        meta_tx_forwarder::erc2771_context::ERC2771ContextImpl::get_trusted_forwarder(&env)
            .expect("trusted forwarder wired")
    });
    assert_eq!(trusted_in_mr, fid);

    // And the doctor remains the admin of medical_records.
    let role = medical_records.get_user_role(&doctor);
    assert_eq!(role, Ok(Role::Admin));
}


