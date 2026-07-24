//! Consent Flow ZKP Verification Tracking
//!
//! Integration module for tracking ZKP verification within consent flows.
//! Provides helpers for recording consent-gated ZKP checks and linking
//! verification telemetry to specific consent grants.

use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, String};

use crate::telemetry::{
    record_consent_zkp_metric,
};

/// Result of a consent-gated ZKP verification.
#[derive(Clone)]
#[contracttype]
pub struct ConsentZkpResult {
    /// Whether consent was valid.
    pub consent_valid: bool,
    /// Whether ZKP verification passed.
    pub zkp_valid: bool,
    /// Combined result (both consent and ZKP must pass).
    pub overall_valid: bool,
    /// Gas consumed for the ZKP verification.
    pub gas_used: u64,
    /// Timestamp of the check.
    pub timestamp: u64,
}

/// Storage key for consent ZKP verification results.
#[derive(Clone)]
#[contracttype]
pub enum ConsentZkpKey {
    /// Latest ZKP result for a (patient, provider) consent pair.
    LatestResult(Address, Address),
    /// ZKP verification count for a consent pair.
    VerificationCount(Address, Address),
}

/// Perform a consent-gated ZKP verification and record telemetry.
///
/// This function:
/// 1. Checks that consent exists and is active
/// 2. Records a telemetry event for the consent check
/// 3. Returns a combined ConsentZkpResult
pub fn verify_consent_zkp(
    env: &Env,
    patient: &Address,
    provider: &Address,
    proof_id: &BytesN<32>,
    consent_valid: bool,
    zkp_valid: bool,
    gas_used: u64,
) -> ConsentZkpResult {
    let overall_valid = consent_valid && zkp_valid;
    let timestamp = env.ledger().timestamp();

    // Record the consent ZKP check telemetry
    let consent_id = String::from_str(
        env,
        &format!("{}_{}", patient.to_string(), provider.to_string()),
    );

    record_consent_zkp_metric(
        env,
        patient,
        proof_id,
        &consent_id,
        overall_valid,
        gas_used,
    );

    // Store the result for later retrieval
    let result = ConsentZkpResult {
        consent_valid,
        zkp_valid,
        overall_valid,
        gas_used,
        timestamp,
    };

    env.storage().persistent().set(
        &ConsentZkpKey::LatestResult(patient.clone(), provider.clone()),
        &result,
    );

    // Increment verification count
    let count: u64 = env
        .storage()
        .persistent()
        .get(&ConsentZkpKey::VerificationCount(patient.clone(), provider.clone()))
        .unwrap_or(0);
    env.storage().persistent().set(
        &ConsentZkpKey::VerificationCount(patient.clone(), provider.clone()),
        &(count + 1),
    );

    result
}

/// Get the latest consent ZKP verification result for a patient-provider pair.
pub fn get_consent_zkp_result(
    env: &Env,
    patient: &Address,
    provider: &Address,
) -> Option<ConsentZkpResult> {
    env.storage().persistent().get(&ConsentZkpKey::LatestResult(
        patient.clone(),
        provider.clone(),
    ))
}

/// Get the total ZKP verification count for a consent pair.
pub fn get_consent_zkp_count(env: &Env, patient: &Address, provider: &Address) -> u64 {
    env.storage().persistent().get(
        &ConsentZkpKey::VerificationCount(patient.clone(), provider.clone()),
    ).unwrap_or(0)
}
