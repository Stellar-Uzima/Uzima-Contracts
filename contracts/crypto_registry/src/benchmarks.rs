//! Contract performance benchmarks.
//!
//! Naming convention: `bench_<operation>` — CI runs `cargo test bench_`.
#![allow(clippy::unwrap_used)]
extern crate std;
use std::time::Instant;

use super::*;
use soroban_sdk::testutils::{budget::Budget, Address as _};
use soroban_sdk::{Address, Bytes, Env};

// ── thresholds (CPU instructions) ──────────────────────────────────────────
const BUDGET_INITIALIZE: u64 = 2_000_000;
const BUDGET_REGISTER_KEY: u64 = 5_000_000;
const BUDGET_GET_BUNDLE: u64 = 2_000_000;
const BUDGET_REVOKE_KEY: u64 = 3_000_000;
const BUDGET_GET_VERSION: u64 = 1_500_000;

// ── helpers ─────────────────────────────────────────────────────────────────

struct BenchResult {
    name: &'static str,
    cpu_instructions: u64,
    memory_bytes: u64,
    wall_us: u128,
}

impl BenchResult {
    fn print(&self) {
        std::println!(
            "[BENCH] {:40} cpu={:>12} insns  mem={:>10} bytes  wall={:>8}µs",
            self.name, self.cpu_instructions, self.memory_bytes, self.wall_us
        );
    }

    fn assert_cpu_under(&self, limit: u64) {
        assert!(
            self.cpu_instructions <= limit,
            "[BENCH] {} exceeded CPU budget: {} > {} instructions",
            self.name,
            self.cpu_instructions,
            limit,
        );
    }
}

fn measure<F: FnOnce()>(env: &Env, name: &'static str, f: F) -> BenchResult {
    env.budget().reset_unlimited();
    let t0 = Instant::now();
    f();
    let wall_us = t0.elapsed().as_micros();
    BenchResult {
        name,
        cpu_instructions: env.budget().cpu_instruction_cost(),
        memory_bytes: env.budget().memory_bytes_cost(),
        wall_us,
    }
}

fn x25519_key(env: &Env, byte: u8) -> PublicKey {
    PublicKey {
        algorithm: KeyAlgorithm::X25519,
        key: Bytes::from_slice(env, &[byte; 32]),
    }
}

fn empty_key(env: &Env) -> PublicKey {
    PublicKey {
        algorithm: KeyAlgorithm::Custom(0),
        key: Bytes::new(env),
    }
}

// ── benchmark tests ─────────────────────────────────────────────────────────

#[test]
fn bench_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, CryptoRegistry);
    let client = CryptoRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);

    let r = measure(&env, "crypto_registry::initialize", || {
        client.initialize(&admin);
    });
    r.print();
    r.assert_cpu_under(BUDGET_INITIALIZE);
}

#[test]
fn bench_register_key_bundle() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, CryptoRegistry);
    let client = CryptoRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let alice = Address::generate(&env);

    let r = measure(&env, "crypto_registry::register_key_bundle (write)", || {
        client.register_key_bundle(
            &alice,
            &x25519_key(&env, 1),
            &empty_key(&env),
            &false,
            &empty_key(&env),
            &false,
        );
    });
    r.print();
    r.assert_cpu_under(BUDGET_REGISTER_KEY);
}

#[test]
fn bench_get_current_key_bundle() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, CryptoRegistry);
    let client = CryptoRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let alice = Address::generate(&env);
    client.register_key_bundle(
        &alice,
        &x25519_key(&env, 1),
        &empty_key(&env),
        &false,
        &empty_key(&env),
        &false,
    );

    let r = measure(&env, "crypto_registry::get_current_key_bundle (read)", || {
        client.get_current_key_bundle(&alice);
    });
    r.print();
    r.assert_cpu_under(BUDGET_GET_BUNDLE);
}

#[test]
fn bench_get_current_version() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, CryptoRegistry);
    let client = CryptoRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let alice = Address::generate(&env);
    client.register_key_bundle(
        &alice,
        &x25519_key(&env, 1),
        &empty_key(&env),
        &false,
        &empty_key(&env),
        &false,
    );

    let r = measure(&env, "crypto_registry::get_current_version (read)", || {
        client.get_current_version(&alice);
    });
    r.print();
    r.assert_cpu_under(BUDGET_GET_VERSION);
}

#[test]
fn bench_revoke_key_bundle() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, CryptoRegistry);
    let client = CryptoRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let alice = Address::generate(&env);
    client.register_key_bundle(
        &alice,
        &x25519_key(&env, 1),
        &empty_key(&env),
        &false,
        &empty_key(&env),
        &false,
    );

    let r = measure(&env, "crypto_registry::revoke_key_bundle (write)", || {
        client.revoke_key_bundle(&alice, &1);
    });
    r.print();
    r.assert_cpu_under(BUDGET_REVOKE_KEY);
}

#[test]
fn bench_key_rotation_three_versions() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, CryptoRegistry);
    let client = CryptoRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let alice = Address::generate(&env);

    // Seed v1
    client.register_key_bundle(
        &alice,
        &x25519_key(&env, 1),
        &empty_key(&env),
        &false,
        &empty_key(&env),
        &false,
    );

    let r2 = measure(&env, "crypto_registry::key_rotation v1→v2", || {
        client.register_key_bundle(
            &alice,
            &x25519_key(&env, 2),
            &empty_key(&env),
            &false,
            &empty_key(&env),
            &false,
        );
    });
    r2.print();

    let r3 = measure(&env, "crypto_registry::key_rotation v2→v3", || {
        client.register_key_bundle(
            &alice,
            &x25519_key(&env, 3),
            &empty_key(&env),
            &false,
            &empty_key(&env),
            &false,
        );
    });
    r3.print();

    std::println!(
        "[BENCH] rotation CPU delta v2→v3 vs v1→v2: {:+} insns",
        r3.cpu_instructions as i64 - r2.cpu_instructions as i64
    );

    assert_eq!(client.get_current_version(&alice), 3);
    r2.assert_cpu_under(BUDGET_REGISTER_KEY);
    r3.assert_cpu_under(BUDGET_REGISTER_KEY);
}

#[test]
fn bench_storage_write_vs_read_ratio() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, CryptoRegistry);
    let client = CryptoRegistryClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let alice = Address::generate(&env);

    let write = measure(&env, "crypto_registry::register_key_bundle (write)", || {
        client.register_key_bundle(
            &alice,
            &x25519_key(&env, 1),
            &empty_key(&env),
            &false,
            &empty_key(&env),
            &false,
        );
    });

    let read = measure(&env, "crypto_registry::get_current_key_bundle (read)", || {
        client.get_current_key_bundle(&alice);
    });

    write.print();
    read.print();

    std::println!(
        "[BENCH] storage write/read CPU ratio: {:.2}x",
        write.cpu_instructions as f64 / read.cpu_instructions.max(1) as f64
    );

    assert!(
        write.cpu_instructions > read.cpu_instructions,
        "expected write to cost more CPU than read"
    );
}
