// #![cfg(test)]

use super::*;
// use soroban_sdk::env::Env;
// use soroban_sdk::testutils::Address as _;
// use soroban_sdk::Env;
// use soroban_sdk::{vec, Address, Env, IntoVal};

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Env, IntoVal};

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let medical_records = Address::generate(&env);
    let identity_registry = Address::generate(&env);
    let governor = Address::generate(&env);
    let dispute = Address::generate(&env);
    let compliance = Address::generate(&env);

    let result = EmergencyAccess::initialize(
        env.clone(),
        admin,
        medical_records,
        identity_registry,
        governor,
        dispute,
        compliance,
        3600, // 1 hour max
        2,    // 2 approvals required
        300,  // 5 min cooldown
    );

    assert!(result.is_ok());
}

#[test]
fn test_register_emergency_authority() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let authority = Address::generate(&env);

    // Initialize first
    EmergencyAccess::initialize(
        env.clone(),
        admin.clone(),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        3600,
        2,
        300,
    )
    .unwrap();

    let result = EmergencyAccess::register_emergency_authority(
        env.clone(),
        admin,
        authority,
        "doctor".into_val(&env),
        "emergency_medicine".into_val(&env),
        "MD12345".into_val(&env),
        1,
    );

    assert!(result.is_ok());
}

#[test]
fn test_request_emergency_access() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let authority = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize
    EmergencyAccess::initialize(
        env.clone(),
        admin.clone(),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        3600,
        2,
        300,
    )
    .unwrap();

    // Register authority
    EmergencyAccess::register_emergency_authority(
        env.clone(),
        admin,
        authority.clone(),
        "doctor".into_val(&env),
        "emergency_medicine".into_val(&env),
        "MD12345".into_val(&env),
        1,
    )
    .unwrap();

    // Request access
    let mfa_factors = vec![&env, MFAFactor::Password, MFAFactor::Biometric];
    let record_scope = vec![&env, 1u64, 2u64];

    let result = EmergencyAccess::request_emergency_access(
        env.clone(),
        authority,
        patient,
        "cardiac_arrest".into_val(&env),
        "Patient in cardiac arrest, need immediate access to medical history".into_val(&env),
        record_scope,
        1800, // 30 minutes
        mfa_factors,
    );

    assert!(result.is_ok());
    let request_id = result.unwrap();
    assert_eq!(request_id, 1);
}

#[test]
fn test_approve_emergency_request() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let authority1 = Address::generate(&env);
    let authority2 = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize with 2 required approvals
    EmergencyAccess::initialize(
        env.clone(),
        admin.clone(),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        3600,
        2,
        300,
    )
    .unwrap();

    // Register authorities
    EmergencyAccess::register_emergency_authority(
        env.clone(),
        admin.clone(),
        authority1.clone(),
        "doctor".into_val(&env),
        "emergency_medicine".into_val(&env),
        "MD12345".into_val(&env),
        1,
    )
    .unwrap();

    EmergencyAccess::register_emergency_authority(
        env.clone(),
        admin,
        authority2.clone(),
        "nurse".into_val(&env),
        "emergency".into_val(&env),
        "RN67890".into_val(&env),
        1,
    )
    .unwrap();

    // Request access
    let mfa_factors = vec![&env, MFAFactor::Password, MFAFactor::Biometric];
    let record_scope = vec![&env, 1u64];

    let request_id = EmergencyAccess::request_emergency_access(
        env.clone(),
        authority1.clone(),
        patient.clone(),
        "trauma".into_val(&env),
        "Multiple injuries from car accident".into_val(&env),
        record_scope,
        1800,
        mfa_factors,
    )
    .unwrap();

    // First approval
    // let result1 = EmergencyAccess::approve_emergency_request(env.clone(), authority1, request_id);
    let result1 =
        EmergencyAccess::approve_emergency_request(env.clone(), authority1.clone(), request_id);
    assert!(result1.is_ok());

    // Second approval should create grant
    let result2 = EmergencyAccess::approve_emergency_request(env.clone(), authority2, request_id);
    assert!(result2.is_ok());

    // Check if access is granted
    // let has_access = EmergencyAccess::has_emergency_access(env, authority1, patient, 1u64);
    let has_access = EmergencyAccess::has_emergency_access(env, authority1.clone(), patient, 1u64);
    assert!(has_access);
}

#[test]
fn test_revoke_emergency_access() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let authority = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize
    EmergencyAccess::initialize(
        env.clone(),
        admin.clone(),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
        3600,
        1, // 1 approval required
        300,
    )
    .unwrap();

    // Register authority
    EmergencyAccess::register_emergency_authority(
        env.clone(),
        admin,
        authority.clone(),
        "doctor".into_val(&env),
        "emergency_medicine".into_val(&env),
        "MD12345".into_val(&env),
        1,
    )
    .unwrap();

    // Request and approve access
    let mfa_factors = vec![&env, MFAFactor::Password, MFAFactor::Biometric];
    let record_scope = vec![&env, 1u64];

    let request_id = EmergencyAccess::request_emergency_access(
        env.clone(),
        authority.clone(),
        patient.clone(),
        "emergency".into_val(&env),
        "Critical condition".into_val(&env),
        record_scope,
        1800,
        mfa_factors,
    )
    .unwrap();

    EmergencyAccess::approve_emergency_request(env.clone(), authority.clone(), request_id).unwrap();

    // Check access granted
    let has_access_before = EmergencyAccess::has_emergency_access(
        env.clone(),
        authority.clone(),
        patient.clone(),
        1u64,
    );
    assert!(has_access_before);

    // Get grant ID (simplified - in real scenario would track this)
    // For test, assume grant_id = 1
    let result = EmergencyAccess::revoke_emergency_access(env.clone(), authority.clone(), 1u64);
    assert!(result.is_ok());

    // Check access revoked
    let has_access_after = EmergencyAccess::has_emergency_access(env, authority, patient, 1u64);
    assert!(!has_access_after);
}
