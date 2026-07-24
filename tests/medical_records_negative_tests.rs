//! Negative-path tests for public entrypoints in medical_records.
//!
//! These tests verify that error handling works correctly for invalid
//! inputs and unauthorized access attempts.

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

/// Negative test: Cannot initialize twice.
#[test]
fn negative_double_initialize_fails() {
    let (env, client, admin) = setup();
    let admin2 = Address::generate(&env);
    let result = client.try_initialize(&admin2);
    assert!(result.is_err());
}

/// Negative test: Empty record data is rejected.
#[test]
fn negative_empty_record_data_rejected() {
    let (env, client, admin) = setup();
    let patient = Address::generate(&env);
    let empty_data = soroban_sdk::Bytes::new(&env);
    let data_hash = env.crypto().sha256(&empty_data);
    let result = client.try_write_record(
        &patient,
        &data_hash,
        &String::from_str(&env, ""),
        &0u64,
    );
    assert!(result.is_err());
}

/// Negative test: Zero timestamp is rejected.
#[test]
fn negative_zero_timestamp_rejected() {
    let (env, client, admin) = setup();
    let patient = Address::generate(&env);
    let data = soroban_sdk::Bytes::from_array(&env, &[1u8; 32]);
    let data_hash = env.crypto().sha256(&data);
    let result = client.try_write_record(
        &patient,
        &data_hash,
        &String::from_str(&env, "test"),
        &0u64,
    );
    assert!(result.is_err());
}

/// Negative test: Unauthorized write is rejected.
#[test]
fn negative_unauthorized_write_rejected() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MedicalRecords);
    let client = MedicalRecordsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);

    let unauthorized = Address::generate(&env);
    let patient = Address::generate(&env);
    let data = soroban_sdk::Bytes::from_array(&env, &[2u8; 32]);
    let data_hash = env.crypto().sha256(&data);

    env.mock_auths(&[]);
    let result = client.try_write_record(
        &unauthorized,
        &data_hash,
        &String::from_str(&env, "test"),
        &100u64,
    );
    assert!(result.is_err());
}

/// Negative test: Reading non-existent record returns None.
#[test]
fn negative_read_nonexistent_record_returns_none() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let record_id = client.write_record(
        &patient,
        &env.crypto().sha256(&soroban_sdk::Bytes::from_array(&env, &[3u8; 32])),
        &String::from_str(&env, "test"),
        &env.ledger().timestamp(),
    );

    let nonexistent_id: u64 = record_id + 999999;
    let result = client.try_read_record(&nonexistent_id);
    assert!(result.is_err());
}

/// Negative test: Invalid hash length is rejected.
#[test]
fn negative_invalid_hash_length_rejected() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let short_hash = BytesN::from_array(&env, &[0u8; 16]);
    let result = client.try_write_record(
        &patient,
        &short_hash,
        &String::from_str(&env, "test"),
        &env.ledger().timestamp(),
    );
    assert!(result.is_err());
}
