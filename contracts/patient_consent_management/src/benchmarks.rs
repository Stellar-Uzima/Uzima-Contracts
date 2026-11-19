//! Storage write-amplification benchmarks for patient_consent_management.
//! Mirrors the pattern in contracts/medical_records/src/benchmarks.rs and is
//! picked up by scripts/measure_storage.sh (looks for `bench_storage_` tests
//! and parses `[STORAGE-BENCH]` output lines).
#![allow(clippy::unwrap_used)]
extern crate std;

use crate::{PatientConsentManagement, PatientConsentManagementClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

fn measure_cpu<F: FnOnce()>(env: &Env, f: F) -> u64 {
    env.budget().reset_unlimited();
    f();
    env.budget().cpu_instruction_cost()
}

fn setup() -> (Env, PatientConsentManagementClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000_000;
    });
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, PatientConsentManagement);
    let client = PatientConsentManagementClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, client, admin)
}

fn populate_consents(
    env: &Env,
    client: &PatientConsentManagementClient<'_>,
    patient: &Address,
    count: u64,
) {
    for _ in 0..count {
        let provider = Address::generate(env);
        client.grant_consent(patient, &provider);
    }
}

fn bench_storage_write_amplification_with_count(existing: u64) {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);

    populate_consents(&env, &client, &patient, existing);

    let new_provider = Address::generate(&env);
    let cost = measure_cpu(&env, || {
        client.grant_consent(&patient, &new_provider);
    });

    std::println!(
        "[STORAGE-BENCH] grant_consent existing={} cpu_cost={}",
        existing,
        cost
    );
}

#[test]
fn bench_storage_grant_consent_1_existing() {
    bench_storage_write_amplification_with_count(1);
}

#[test]
fn bench_storage_grant_consent_10_existing() {
    bench_storage_write_amplification_with_count(10);
}

#[test]
fn bench_storage_grant_consent_50_existing() {
    bench_storage_write_amplification_with_count(50);
}

// ============================================================================
// Instruction-count regression budgets for consent flows (issue #1956)
// ============================================================================

/// CPU-instruction ceilings for the hot consent paths. Mirrors the
/// `BUDGET_*` constants in `contracts/crypto_registry/src/benchmarks.rs` and
/// the `soroban_baselines` module in `tests/utils/performance.rs`.
const BUDGET_GRANT: u64 = 3_000_000;
const BUDGET_REVOKE: u64 = 3_000_000;
const BUDGET_ACCESS_CHECK: u64 = 2_000_000;

struct BenchResult {
    name: &'static str,
    cpu_instructions: u64,
}

impl BenchResult {
    fn assert_cpu_under(&self, limit: u64) {
        assert!(
            self.cpu_instructions <= limit,
            "[BENCH] {} exceeded CPU budget: {} > {} instructions",
            self.name,
            self.cpu_instructions,
            limit
        );
    }
}

fn measure_named<F: FnOnce()>(env: &Env, name: &'static str, f: F) -> BenchResult {
    env.budget().reset_unlimited();
    f();
    let cpu = env.budget().cpu_instruction_cost();
    std::println!("[BENCH] {:48} cpu={:>12} insns", name, cpu);
    BenchResult {
        name,
        cpu_instructions: cpu,
    }
}

#[test]
fn bench_grant_consent_instructions() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    let result = measure_named(
        &env,
        "patient_consent_management::grant_consent (write)",
        || {
            client.grant_consent(&patient, &provider);
        },
    );
    result.assert_cpu_under(BUDGET_GRANT);
}

#[test]
fn bench_revoke_consent_instructions() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    client.grant_consent(&patient, &provider);
    let result = measure_named(
        &env,
        "patient_consent_management::revoke_consent (write)",
        || {
            client.revoke_consent(&patient, &provider);
        },
    );
    result.assert_cpu_under(BUDGET_REVOKE);
}

#[test]
fn bench_check_consent_instructions() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    client.grant_consent(&patient, &provider);
    let result = measure_named(
        &env,
        "patient_consent_management::check_consent (access-check)",
        || {
            client.check_consent(&patient, &provider);
        },
    );
    assert!(client.check_consent(&patient, &provider));
    result.assert_cpu_under(BUDGET_ACCESS_CHECK);
}

#[test]
fn bench_verify_consent_with_audit_instructions() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);
    client.grant_consent(&patient, &provider);
    let result = measure_named(
        &env,
        "patient_consent_management::verify_consent_with_audit (access-check)",
        || {
            client.verify_consent_with_audit(&patient, &provider);
        },
    );
    result.assert_cpu_under(BUDGET_ACCESS_CHECK);
}
