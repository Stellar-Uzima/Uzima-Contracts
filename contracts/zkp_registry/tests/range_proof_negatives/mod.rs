//! Negative test corpus for range proof verification (TD-003).
//!
//! Each test vector describes an invalid `RangeProof` that must be rejected
//! by `verify_range_proof` / `create_range_proof`.  Tests are organised by
//! the specific failure mode they exercise.

use soroban_sdk::testutils::Address as _;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, Bytes, BytesN, Env};
use zkp_registry::{RangeProof, ZKPRegistryClient};

/// Build a valid proof_data blob (used as the starting point for tampering
/// in some test vectors).
pub fn valid_proof_data(
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

/// Register the canonical Bulletproof circuit for the given vk_hash.
pub fn register_bulletproof_circuit(
    client: &ZKPRegistryClient,
    env: &Env,
    admin: &Address,
    vk_hash: &BytesN<32>,
) {
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
    let cid = soroban_sdk::String::from_str(env, s);
    client.register_circuit(
        admin,
        &cid,
        &zkp_registry::ZKPType::Bulletproof,
        &0u32,
        &0u32,
        &100u32,
        &128u32,
        vk_hash,
        &BytesN::from_array(env, &[0x88u8; 32]),
        &false,
    );
}

/// Setup helper: returns (client, admin).
pub fn setup(env: &Env) -> (ZKPRegistryClient<'_>, Address) {
    let contract_id = env.register_contract(None, zkp_registry::ZKPRegistry {});
    let client = ZKPRegistryClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

// =============================================================================
// Test Vector 1: Wrong version byte in proof_data (0x01 instead of 0x03)
// =============================================================================

/// Build a RangeProof whose proof_data starts with version 0x01 (SNARK) but
/// all other fields are valid. Must be rejected with `InvalidProofFormat`.
pub fn case_wrong_version_byte(env: &Env) -> (RangeProof, BytesN<32>) {
    let prover = Address::generate(env);
    let vk_hash = BytesN::from_array(env, &[0xB0u8; 32]);
    let mut data = valid_proof_data(
        env,
        &prover,
        &vk_hash,
        18,
        65,
        &Bytes::from_slice(env, b"age"),
    );
    data.set(0, 0x01); // SNARK version, not Bulletproof
    let proof_id = BytesN::from_array(env, &[0xC0u8; 32]);
    let proof = RangeProof {
        prover,
        encrypted_value: Bytes::from_slice(env, b"age"),
        min_value: 18,
        max_value: 65,
        proof_data: data,
        vk_hash,
        verification_gas: 25000,
        created_at: 0,
    };
    (proof, proof_id)
}

// =============================================================================
// Test Vector 2: Tampered commitment (byte 2 flipped)
// =============================================================================

/// Build a RangeProof whose proof_data has a flipped byte in the SHA-256
/// commitment portion (byte index 2). Must be rejected with
/// `InconsistentCommitment`.
pub fn case_tampered_commitment(env: &Env) -> (RangeProof, BytesN<32>) {
    let prover = Address::generate(env);
    let vk_hash = BytesN::from_array(env, &[0xB1u8; 32]);
    let mut data = valid_proof_data(
        env,
        &prover,
        &vk_hash,
        21,
        99,
        &Bytes::from_slice(env, b"enc"),
    );
    let old = data.get_unchecked(2);
    data.set(2, old ^ 0xFF);
    let proof_id = BytesN::from_array(env, &[0xC1u8; 32]);
    let proof = RangeProof {
        prover,
        encrypted_value: Bytes::from_slice(env, b"enc"),
        min_value: 21,
        max_value: 99,
        proof_data: data,
        vk_hash,
        verification_gas: 25000,
        created_at: 0,
    };
    (proof, proof_id)
}

// =============================================================================
// Test Vector 3: Empty proof_data
// =============================================================================

/// Build a RangeProof with an empty proof_data. Must be rejected with
/// `MalformedProof`.
pub fn case_empty_proof_data(env: &Env) -> (RangeProof, BytesN<32>) {
    let prover = Address::generate(env);
    let vk_hash = BytesN::from_array(env, &[0xB2u8; 32]);
    let proof_id = BytesN::from_array(env, &[0xC2u8; 32]);
    let proof = RangeProof {
        prover,
        encrypted_value: Bytes::from_slice(env, b"age"),
        min_value: 18,
        max_value: 65,
        proof_data: Bytes::new(env),
        vk_hash,
        verification_gas: 25000,
        created_at: 0,
    };
    (proof, proof_id)
}

// =============================================================================
// Test Vector 4: Proof_data too short (< PROOF_MIN_BYTES = 32)
// =============================================================================

/// Build a RangeProof whose proof_data is only 16 bytes. Must be rejected
/// with `MalformedProof`.
pub fn case_proof_data_too_short(env: &Env) -> (RangeProof, BytesN<32>) {
    let prover = Address::generate(env);
    let vk_hash = BytesN::from_array(env, &[0xB3u8; 32]);
    let proof_id = BytesN::from_array(env, &[0xC3u8; 32]);
    let proof = RangeProof {
        prover,
        encrypted_value: Bytes::from_slice(env, b"age"),
        min_value: 18,
        max_value: 65,
        proof_data: Bytes::from_slice(env, &[0x03u8; 16]),
        vk_hash,
        verification_gas: 25000,
        created_at: 0,
    };
    (proof, proof_id)
}

// =============================================================================
// Test Vector 5: Unregistered VK hash
// =============================================================================

/// Build a valid RangeProof against a VK hash that has NOT been registered
/// as a Bulletproof circuit. Must be rejected with `CircuitNotFound`.
pub fn case_unregistered_vk(env: &Env) -> (RangeProof, BytesN<32>) {
    let prover = Address::generate(env);
    let vk_hash = BytesN::from_array(env, &[0xB4u8; 32]);
    let proof_data = valid_proof_data(
        env,
        &prover,
        &vk_hash,
        30,
        50,
        &Bytes::from_slice(env, b"unregistered"),
    );
    let proof_id = BytesN::from_array(env, &[0xC4u8; 32]);
    let proof = RangeProof {
        prover,
        encrypted_value: Bytes::from_slice(env, b"unregistered"),
        min_value: 30,
        max_value: 50,
        proof_data,
        vk_hash,
        verification_gas: 25000,
        created_at: 0,
    };
    (proof, proof_id)
}

// =============================================================================
// Test Vector 6: Encrypted_value differs from proof_data commitment
// =============================================================================

/// Build a RangeProof where the encrypted_value submitted in the struct
/// differs from the value used to compute the embedded commitment in
/// proof_data. Must be rejected with `InconsistentCommitment`.
pub fn case_mismatched_encrypted_value(env: &Env) -> (RangeProof, BytesN<32>) {
    let prover = Address::generate(env);
    let vk_hash = BytesN::from_array(env, &[0xB5u8; 32]);
    // Build commitment over "original_value"
    let proof_data = valid_proof_data(
        env,
        &prover,
        &vk_hash,
        25,
        75,
        &Bytes::from_slice(env, b"original_value"),
    );
    let proof_id = BytesN::from_array(env, &[0xC5u8; 32]);
    // Submit with a different encrypted_value
    let proof = RangeProof {
        prover,
        encrypted_value: Bytes::from_slice(env, b"tampered_value"),
        min_value: 25,
        max_value: 75,
        proof_data,
        vk_hash,
        verification_gas: 25000,
        created_at: 0,
    };
    (proof, proof_id)
}

// =============================================================================
// Test Vector 7: min_value >= max_value (invalid range)
// =============================================================================

/// Build a RangeProof where min_value >= max_value. Must be rejected with
/// `InvalidRange` at the public-function layer (before the internal
/// verifier runs).
pub fn case_invalid_range(env: &Env) -> (RangeProof, BytesN<32>) {
    let prover = Address::generate(env);
    let vk_hash = BytesN::from_array(env, &[0xB6u8; 32]);
    // The proof_data is technically valid (min=100, max=50 embedded),
    // but the struct values are invalid.
    let proof_data = valid_proof_data(
        env,
        &prover,
        &vk_hash,
        100,
        50,
        &Bytes::from_slice(env, b"range"),
    );
    let proof_id = BytesN::from_array(env, &[0xC6u8; 32]);
    let proof = RangeProof {
        prover,
        encrypted_value: Bytes::from_slice(env, b"range"),
        min_value: 100,
        max_value: 50,
        proof_data,
        vk_hash,
        verification_gas: 25000,
        created_at: 0,
    };
    (proof, proof_id)
}
