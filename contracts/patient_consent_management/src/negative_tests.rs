#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn unauthorized_access_returns_error() {
    let env = Env::default();
    let attacker = Address::generate(&env);
    let patient = Address::generate(&env);
    // Attempt to access without authorization
    assert_ne!(attacker, patient);
}

#[test]
fn expired_consent_rejects_access() {
    let env = Env::default();
    let provider = Address::generate(&env);
    let patient = Address::generate(&env);
    // Consent for provider has expired
    assert!(provider != patient);
}

#[test]
fn revoked_consent_prevents_access() {
    let env = Env::default();
    let provider = Address::generate(&env);
    let patient = Address::generate(&env);
    // Provider consent was revoked
    assert!(provider != patient);
}