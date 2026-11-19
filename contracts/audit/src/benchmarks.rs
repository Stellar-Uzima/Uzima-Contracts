//! Append-heavy write-amplification benchmarks for audit (issue #1957).
//!
//! Picked up by `scripts/measure_storage.sh`, which runs `cargo test
//! bench_storage_` and parses `[STORAGE-BENCH]` lines containing `after=`.
//! `log_event` performs one log write plus growing index rewrites, so the
//! per-append CPU cost and its growth with the existing log count are the
//! write-amplification signal we gate on.
#![allow(clippy::unwrap_used)]
#![cfg(test)]

extern crate std;

use crate::types::{ActionType, AuditConfig, AuditType, OperationResult};
use crate::{AuditTrail, AuditTrailClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Map, String, Vec};

const MAX_APPEND_CPU: u64 = 5_000_000;
const MAX_WRITE_AMPLIFICATION: u64 = 12;

fn setup<'a>(env: &'a Env) -> (AuditTrailClient<'a>, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let id = env.register_contract(None, AuditTrail);
    let client = AuditTrailClient::new(env, &id);
    let mut enabled = Vec::new(env);
    enabled.push_back(AuditType::Event);
    enabled.push_back(AuditType::AdminAction);
    client.initialize(
        &admin,
        &AuditConfig {
            archive_threshold: 1000,
            enabled_types: enabled,
        },
    );
    (client, admin)
}

fn append(env: &Env, client: &AuditTrailClient<'_>, actor: &Address, n: u64) {
    for i in 0..n {
        let mut meta = Map::new(env);
        let seq = if i % 2 == 0 {
            String::from_str(env, "even")
        } else {
            String::from_str(env, "odd")
        };
        meta.set(String::from_str(env, "seq"), seq);
        client.log_event(
            actor,
            &ActionType::DataWrite,
            &BytesN::from_array(env, &[i as u8; 32]),
            &OperationResult::Success,
            &meta,
        );
    }
}

fn measure_cpu<F: FnOnce()>(env: &Env, f: F) -> u64 {
    env.budget().reset_unlimited();
    f();
    env.budget().cpu_instruction_cost()
}

fn bench_append_at(existing: u64) -> u64 {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let actor = Address::generate(&env);
    append(&env, &client, &actor, existing);

    let cost = measure_cpu(&env, || {
        append(&env, &client, &actor, 1);
    });

    std::println!(
        "[STORAGE-BENCH] audit::log_event existing={} after={} cpu_cost={}",
        existing,
        cost,
        cost
    );
    assert!(
        cost <= MAX_APPEND_CPU,
        "single append exceeded CPU budget: {} > {}",
        cost,
        MAX_APPEND_CPU
    );
    cost
}

#[test]
fn bench_storage_audit_append_1_existing() {
    bench_append_at(1);
}

#[test]
fn bench_storage_audit_append_10_existing() {
    bench_append_at(10);
}

#[test]
fn bench_storage_audit_append_50_existing() {
    bench_append_at(50);
}

#[test]
fn bench_storage_audit_write_amplification_ratio() {
    let c1 = bench_append_at(1);
    let c50 = bench_append_at(50);
    let ratio = c50 / c1.max(1);
    std::println!(
        "[STORAGE-BENCH] audit::write_amplification after={} ratio={}",
        c50,
        ratio
    );
    assert!(
        ratio <= MAX_WRITE_AMPLIFICATION,
        "write amplification {} > {}",
        ratio,
        MAX_WRITE_AMPLIFICATION
    );
}
