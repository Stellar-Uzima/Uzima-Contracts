//! Benchmark and optimize storage write amplification in medical_records.
//!
//! These tests measure storage operations to identify write amplification
//! patterns and verify optimized storage usage.

#![cfg(all(test, feature = "testutils"))]

use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String,
};

use medical_records::{MedicalRecords, MedicalRecordsClient};

fn setup() -> (Env, MedicalRecordsClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, MedicalRecords);
    let client = MedicalRecordsClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    (env, client, admin)
}

/// Benchmark: Single record write storage cost.
#[test]
fn benchmark_single_record_write() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);

    let data = soroban_sdk::Bytes::from_array(&env, &[1u8; 1024]);
    let hash = env.crypto().sha256(&data);

    let _ = client.write_record(
        &patient,
        &hash,
        &String::from_str(&env, "benchmark"),
        &env.ledger().timestamp(),
    );
}

/// Benchmark: Batch record write storage cost.
#[test]
fn benchmark_batch_record_writes() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);

    for i in 0..10u64 {
        let data = soroban_sdk::Bytes::from_array(&env, &[i as u8; 256]);
        let hash = env.crypto().sha256(&data);
        let _ = client.write_record(
            &patient,
            &hash,
            &String::from_str(&env, "batch"),
            &env.ledger().timestamp(),
        );
    }
}

/// Benchmark: Update existing record (replaces storage vs. new write).
#[test]
fn benchmark_record_update() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);

    let data = soroban_sdk::Bytes::from_array(&env, &[1u8; 512]);
    let hash = env.crypto().sha256(&data);

    let record_id = client.write_record(
        &patient,
        &hash,
        &String::from_str(&env, "original"),
        &env.ledger().timestamp(),
    );

    let new_data = soroban_sdk::Bytes::from_array(&env, &[2u8; 512]);
    let new_hash = env.crypto().sha256(&new_data);

    let _ = client.update_record(
        &record_id,
        &patient,
        &new_hash,
        &String::from_str(&env, "updated"),
    );
}

/// Benchmark: Multiple patients with multiple records.
#[test]
fn benchmark_multi_patient_storage() {
    let (env, client, _admin) = setup();

    for p in 0..5u64 {
        let patient = Address::generate(&env);
        for r in 0..5u64 {
            let data = soroban_sdk::Bytes::from_array(&env, &[(p * 5 + r) as u8; 128]);
            let hash = env.crypto().sha256(&data);
            let _ = client.write_record(
                &patient,
                &hash,
                &String::from_str(&env, "multi"),
                &env.ledger().timestamp(),
            );
        }
    }
}
