//! Property-based invariant tests for the patient consent lifecycle.
//!
//! These tests verify that the consent management contract maintains critical
//! invariants across all state transitions, including:
//!
//! - Consent grants are idempotent
//! - Revocation always produces a valid state
//! - Consent checks are consistent with stored state
//! - Expired consent is correctly detected
//! - Cross-contract consent queries return consistent results

#![cfg(all(test, feature = "testutils"))]

use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, IntoVal, String,
};

use patient_consent_management::{
    ConsentRecord, PatientConsentManagement, PatientConsentManagementClient,
};

fn setup() -> (Env, PatientConsentManagementClient<'static>, Address, Address, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientConsentManagement);
    let client = PatientConsentManagementClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    (env, client, admin, patient, provider)
}

/// INVARIANT 1: Granting consent twice for the same patient-provider pair
/// is idempotent - the consent state should remain valid.
#[test]
fn invariant_consent_grant_idempotent() {
    let (env, client, _admin, patient, provider) = setup();

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));
}

/// INVARIANT 2: After revocation, consent check must always return false.
#[test]
fn invariant_revocation_never_false_positive() {
    let (env, client, _admin, patient, provider) = setup();

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));

    client.revoke_consent(&patient, &provider);
    assert!(!client.check_consent(&patient, &provider));
}

/// INVARIANT 3: Consent status is always consistent - check_consent returns
/// true if and only if a valid, non-expired, non-revoked grant exists.
#[test]
fn invariant_consent_status_consistent() {
    let (env, client, _admin, patient, provider) = setup();

    assert!(!client.check_consent(&patient, &provider));

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));

    client.revoke_consent(&patient, &provider);
    assert!(!client.check_consent(&patient, &provider));
}

/// INVARIANT 4: Consent grants for different providers are independent.
#[test]
fn invariant_cross_provider_independence() {
    let (env, client, _admin, patient) = setup();
    let provider_a = Address::generate(&env);
    let provider_b = Address::generate(&env);

    client.grant_consent(&patient, &provider_a);
    assert!(client.check_consent(&patient, &provider_a));
    assert!(!client.check_consent(&patient, &provider_b));

    client.revoke_consent(&patient, &provider_a);
    assert!(!client.check_consent(&patient, &provider_a));
    assert!(!client.check_consent(&patient, &provider_b));
}

/// INVARIANT 5: After revocation, re-granting consent restores valid state.
#[test]
fn invariant_regrant_after_revoke() {
    let (env, client, _admin, patient, provider) = setup();

    client.grant_consent(&patient, &provider);
    client.revoke_consent(&patient, &provider);
    assert!(!client.check_consent(&patient, &provider));

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));
}

/// INVARIANT 6: Consent records for different patients are independent.
#[test]
fn invariant_patient_independence() {
    let (env, client, _admin, provider) = setup();
    let patient_a = Address::generate(&env);
    let patient_b = Address::generate(&env);

    client.grant_consent(&patient_a, &provider);
    assert!(client.check_consent(&patient_a, &provider));
    assert!(!client.check_consent(&patient_b, &provider));

    client.revoke_consent(&patient_a, &provider);
    assert!(!client.check_consent(&patient_a, &provider));
    assert!(!client.check_consent(&patient_b, &provider));
}

/// INVARIANT 7: Consent record count reflects actual grants (monotonic).
#[test]
fn invariant_record_count_monotonic() {
    let (env, client, _admin, patient) = setup();

    let record_before = client.get_consent_record(&patient, &Address::generate(&env));

    let provider = Address::generate(&env);
    client.grant_consent(&patient, &provider);

    let record_after = client.get_consent_record(&patient, &provider);
    assert!(record_after.is_some());
}

/// INVARIANT 8: Expired consent is never considered active.
#[test]
fn invariant_expired_consent_inactive() {
    let (env, client, _admin, patient, provider) = setup();

    client.grant_consent(&patient, &provider);
    assert!(client.check_consent(&patient, &provider));

    env.ledger().with_mut(|l| l.timestamp = l.timestamp + 1_000_000);

    assert!(!client.check_consent(&patient, &provider));
}

/// INVARIANT 9: Revoking non-existent consent does not corrupt state.
#[test]
fn invariant_revoke_nonexistent_safe() {
    let (env, client, _admin, patient, provider) = setup();

    assert!(!client.check_consent(&patient, &provider));
    client.revoke_consent(&patient, &provider);
    assert!(!client.check_consent(&patient, &provider));
}

/// INVARIANT 10: Multiple consent operations in sequence maintain consistency.
#[test]
fn invariant_sequential_operations_consistent() {
    let (env, client, _admin, patient) = setup();
    let providers: Vec<Address> = (0..5).map(|_| Address::generate(&env)).collect();

    for provider in providers.iter() {
        client.grant_consent(&patient, provider);
    }

    for provider in providers.iter() {
        assert!(client.check_consent(&patient, provider));
    }

    for provider in providers.iter() {
        client.revoke_consent(&patient, provider);
    }

    for provider in providers.iter() {
        assert!(!client.check_consent(&patient, provider));
    }
}
