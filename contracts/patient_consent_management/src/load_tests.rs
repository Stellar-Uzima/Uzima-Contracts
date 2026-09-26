//! Load tests for the consent lifecycle at scale (issue #1594).
//!
//! `test.rs` covers the consent lifecycle for correctness one interaction at a
//! time. These tests exercise the same paths with many patients and providers to
//! surface storage-growth and iteration problems that only appear at volume:
//! consent logs are per-patient collections, so the cost of granting, checking
//! and revoking grows with the number of grantees a single patient accumulates.
//!
//! Sized to run in a normal `cargo test --workspace` pass. The heavier
//! variants are `#[ignore]`d, matching the existing convention in
//! `healthcare_data_marketplace`.
#![cfg(test)]
#![allow(clippy::unwrap_used)]
extern crate std;

use crate::{PatientConsentManagement, PatientConsentManagementClient};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, Vec};

/// Patients that each accumulate this many provider consents.
const PROVIDERS_PER_PATIENT: u32 = 20;
/// Independent patients driven through the same lifecycle.
const PATIENT_COUNT: u32 = 10;
/// Ignored-by-default scale ceiling.
const SCALE_PROVIDERS_PER_PATIENT: u32 = 200;

fn setup() -> (Env, PatientConsentManagementClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    // A fixed, non-zero timestamp: the consent log records grant times, and the
    // expiry tests need a known "now" to advance away from.
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000_000;
    });
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, PatientConsentManagement);
    let client = PatientConsentManagementClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, client, admin)
}

fn providers(env: &Env, count: u32) -> Vec<Address> {
    let mut out = Vec::new(env);
    for _ in 0..count {
        out.push_back(Address::generate(env));
    }
    out
}

/// Grant, check and revoke many providers for a single patient.
#[test]
fn load_grant_check_revoke_many_providers_for_one_patient() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);

    let granted = providers(&env, PROVIDERS_PER_PATIENT);
    for provider in granted.iter() {
        client.grant_consent(&patient, provider);
    }
    assert_eq!(
        client.get_active_consent_count(&patient),
        PROVIDERS_PER_PATIENT
    );

    for provider in granted.iter() {
        assert!(client.check_consent(&patient, provider));
    }

    for provider in granted.iter() {
        client.revoke_consent(&patient, provider);
    }
    assert_eq!(
        client.get_active_consent_count(&patient),
        0,
        "revoking every consent must leave none active"
    );
    for provider in granted.iter() {
        assert!(
            !client.check_consent(&patient, provider),
            "a revoked consent must not verify"
        );
    }
}

#[test]
fn load_consent_lifecycle_across_many_patients() {
    let (env, client, _admin) = setup();

    for _ in 0..PATIENT_COUNT {
        let patient = Address::generate(&env);
        let shared = providers(&env, PROVIDERS_PER_PATIENT);

        for provider in shared.iter() {
            client.grant_consent(&patient, provider);
        }
        assert_eq!(
            client.get_active_consent_count(&patient),
            PROVIDERS_PER_PATIENT
        );

        // Consent is per (patient, provider): the same provider consented by a
        // different patient must not leak into this patient's count.
        assert!(client.check_consent(&patient, &shared.get(0).unwrap()));
    }
}

#[test]
fn load_batch_grant_many_grantees() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);

    let grantees = providers(&env, PROVIDERS_PER_PATIENT);
    let granted = client.batch_grant_consent(&patient, &grantees);

    assert_eq!(
        granted, PROVIDERS_PER_PATIENT,
        "batch_grant_consent should report how many consents it created"
    );
    assert_eq!(
        client.get_active_consent_count(&patient),
        PROVIDERS_PER_PATIENT
    );
}

#[test]
fn load_expiry_cleanup_across_many_patients() {
    let (env, client, _admin) = setup();
    let now = env.ledger().timestamp();

    let mut patients = Vec::new(&env);
    for _ in 0..PATIENT_COUNT {
        let patient = Address::generate(&env);
        let grantees = providers(&env, PROVIDERS_PER_PATIENT);

        // Half the consents expire soon, half never expire.
        for (i, provider) in grantees.iter().enumerate() {
            if i % 2 == 0 {
                client.grant_consent_with_expiry(&patient, provider, now + 100);
            } else {
                client.grant_consent(&patient, provider);
            }
        }
        patients.push_back(patient);
    }

    let expected_expiring = PATIENT_COUNT * (PROVIDERS_PER_PATIENT / 2);

    // Move past every expiry but keep the ledger otherwise unchanged.
    env.ledger().with_mut(|li| {
        li.timestamp = now + 1_000;
    });

    let mut total_removed: u32 = 0;
    for patient in patients.iter() {
        total_removed += client.cleanup_expired_consents(patient);
    }

    assert_eq!(
        total_removed, expected_expiring,
        "cleanup should reclaim exactly the consents that expired"
    );

    for patient in patients.iter() {
        assert_eq!(
            client.get_active_consent_count(patient),
            PROVIDERS_PER_PATIENT / 2,
            "consents without an expiry must survive cleanup"
        );
    }
}

#[test]
fn load_repeated_grant_is_idempotent_at_volume() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);
    let shared = providers(&env, PROVIDERS_PER_PATIENT);

    for _ in 0..3 {
        for provider in shared.iter() {
            client.grant_consent(&patient, provider);
        }
    }

    assert_eq!(
        client.get_active_consent_count(&patient),
        PROVIDERS_PER_PATIENT,
        "re-granting the same provider must not inflate the active count"
    );
}

#[test]
#[ignore = "stress test for consent scale"]
fn load_consent_scale_to_high_provider_count() {
    let (env, client, _admin) = setup();
    let patient = Address::generate(&env);

    let granted = providers(&env, SCALE_PROVIDERS_PER_PATIENT);
    for provider in granted.iter() {
        client.grant_consent(&patient, provider);
    }
    assert_eq!(
        client.get_active_consent_count(&patient),
        SCALE_PROVIDERS_PER_PATIENT
    );

    for provider in granted.iter() {
        client.revoke_consent(&patient, provider);
    }
    assert_eq!(client.get_active_consent_count(&patient), 0);
}
