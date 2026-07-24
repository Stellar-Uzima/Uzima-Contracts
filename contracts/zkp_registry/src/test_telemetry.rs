#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    (env, admin)
}

#[test]
fn test_telemetry_event_types() {
    let (env, _) = setup();
    let _ = env;

    // Verify all telemetry event types can be constructed
    let _ = TelemetryEventType::ProofSubmitted;
    let _ = TelemetryEventType::VerificationPassed;
    let _ = TelemetryEventType::VerificationFailed;
    let _ = TelemetryEventType::BatchVerificationCompleted;
    let _ = TelemetryEventType::RangeProofVerified;
    let _ = TelemetryEventType::CredentialProofVerified;
    let _ = TelemetryEventType::RecursiveProofComposed;
    let _ = TelemetryEventType::ConsentZkpCheck;
}

#[test]
fn test_emit_telemetry_event_stores_event() {
    let (env, actor) = setup();
    let proof_id = soroban_sdk::BytesN::<32>::from_array(&env, &[1u8; 32]);
    let context = String::from_str(&env, "test_circuit");

    emit_telemetry_event(
        &env,
        TelemetryEventType::VerificationPassed,
        &actor,
        &proof_id,
        &context,
        5000,
    );

    // Verify counter incremented
    let counter: u64 = env
        .storage()
        .persistent()
        .get(&TelemetryKey::EventCounter)
        .unwrap_or(0);
    assert_eq!(counter, 1);
}

#[test]
fn test_emit_telemetry_event_updates_aggregated() {
    let (env, actor) = setup();
    let proof_id = soroban_sdk::BytesN::<32>::from_array(&env, &[2u8; 32]);
    let context = String::from_str(&env, "circuit_a");

    emit_telemetry_event(
        &env,
        TelemetryEventType::ProofSubmitted,
        &actor,
        &proof_id,
        &context,
        1000,
    );

    let agg = get_aggregated_telemetry(&env);
    assert_eq!(agg.total_attempts, 1);
    assert_eq!(agg.total_passed, 0);
    assert_eq!(agg.total_failed, 0);
    assert_eq!(agg.total_gas, 1000);
}

#[test]
fn test_emit_telemetry_passed_and_failed() {
    let (env, actor) = setup();

    // Passed verification
    let proof_id_pass = soroban_sdk::BytesN::<32>::from_array(&env, &[3u8; 32]);
    emit_telemetry_event(
        &env,
        TelemetryEventType::VerificationPassed,
        &actor,
        &proof_id_pass,
        &String::from_str(&env, "c1"),
        5000,
    );

    // Failed verification
    let proof_id_fail = soroban_sdk::BytesN::<32>::from_array(&env, &[4u8; 32]);
    emit_telemetry_event(
        &env,
        TelemetryEventType::VerificationFailed,
        &actor,
        &proof_id_fail,
        &String::from_str(&env, "c1"),
        3000,
    );

    let agg = get_aggregated_telemetry(&env);
    assert_eq!(agg.total_attempts, 0);
    assert_eq!(agg.total_passed, 1);
    assert_eq!(agg.total_failed, 1);
    assert_eq!(agg.total_gas, 8000);
}

#[test]
fn test_record_consent_zkp_metric() {
    let (env, actor) = setup();
    let proof_id = soroban_sdk::BytesN::<32>::from_array(&env, &[5u8; 32]);
    let consent_id = String::from_str(&env, "consent_abc");

    record_consent_zkp_metric(&env, &actor, &proof_id, &consent_id, true, 2000);

    let agg = get_aggregated_telemetry(&env);
    assert_eq!(agg.total_attempts, 1);
    assert_eq!(agg.event_count, 1);
}

#[test]
fn test_record_consent_zkp_metric_failed() {
    let (env, actor) = setup();
    let proof_id = soroban_sdk::BytesN::<32>::from_array(&env, &[6u8; 32]);
    let consent_id = String::from_str(&env, "consent_xyz");

    record_consent_zkp_metric(&env, &actor, &proof_id, &consent_id, false, 1500);

    let agg = get_aggregated_telemetry(&env);
    assert_eq!(agg.total_failed, 1);
    assert_eq!(agg.total_gas, 1500);
}

#[test]
fn test_get_telemetry_event() {
    let (env, actor) = setup();
    let proof_id = soroban_sdk::BytesN::<32>::from_array(&env, &[7u8; 32]);
    let context = String::from_str(&env, "test");

    emit_telemetry_event(
        &env,
        TelemetryEventType::RangeProofVerified,
        &actor,
        &proof_id,
        &context,
        4000,
    );

    let event_id = derive_event_id(&env, &proof_id, TelemetryEventType::RangeProofVerified);
    let event = get_telemetry_event(&env, &event_id);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.event_type, TelemetryEventType::RangeProofVerified);
    assert_eq!(event.gas_used, 4000);
}

#[test]
fn test_multiple_telemetry_events() {
    let (env, actor) = setup();

    for i in 0..10u8 {
        let proof_id = soroban_sdk::BytesN::<32>::from_array(&env, &[i; 32]);
        let event_type = if i % 2 == 0 {
            TelemetryEventType::VerificationPassed
        } else {
            TelemetryEventType::VerificationFailed
        };
        emit_telemetry_event(
            &env,
            event_type,
            &actor,
            &proof_id,
            &String::from_str(&env, "circuit"),
            (i as u64) * 1000,
        );
    }

    let agg = get_aggregated_telemetry(&env);
    assert_eq!(agg.event_count, 10);
    assert_eq!(agg.total_passed, 5);
    assert_eq!(agg.total_failed, 5);
    assert_eq!(agg.total_gas, 45000); // 0+1000+2000+...+9000
}

#[test]
fn test_batch_verification_telemetry() {
    let (env, actor) = setup();
    let proof_id = soroban_sdk::BytesN::<32>::from_array(&env, &[10u8; 32]);

    emit_telemetry_event(
        &env,
        TelemetryEventType::BatchVerificationCompleted,
        &actor,
        &proof_id,
        &String::from_str(&env, "batch_1"),
        15000,
    );

    let agg = get_aggregated_telemetry(&env);
    assert_eq!(agg.total_passed, 1);
    assert_eq!(agg.total_gas, 15000);
}
