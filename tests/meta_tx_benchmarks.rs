//! # Meta-Transaction Forwarder Benchmarks
//!
//! Compares the CPU / memory cost of executing a target contract call
//! directly (signed auth, single hop) versus through the meta-tx
//! forwarder (Ed25519 signature verification + nonce + dispatch + target
//! call). The CI threshold is generous so the benchmarks document the
//! real overhead rather than fail on ledger-state variance.

#![cfg(test)]
#![allow(clippy::unwrap_used)]

use std::time::Instant;

use ed25519_dalek::{Signer, SigningKey};
use meta_tx_forwarder::{ForwardRequest, MetaTxForwarder, MetaTxForwarderClient};
use rand::rngs::OsRng;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{symbol_short, vec, Address, Bytes, BytesN, Env, IntoVal, Val, Vec};

use crate::meta_tx_integration_test::{vals_to_target_args, MockTargetContract, MockTargetContractClient};

// ---------------------------------------------------------------------------
// Sign-message helper (uses the contract's public DOMAIN_PREFIX so any
// future change to the prefix is picked up automatically by tests).
// ---------------------------------------------------------------------------

fn sign_message(
    env: &Env,
    trusted_forwarder: &Address,
    signing_key: &SigningKey,
    request: &ForwardRequest,
) -> BytesN<64> {
    let mut message = Bytes::new(env);
    message.append(&Bytes::from_slice(env, &meta_tx_forwarder::DOMAIN_PREFIX));
    message.append(&trusted_forwarder.to_xdr(env));
    message.append(&request.to_xdr(env));

    let mut out: std::vec::Vec<u8> = std::vec::Vec::with_capacity(message.len() as usize);
    let mut i: u32 = 0;
    while i < message.len() {
        out.push(message.get(i).unwrap_or(0));
        i = i.saturating_add(1);
    }
    let sig_bytes = signing_key.sign(&out).to_bytes();
    BytesN::<64>::from_array(env, &sig_bytes)
}
    env: &Env,
    trusted_forwarder: &Address,
    signing_key: &SigningKey,
    request: &ForwardRequest,
) -> BytesN<64> {
    let mut message = Bytes::new(env);
    message.append(&Bytes::from_slice(
        env,
        &[
            b'U', b'Z', b'M', b'-', b'M', b'T', b'X', b'-', b'v', b'1', 0, 0, 0, 0, 0, 0,
        ],
    ));
    message.append(&trusted_forwarder.to_xdr(env));
    message.append(&request.to_xdr(env));

    let mut out: std::vec::Vec<u8> = std::vec::Vec::with_capacity(message.len() as usize);
    let mut i: u32 = 0;
    while i < message.len() {
        out.push(message.get(i).unwrap_or(0));
        i = i.saturating_add(1);
    }
    let sig_bytes = signing_key.sign(&out).to_bytes();
    BytesN::<64>::from_array(env, &sig_bytes)
}

struct BenchResult {
    name: &'static str,
    cpu_instructions: u64,
    memory_bytes: u64,
    wall_us: u128,
}

impl BenchResult {
    fn print(&self) {
        std::println!(
            "[BENCH] {:>40} cpu={:>12} insns  mem={:>10} bytes  wall={:>8}µs",
            self.name, self.cpu_instructions, self.memory_bytes, self.wall_us
        );
    }
}

fn measure<F: FnOnce()>(env: &Env, name: &'static str, f: F) -> BenchResult {
    env.budget().reset_unlimited();
    let t0 = Instant::now();
    f();
    BenchResult {
        name,
        cpu_instructions: env.budget().cpu_instruction_cost(),
        memory_bytes: env.budget().memory_bytes_cost(),
        wall_us: t0.elapsed().as_micros(),
    }
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

#[test]
fn bench_direct_target_call() {
    let env = Env::default();
    env.mock_all_auths();
    let tid = install(&env, MockTargetContract);
    let client = MockTargetContractClient::new(&env, &tid);
    let user = Address::generate(&env);
    env.mock_all_auths();

    let r = measure(&env, "direct::record_and_add", || {
        client.record_and_add(&user, &10, &20);
    });
    r.print();
}

#[test]
fn bench_meta_tx_relayed_call() {
    let env = Env::default();
    env.mock_all_auths();

    let fid = env.register_contract(None, MetaTxForwarder);
    let forwarder = MetaTxForwarderClient::new(&env, &fid);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    forwarder.initialize(&owner, &fc, &1_000_000i128);
    let relayer = Address::generate(&env);
    forwarder.register_relayer(&owner, &relayer, &100u32);

    let tid = env.register_contract(None, MockTargetContract);
    let _target = MockTargetContractClient::new(&env, &tid);

    // Ed25519 keypair for the user.
    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let pk_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
    let pk = BytesN::<32>::from_array(&env, &pk_bytes);
    let user_addr = Address::generate(&env);
    forwarder.register_user_pub_key(&user_addr, &pk);

    let args: Vec<Val> = vec![&env, 10u32.into_val(&env), 20u32.into_val(&env)];

    let r = measure(&env, "meta_tx::execute (verify + dispatch)", || {
        let req = ForwardRequest {
            from: user_addr.clone(),
            to: tid.clone(),
            value: 0,
            gas: 100_000,
            nonce: forwarder.get_nonce(&user_addr),
            deadline: env.ledger().timestamp() + 3600,
            data: Bytes::new(&env),
            target_fn: symbol_short!("record_a"),
            target_args: vals_to_target_args(&env, &[31u32.into_val(&env), 28u32.into_val(&env)]),
        };
        let sig = sign_message(&env, &fid, &signing_key, &req);
        forwarder.execute(&relayer, &req, &sig);
    });
    r.print();
    assert_eq!(forwarder.get_nonce(&user_addr), 1);
}

#[test]
fn bench_relayed_overhead_factor() {
    // Headline benchmark: prints a single [BENCH] line with the ratio of
    // relayed cost vs direct cost so it surfaces in CI logs and any review.
    let env = Env::default();
    env.mock_all_auths();

    let tid = env.register_contract(None, MockTargetContract);
    let target = MockTargetContractClient::new(&env, &tid);
    let direct_user = Address::generate(&env);

    let direct = measure(&env, "direct_target_call", || {
        target.record_and_add(&direct_user, &1, &1);
    });

    let fid = env.register_contract(None, MetaTxForwarder);
    let forwarder = MetaTxForwarderClient::new(&env, &fid);
    let owner = Address::generate(&env);
    let fc = Address::generate(&env);
    forwarder.initialize(&owner, &fc, &1_000_000i128);
    let relayer = Address::generate(&env);
    forwarder.register_relayer(&owner, &relayer, &100u32);

    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let pk_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
    let pk = BytesN::<32>::from_array(&env, &pk_bytes);
    let relayed_user = Address::generate(&env);
    forwarder.register_user_pub_key(&relayed_user, &pk);

    let indirect = measure(&env, "relayed_call_via_forwarder", || {
        let req = ForwardRequest {
            from: relayed_user.clone(),
            to: tid.clone(),
            value: 0,
            gas: 100_000,
            nonce: forwarder.get_nonce(&relayed_user),
            deadline: env.ledger().timestamp() + 3600,
            data: Bytes::new(&env),
            target_fn: symbol_short!("record_a"),
            target_args: vals_to_target_args(&env, &[1u32.into_val(&env), 1u32.into_val(&env)]),
        };
        let sig = sign_message(&env, &fid, &signing_key, &req);
        forwarder.execute(&relayer, &req, &sig);
    });

    direct.print();
    indirect.print();
    std::println!(
        "[BENCH] relayed-vs-direct CPU factor: {:.2}x (extra cost of meta-tx)",
        indirect.cpu_instructions as f64 / direct.cpu_instructions.max(1) as f64
    );
}
