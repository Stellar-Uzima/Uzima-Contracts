#![cfg(test)]

//! Property-based tests for range proof verification (TD-003).
//!
//! Exercises the SHA-256 commitment binding inside
//! `verify_range_proof_internal` with randomly-generated inputs to ensure
//! the cryptographic checks hold for arbitrary values.

use proptest::prelude::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, Bytes, BytesN, Env, String};
use zkp_registry::{Error, ZKPRegistryClient, ZKPType};

// ---------------------------------------------------------------------------
// Local helpers
// ---------------------------------------------------------------------------

fn canonical_circuit_id(env: &Env, vk_hash: &BytesN<32>) -> String {
    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_slice(env, b"UZIMA_RANGE_CIRCUIT_V1"));
    payload.append(&Bytes::from_slice(env, &vk_hash.to_array()));
    let digest: BytesN<32> = env.crypto().sha256(&payload).into();
    let bytes = digest.to_array();
    let hex_chars = b"0123456789abcdef";
    let mut arr = [0u8; 64];
    for i in 0..32 {
        arr[2 * i] = hex_chars[(bytes[i] >> 4) as usize];
        arr[2 * i + 1] = hex_chars[(bytes[i] & 0x0f) as usize];
    }
    let s = core::str::from_utf8(&arr).unwrap_or("");
    String::from_str(env, s)
}

fn register_bulletproof_circuit(
    client: &ZKPRegistryClient,
    env: &Env,
    admin: &Address,
    vk_hash: &BytesN<32>,
) {
    let cid = canonical_circuit_id(env, vk_hash);
    client.register_circuit(
        admin,
        &cid,
        &ZKPType::Bulletproof,
        &0u32,
        &0u32,
        &100u32,
        &128u32,
        vk_hash,
        &BytesN::from_array(env, &[0x88u8; 32]),
        &false,
    );
}

fn build_valid_range_proof_data(
    env: &Env,
    prover: &Address,
    vk_hash: &BytesN<32>,
    min_value: u64,
    max_value: u64,
    encrypted_value: &Bytes,
) -> Bytes {
    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_slice(env, b"UZIMA_RANGE_V1"));
    payload.append(&prover.to_xdr(env));
    payload.append(&Bytes::from_slice(env, &vk_hash.to_array()));
    payload.append(&Bytes::from_slice(env, &min_value.to_be_bytes()));
    payload.append(&Bytes::from_slice(env, &max_value.to_be_bytes()));
    payload.append(&Bytes::from_slice(
        env,
        &encrypted_value.len().to_be_bytes(),
    ));
    payload.append(encrypted_value);
    let commitment: BytesN<32> = env.crypto().sha256(&payload).into();
    let mut out = [0u8; 64];
    out[0] = 0x03;
    out[1..33].copy_from_slice(&commitment.to_array());
    Bytes::from_slice(env, &out)
}

fn setup(env: &Env) -> (ZKPRegistryClient<'_>, Address) {
    let contract_id = env.register_contract(None, zkp_registry::ZKPRegistry {});
    let client = ZKPRegistryClient::new(env, &contract_id);
    (client, contract_id)
}

// =============================================================================
// Property-Based Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// For any randomly-generated prover, vk_hash, min/max range, and
    /// encrypted_value, a range proof whose proof_data embeds the correct
    /// SHA-256 commitment MUST be accepted by the verifier.
    #[test]
    fn prop_valid_range_proof_accepted(
        vk_seed in any::<[u8; 32]>(),
        min_value in 0u64..1_000_000,
        max_value in 1u64..1_000_001,
        enc_val_seed in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        prop_assume!(min_value < max_value);
        let env = Env::default();
        env.mock_all_auths();
        let (client, _id) = setup(&env);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let prover = Address::generate(&env);
        let vk_hash = BytesN::from_array(&env, &vk_seed);
        let encrypted_value = Bytes::from_slice(&env, &enc_val_seed);
        let proof_data = build_valid_range_proof_data(
            &env, &prover, &vk_hash, min_value, max_value, &encrypted_value,
        );

        register_bulletproof_circuit(&client, &env, &admin, &vk_hash);

        let result = client.try_verify_range_proof(
            &zkp_registry::RangeProof {
                prover,
                encrypted_value,
                min_value,
                max_value,
                proof_data,
                vk_hash,
                verification_gas: 25000,
                created_at: 0,
            },
        );
        prop_assert!(
            matches!(result, Ok(Ok(true))),
            "valid range proof must be accepted, got {:?}",
            result,
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// For any valid range proof, flipping an arbitrary byte inside the
    /// verifier-checked portion of proof_data (version byte at index 0 or
    /// SHA-256 commitment at indices 1..32) MUST cause verification to fail.
    /// Padding bytes (33+) are not checked by the verifier, so they are
    /// excluded from the tamper range.
    #[test]
    fn prop_tampered_range_proof_rejected(
        vk_seed in any::<[u8; 32]>(),
        min_value in 0u64..10_000u64,
        delta in 1u64..10_000u64,
        enc_val_byte in any::<u8>(),
        // Only tamper bytes 0..33: version byte (0) + commitment (1..32)
        tamper_position in 0u32..33u32,
        tamper_bit in 0u8..8u8,
    ) {
        let max_value = min_value + delta;
        let env = Env::default();
        env.mock_all_auths();
        let (client, _id) = setup(&env);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let prover = Address::generate(&env);
        let vk_hash = BytesN::from_array(&env, &vk_seed);
        let encrypted_value = Bytes::from_slice(&env, &[enc_val_byte; 8]);
        let proof_data = build_valid_range_proof_data(
            &env, &prover, &vk_hash, min_value, max_value, &encrypted_value,
        );

        register_bulletproof_circuit(&client, &env, &admin, &vk_hash);

        let mut tampered = proof_data.clone();
        let old = tampered.get_unchecked(tamper_position);
        tampered.set(tamper_position, old ^ (1u8 << tamper_bit));

        let result = client.try_verify_range_proof(
            &zkp_registry::RangeProof {
                prover,
                encrypted_value,
                min_value,
                max_value,
                proof_data: tampered,
                vk_hash,
                verification_gas: 25000,
                created_at: 0,
            },
        );
        // Must reject: tampering version byte → InvalidProofFormat;
        // tampering commitment → InconsistentCommitment.
        prop_assert!(
            result.is_err(),
            "tampered range proof must be rejected (position={}, bit={}), got {:?}",
            tamper_position,
            tamper_bit,
            result,
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        max_shrink_iters: 0,
        .. ProptestConfig::default()
    })]

    /// For any valid range proof, changing `min_value` in the struct
    /// (so that the embedded commitment no longer matches) MUST cause
    /// verification to fail with `InconsistentCommitment`.
    #[test]
    fn prop_modified_range_values_rejected(
        vk_seed in any::<[u8; 32]>(),
        orig_min in 0u64..10_000u64,
        delta in 1u64..10_000u64,
        bad_min_shift in 1u64..1_000u64,
        enc_val_byte in any::<u8>(),
    ) {
        let orig_max = orig_min + delta;
        let bad_min = orig_min + bad_min_shift;
        prop_assume!(bad_min < orig_max);

        let env = Env::default();
        env.mock_all_auths();
        let (client, _id) = setup(&env);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let prover = Address::generate(&env);
        let vk_hash = BytesN::from_array(&env, &vk_seed);
        let encrypted_value = Bytes::from_slice(&env, &[enc_val_byte; 8]);

        let proof_data = build_valid_range_proof_data(
            &env, &prover, &vk_hash, orig_min, orig_max, &encrypted_value,
        );

        register_bulletproof_circuit(&client, &env, &admin, &vk_hash);

        let result = client.try_verify_range_proof(
            &zkp_registry::RangeProof {
                prover,
                encrypted_value,
                min_value: bad_min,
                max_value: orig_max,
                proof_data,
                vk_hash,
                verification_gas: 25000,
                created_at: 0,
            },
        );
        // Contract returns Err(Error::InconsistentCommitment) which the
        // Soroban SDK maps to Err(Ok(Error::InconsistentCommitment)).
        prop_assert_eq!(
            result,
            Err(Ok(Error::InconsistentCommitment)),
            "modified min_value must yield InconsistentCommitment",
        );
    }
}
