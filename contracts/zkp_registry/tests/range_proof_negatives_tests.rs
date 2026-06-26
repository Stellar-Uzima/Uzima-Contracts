#![cfg(test)]

//! Integration tests that exercise the negative test corpus in
//! `range_proof_negatives/`. Each test imports a pre-built invalid
//! `RangeProof` and asserts the expected error.

mod range_proof_negatives;

use soroban_sdk::Env;
use zkp_registry::Error;

/// Test Vector 1: Wrong version byte → InvalidProofFormat
#[test]
fn negative_wrong_version_byte() {
    let env = Env::default();
    env.mock_all_auths();
    let (proof, _proof_id) = range_proof_negatives::case_wrong_version_byte(&env);

    let result = try_verify(&env, &proof);
    assert_eq!(result, Err(Ok(Error::InvalidProofFormat)));
}

/// Test Vector 2: Tampered commitment → InconsistentCommitment
#[test]
fn negative_tampered_commitment() {
    let env = Env::default();
    env.mock_all_auths();
    let (proof, _proof_id) = range_proof_negatives::case_tampered_commitment(&env);

    let (client, admin) = range_proof_negatives::setup(&env);
    range_proof_negatives::register_bulletproof_circuit(
        &client, &env, &admin, &proof.vk_hash,
    );

    let result = client.try_verify_range_proof(&proof);
    assert_eq!(result, Err(Ok(Error::InconsistentCommitment)));
}

/// Test Vector 3: Empty proof_data → MalformedProof
#[test]
fn negative_empty_proof_data() {
    let env = Env::default();
    env.mock_all_auths();
    let (proof, _proof_id) = range_proof_negatives::case_empty_proof_data(&env);

    let result = try_verify(&env, &proof);
    assert_eq!(result, Err(Ok(Error::MalformedProof)));
}

/// Test Vector 4: Proof_data too short → MalformedProof
#[test]
fn negative_proof_data_too_short() {
    let env = Env::default();
    env.mock_all_auths();
    let (proof, _proof_id) = range_proof_negatives::case_proof_data_too_short(&env);

    let result = try_verify(&env, &proof);
    assert_eq!(result, Err(Ok(Error::MalformedProof)));
}

/// Test Vector 5: Unregistered VK → CircuitNotFound
#[test]
fn negative_unregistered_vk() {
    let env = Env::default();
    env.mock_all_auths();
    let (proof, _proof_id) = range_proof_negatives::case_unregistered_vk(&env);

    let result = try_verify(&env, &proof);
    assert_eq!(result, Err(Ok(Error::CircuitNotFound)));
}

/// Test Vector 6: Mismatched encrypted_value → InconsistentCommitment
#[test]
fn negative_mismatched_encrypted_value() {
    let env = Env::default();
    env.mock_all_auths();
    let (proof, _proof_id) = range_proof_negatives::case_mismatched_encrypted_value(&env);

    let (client, admin) = range_proof_negatives::setup(&env);
    range_proof_negatives::register_bulletproof_circuit(
        &client, &env, &admin, &proof.vk_hash,
    );

    let result = client.try_verify_range_proof(&proof);
    assert_eq!(result, Err(Ok(Error::InconsistentCommitment)));
}

/// Test Vector 7: Invalid range (min >= max) → InvalidRange
#[test]
fn negative_invalid_range() {
    let env = Env::default();
    env.mock_all_auths();
    let (proof, _proof_id) = range_proof_negatives::case_invalid_range(&env);

    let result = try_verify(&env, &proof);
    assert_eq!(result, Err(Ok(Error::InvalidRange)));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Call `verify_range_proof` on a freshly-deployed contract.
fn try_verify(
    env: &Env,
    proof: &zkp_registry::RangeProof,
) -> Result<
    Result<bool, soroban_sdk::ConversionError>,
    Result<zkp_registry::Error, soroban_sdk::InvokeError>,
> {
    let (client, _admin) = range_proof_negatives::setup(env);
    client.try_verify_range_proof(proof)
}
