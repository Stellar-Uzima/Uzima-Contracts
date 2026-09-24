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
