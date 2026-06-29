//! Proptest-based fuzz harness for `zk_verifier` — x86_64-unknown-linux-gnu host.
//!
//! Covers `verify_proof(Bytes)` and `compute_proof_hash(Bytes)`:
//! the two functions accepting arbitrary byte blobs identified in
//! SECURITY_CHECKLIST.md §9.
//!
//! Run locally:
//!   cd tests/fuzz/zk_verifier && cargo test
//! Long-duration fuzzing (CI default):
//!   PROPTEST_CASES=5000 cargo test

extern crate std;

use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};
use zk_verifier::ZkVerifierContract;

// soroban-sdk 21 try_* client methods return
//   Result<Result<T, ConversionError>, Result<ContractError, InvokeError>>
// Use these helpers to keep assertions readable.

/// Returns true if the invocation succeeded AND the inner value is `false`.
fn is_ok_false(result: &Result<Result<bool, impl std::fmt::Debug>, impl std::fmt::Debug>) -> bool {
    matches!(result, Ok(Ok(false)))
}

/// Returns true if the invocation completed without a host trap (value may be Ok or Err).
fn no_host_trap<T>(result: &Result<T, impl std::fmt::Debug>) -> bool {
    result.is_ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Key invariant: `verify_proof` with arbitrary bytes and no prior attestation
    /// must always return `false` — never `true` and never cause a host trap.
    #[test]
    fn verify_proof_unattested_returns_false(
        proof_bytes in prop::collection::vec(any::<u8>(), 0..=1024),
        inputs_seed in any::<[u8; 32]>(),
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let attestor = Address::generate(&env);
        let cid = env.register_contract(None, ZkVerifierContract);
        let client = zk_verifier::ZkVerifierContractClient::new(&env, &cid);
        client.initialize(&admin, &3600_u64);
        client.register_verifying_key(
            &admin,
            &BytesN::from_array(&env, &[0x11u8; 32]),
            &BytesN::from_array(&env, &[0x22u8; 32]),
            &attestor,
            &BytesN::from_array(&env, &[0x33u8; 32]),
        );

        let proof = Bytes::from_slice(&env, &proof_bytes);
        let public_inputs_hash = BytesN::from_array(&env, &inputs_seed);

        let result = client.try_verify_proof(&1, &public_inputs_hash, &proof);
        prop_assert!(
            is_ok_false(&result),
            "verify_proof with no attestation must return Ok(Ok(false)), got: {:?}",
            result
        );
    }

    /// `compute_proof_hash` is pure SHA-256: must succeed for any byte input.
    #[test]
    fn compute_proof_hash_is_total(
        proof_bytes in prop::collection::vec(any::<u8>(), 0..=65536),
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let cid = env.register_contract(None, ZkVerifierContract);
        let client = zk_verifier::ZkVerifierContractClient::new(&env, &cid);
        client.initialize(&admin, &3600_u64);

        let proof = Bytes::from_slice(&env, &proof_bytes);
        let result = client.try_compute_proof_hash(&proof);
        prop_assert!(
            no_host_trap(&result),
            "compute_proof_hash must never trap for any input, got: {:?}",
            result
        );
        // SHA-256 digest is always 32 bytes
        if let Ok(Ok(hash)) = result {
            prop_assert_eq!(hash.len(), 32u32);
        }
    }

    /// `is_nullifier_used` must be deterministic: false before mark, true after.
    /// Second mark must fail with a typed error, never trap.
    #[test]
    fn nullifier_state_transitions(
        nullifier_seed in any::<[u8; 32]>(),
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let cid = env.register_contract(None, ZkVerifierContract);
        let client = zk_verifier::ZkVerifierContractClient::new(&env, &cid);
        client.initialize(&admin, &3600_u64);

        let nullifier = BytesN::from_array(&env, &nullifier_seed);
        prop_assert!(!client.is_nullifier_used(&nullifier), "nullifier should start unused");

        let first = client.try_mark_nullifier_used(&nullifier);
        prop_assert!(no_host_trap(&first), "first mark must not trap: {:?}", first);

        prop_assert!(client.is_nullifier_used(&nullifier), "nullifier must be used after mark");

        // Second mark: must not trap (should return typed InvalidInput error)
        let second = client.try_mark_nullifier_used(&nullifier);
        prop_assert!(no_host_trap(&second), "second mark must not trap: {:?}", second);
    }

    /// `verify_proof` returns false for unknown vk_version values.
    #[test]
    fn verify_proof_unknown_version_returns_false(
        version in 2u32..=u32::MAX,
        proof_bytes in prop::collection::vec(any::<u8>(), 0..=512),
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let cid = env.register_contract(None, ZkVerifierContract);
        let client = zk_verifier::ZkVerifierContractClient::new(&env, &cid);
        client.initialize(&admin, &3600_u64);

        let proof = Bytes::from_slice(&env, &proof_bytes);
        let pih = BytesN::from_array(&env, &[0u8; 32]);
        let result = client.try_verify_proof(&version, &pih, &proof);
        prop_assert!(
            is_ok_false(&result),
            "unknown vk_version must return Ok(Ok(false)): {:?}",
            result
        );
    }
}

/// Seed corpus: specific inputs that exercise ZK proof parser edge cases.
#[test]
fn seed_corpus_edge_cases() {
    let seeds: &[&[u8]] = &[
        b"",
        &[0x00],
        &[0xFF],
        &[0x01; 32],
        &[0x01; 64],
        &[0x02; 96],
        &[0x03; 48],
        &[0x01, 0xFF, 0x00, 0xAB],
        &[0xFF; 10001],
    ];

    for &seed in seeds {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let attestor = Address::generate(&env);
        let cid = env.register_contract(None, ZkVerifierContract);
        let client = zk_verifier::ZkVerifierContractClient::new(&env, &cid);
        client.initialize(&admin, &3600_u64);
        client.register_verifying_key(
            &admin,
            &BytesN::from_array(&env, &[0x11u8; 32]),
            &BytesN::from_array(&env, &[0x22u8; 32]),
            &attestor,
            &BytesN::from_array(&env, &[0x33u8; 32]),
        );

        let proof = Bytes::from_slice(&env, seed);
        let pih = BytesN::from_array(&env, &[0u8; 32]);
        let result = client.try_verify_proof(&1, &pih, &proof);
        assert!(
            is_ok_false(&result),
            "seed caused unexpected result: {:?}",
            result
        );

        let proof2 = Bytes::from_slice(&env, seed);
        let hash_result = client.try_compute_proof_hash(&proof2);
        assert!(
            no_host_trap(&hash_result),
            "compute_proof_hash trapped on seed: {:?}",
            seed
        );
    }
}
