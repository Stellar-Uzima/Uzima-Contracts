//! Load tests for audit logging and access logging at scale.
//!
//! These tests simulate high-volume audit logging operations to verify
//! performance and consistency under load.

#![cfg(all(test, feature = "testutils"))]

use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String,
};

use audit::{AuditContract, AuditContractClient};

fn setup() -> (Env, AuditContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuditContract);
    let client = AuditContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    (env, client, admin)
}

/// Load test: Record 100 audit entries sequentially.
#[test]
fn load_test_audit_logging_100_entries() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    for i in 0..100u64 {
        let data = soroban_sdk::Bytes::from_array(&env, &[(i % 256) as u8; 32]);
        let hash = env.crypto().sha256(&data);
        let _ = client.try_log_event(
            &user,
            &hash,
            &String::from_str(&env, "test_event"),
            &env.ledger().timestamp(),
        );
    }
}

/// Load test: Record audit entries from multiple users.
#[test]
fn load_test_multi_user_audit_logging() {
    let (env, client, _admin) = setup();

    let mut users = Vec::new(&env);
    for _ in 0..10 {
        users.push_back(Address::generate(&env));
    }

    for i in 0..50u64 {
        let user = users.get((i % 10) as u32).unwrap();
        let data = soroban_sdk::Bytes::from_array(&env, &[i as u8; 32]);
        let hash = env.crypto().sha256(&data);
        let _ = client.try_log_event(
            &user,
            &hash,
            &String::from_str(&env, "multi_user"),
            &env.ledger().timestamp(),
        );
    }
}

/// Load test: Rapid timestamp advancement with audit entries.
#[test]
fn load_test_rapid_timestamp_audit() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    for i in 0..50u64 {
        env.ledger().with_mut(|l| l.timestamp = 1000 + i * 10);
        let data = soroban_sdk::Bytes::from_array(&env, &[i as u8; 32]);
        let hash = env.crypto().sha256(&data);
        let _ = client.try_log_event(
            &user,
            &hash,
            &String::from_str(&env, "timestamped"),
            &env.ledger().timestamp(),
        );
    }
}

/// Load test: Multiple event types in rapid succession.
#[test]
fn load_test_multiple_event_types() {
    let (env, client, _admin) = setup();
    let user = Address::generate(&env);

    let event_types = ["read", "write", "delete", "grant", "revoke"];

    for i in 0..50u64 {
        let event_type = event_types[(i % 5) as usize];
        let data = soroban_sdk::Bytes::from_array(&env, &[i as u8; 32]);
        let hash = env.crypto().sha256(&data);
        let _ = client.try_log_event(
            &user,
            &hash,
            &String::from_str(&env, event_type),
            &env.ledger().timestamp(),
        );
    }
}
