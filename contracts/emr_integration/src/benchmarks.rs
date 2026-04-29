//! Contract performance benchmarks.
//!
//! Naming convention: `bench_<operation>` — CI runs `cargo test bench_`.
#![allow(clippy::unwrap_used)]
extern crate std;
use std::time::Instant;

use super::*;
use soroban_sdk::testutils::{budget::Budget as _, Address as _};
use soroban_sdk::{Address, Env, Map, String};

// ── thresholds (CPU instructions) ──────────────────────────────────────────
const BUDGET_INITIALIZE: u64 = 2_000_000;
const BUDGET_REGISTER_SYSTEM: u64 = 5_000_000;
const BUDGET_PARSE_MESSAGE: u64 = 8_000_000;
const BUDGET_GENERATE_MESSAGE: u64 = 8_000_000;
const BUDGET_GET_SYSTEM: u64 = 2_000_000;
const BUDGET_VALIDATE_MESSAGE: u64 = 5_000_000;

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
            self.name,
            self.cpu_instructions,
            self.memory_bytes,
            self.wall_us
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

fn register_system(env: &Env, client: &EMRIntegrationContractClient<'_>, admin: &Address) {
    client.register_emr_system(
        admin,
        &String::from_str(env, "epic-bench"),
        &String::from_str(env, "Epic"),
        &String::from_str(env, "bench@epic.test"),
        &String::from_str(env, "2026.1"),
        &soroban_sdk::vec![
            env,
            String::from_str(env, "HL7 v2"),
            String::from_str(env, "HL7 v3"),
        ],
        &soroban_sdk::vec![env, String::from_str(env, "mllp://epic-bench")],
    );
}

fn sample_metadata(env: &Env) -> Map<String, String> {
    let mut m = Map::new(env);
    m.set(
        String::from_str(env, "control_id"),
        String::from_str(env, "CTRL-BENCH"),
    );
    m.set(
        String::from_str(env, "patient_id"),
        String::from_str(env, "PAT-BENCH"),
    );
    m.set(
        String::from_str(env, "patient_name"),
        String::from_str(env, "DOE^JANE"),
    );
    m.set(
        String::from_str(env, "document_title"),
        String::from_str(env, "Benchmark Record"),
    );
    m.set(
        String::from_str(env, "document_text"),
        String::from_str(env, "Benchmark text content."),
    );
    m
}

// ── benchmark tests ─────────────────────────────────────────────────────────

#[test]
fn bench_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let fhir = Address::generate(&env);

    let r = measure(&env, "emr_integration::initialize", || {
        client.initialize(&admin, &fhir);
    });
    r.print();
    r.assert_cpu_under(BUDGET_INITIALIZE);
}

#[test]
fn bench_register_emr_system() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &Address::generate(&env));

    let r = measure(&env, "emr_integration::register_emr_system (write)", || {
        register_system(&env, &client, &admin);
    });
    r.print();
    r.assert_cpu_under(BUDGET_REGISTER_SYSTEM);
}

#[test]
fn bench_get_emr_system() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &Address::generate(&env));
    register_system(&env, &client, &admin);

    let r = measure(&env, "emr_integration::get_emr_system (read)", || {
        client.get_emr_system(&String::from_str(&env, "epic-bench"));
    });
    r.print();
    r.assert_cpu_under(BUDGET_GET_SYSTEM);
}

#[test]
fn bench_generate_message() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &Address::generate(&env));
    register_system(&env, &client, &admin);
    let sender = Address::generate(&env);

    let r = measure(&env, "emr_integration::generate_message (write)", || {
        client.generate_message(
            &sender,
            &String::from_str(&env, "msg-bench-1"),
            &String::from_str(&env, "epic-bench"),
            &MessagingStandard::HL7v2,
            &String::from_str(&env, "2.5.1"),
            &String::from_str(&env, "ADT^A01"),
            &CharacterEncoding::UTF8,
            &TransportProtocol::MLLP,
            &String::from_str(&env, "application/hl7-v2"),
            &sample_metadata(&env),
        );
    });
    r.print();
    r.assert_cpu_under(BUDGET_GENERATE_MESSAGE);
}

#[test]
fn bench_parse_message() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &Address::generate(&env));
    register_system(&env, &client, &admin);
    let sender = Address::generate(&env);

    let payload = String::from_str(
        &env,
        "MSH|^~\\&|EPIC|BENCH|LAB|HL7|20260101120000||ADT^A01|CTRL-001|P|2.5.1",
    );

    let r = measure(&env, "emr_integration::parse_message (read+write)", || {
        client.parse_message(
            &sender,
            &String::from_str(&env, "msg-parse-bench"),
            &String::from_str(&env, "epic-bench"),
            &CharacterEncoding::UTF8,
            &TransportProtocol::MLLP,
            &String::from_str(&env, "application/hl7-v2"),
            &payload,
        );
    });
    r.print();
    r.assert_cpu_under(BUDGET_PARSE_MESSAGE);
}

#[test]
fn bench_validate_message() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &Address::generate(&env));
    register_system(&env, &client, &admin);
    let sender = Address::generate(&env);

    client.generate_message(
        &sender,
        &String::from_str(&env, "msg-to-validate"),
        &String::from_str(&env, "epic-bench"),
        &MessagingStandard::HL7v2,
        &String::from_str(&env, "2.5.1"),
        &String::from_str(&env, "ADT^A01"),
        &CharacterEncoding::UTF8,
        &TransportProtocol::MLLP,
        &String::from_str(&env, "application/hl7-v2"),
        &sample_metadata(&env),
    );

    let r = measure(
        &env,
        "emr_integration::validate_message (read+write)",
        || {
            client.validate_message(
                &sender,
                &String::from_str(&env, "report-bench-1"),
                &String::from_str(&env, "msg-to-validate"),
            );
        },
    );
    r.print();
    r.assert_cpu_under(BUDGET_VALIDATE_MESSAGE);
}

#[test]
fn bench_storage_write_vs_read_ratio() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, EMRIntegrationContract);
    let client = EMRIntegrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &Address::generate(&env));
    register_system(&env, &client, &admin);
    let sender = Address::generate(&env);

    let write = measure(&env, "emr_integration::generate_message (write)", || {
        client.generate_message(
            &sender,
            &String::from_str(&env, "msg-ratio-test"),
            &String::from_str(&env, "epic-bench"),
            &MessagingStandard::HL7v2,
            &String::from_str(&env, "2.5.1"),
            &String::from_str(&env, "ADT^A01"),
            &CharacterEncoding::UTF8,
            &TransportProtocol::MLLP,
            &String::from_str(&env, "application/hl7-v2"),
            &sample_metadata(&env),
        );
    });

    let read = measure(&env, "emr_integration::get_message (read)", || {
        client.get_message(&String::from_str(&env, "msg-ratio-test"));
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
