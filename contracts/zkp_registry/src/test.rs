use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};

// Construct a properly-formed `encrypted_expiration` blob for tests: 16 bytes
// that, after XOR with the default issuer salt, reveal the 8-byte domain tag
// followed by the 8-byte big-endian expiration timestamp (far in the future
// for all tests).
fn make_far_future_expiration(env: &Env, now: u64) -> Bytes {
    use crate::CRED_EXPIRATION_DOMAIN_TAG;
    use crate::DEFAULT_ISSUER_SALT;
    let future_ts = now.saturating_add(365 * 86_400);
    let mut plaintext = [0u8; 16];
    for i in 0..8 {
        plaintext[i] = CRED_EXPIRATION_DOMAIN_TAG[i];
    }
    let ts_bytes = future_ts.to_be_bytes();
    for i in 0..8 {
        plaintext[8 + i] = ts_bytes[i];
    }
    let mut ciphertext = [0u8; 16];
    for i in 0..16 {
        ciphertext[i] = plaintext[i] ^ DEFAULT_ISSUER_SALT[i % DEFAULT_ISSUER_SALT.len()];
    }
    Bytes::from_slice(env, &ciphertext)
}

// Construct a properly-formed Bulletproof-format `proof_data` blob for tests:
// bytes [0] = version 0x03, bytes [1..33] = commitment computed from public
// fields, bytes [33..64] zero-padded.
fn build_bulletproof_range_proof_data(
    env: &Env,
    prover: &Address,
    vk_hash: &BytesN<32>,
    min_value: u64,
    max_value: u64,
    encrypted_value: &Bytes,
) -> Bytes {
    use crate::PROOF_FORMAT_VERSION_BULLETPROOF;
    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_slice(env, b"UZIMA_RANGE_V1"));
    payload.append(&prover.to_xdr(env));
    let arr: [u8; 32] = vk_hash.to_array();
    payload.append(&Bytes::from_slice(env, &arr));
    payload.append(&Bytes::from_slice(env, &min_value.to_be_bytes()));
    payload.append(&Bytes::from_slice(env, &max_value.to_be_bytes()));
    payload.append(&Bytes::from_slice(
        env,
        &(encrypted_value.len() as u32).to_be_bytes(),
    ));
    payload.append(encrypted_value);
    let commitment: BytesN<32> = env.crypto().sha256(&payload).into();
    let mut out = [0u8; 64];
    out[0] = PROOF_FORMAT_VERSION_BULLETPROOF;
    let carr = commitment.to_array();
    for i in 0..32 {
        out[1 + i] = carr[i];
    }
    Bytes::from_slice(env, &out)
}

// Construct a properly-formed version-byte leading `proof_data` for a SNARK.
// `body_len` is accepted for API stability but the actual byte count is a
// compile-time constant — Soroban's `vec![x; n]` macro requires a literal
// repeat count, so we emit a fixed 64-byte buffer unconditionally (SNARK
// proofs must satisfy `type_specific_min = 64` per `verify_proof_format`).
fn build_snark_proof_data(env: &Env, body_len: usize) -> Bytes {
    use crate::PROOF_FORMAT_VERSION_SNARK;
    let _ = body_len;
    let arr = [PROOF_FORMAT_VERSION_SNARK; 64];
    Bytes::from_slice(env, &arr)
}

#[test]
fn test_zkp_registry_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let result = client.try_get_circuit_params(&String::from_str(&env, "test_circuit"));
    assert!(matches!(result, Err(Ok(Error::CircuitNotFound))));
}

#[test]
fn test_circuit_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "medical_authenticity");
    let vk_hash = BytesN::from_array(&env, &[1u8; 32]);
    let pk_hash = BytesN::from_array(&env, &[2u8; 32]);

    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &5u32,
        &10u32,
        &1000u32,
        &128u32,
        &vk_hash,
        &pk_hash,
        &true,
    );

    let params = client.get_circuit_params(&circuit_id);
    assert!(params.circuit_id == circuit_id);
    assert!(params.circuit_type == ZKPType::SNARK);
    assert_eq!(params.num_public_inputs, 5);
    assert_eq!(params.num_private_inputs, 10);
    assert_eq!(params.num_constraints, 1000);
    assert_eq!(params.security_param, 128);
    assert_eq!(params.vk_hash, vk_hash);
    assert_eq!(params.pk_hash, pk_hash);
    assert!(params.trusted_setup);
}

#[test]
fn test_zkp_submission_and_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "test_circuit");
    let vk_hash = BytesN::from_array(&env, &[1u8; 32]);
    let pk_hash = BytesN::from_array(&env, &[2u8; 32]);

    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &2u32,
        &3u32,
        &100u32,
        &128u32,
        &vk_hash,
        &pk_hash,
        &false,
    );

    let submitter = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[3u8; 32]);
    let public_inputs = vec![
        &env,
        Bytes::from_slice(&env, b"input1"),
        Bytes::from_slice(&env, b"input2"),
    ];
    // Use a properly-formed SNARK-format proof payload (version byte = 0x01)
    // so the new strict format-integrity check accepts it.
    let proof_data = build_snark_proof_data(&env, 64);

    client.submit_zkp(
        &submitter,
        &proof_id,
        &ZKPType::SNARK,
        &ZKPHashFunction::Poseidon,
        &circuit_id,
        &public_inputs,
        &proof_data,
        &vk_hash,
        &50000u64,
    );

    let result = client.get_verification_result(&proof_id);
    assert!(result.is_valid);
    assert!(result.proof_id == proof_id);
    assert!(result.verifier == submitter);
    assert_eq!(result.gas_used, 50000);
}

#[test]
fn test_medical_record_authenticity_proof() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let patient = Address::generate(&env);
    let record_id = 12345u64;
    let metadata_hash = BytesN::from_array(&env, &[4u8; 32]);

    let authenticity_proof = ZKProof {
        proof_type: ZKPType::SNARK,
        hash_function: ZKPHashFunction::Poseidon,
        circuit_id: String::from_str(&env, "record_authenticity"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"record_hash")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[5u8; 32]),
        verification_gas: 45000u64,
        created_at: env.ledger().timestamp(),
    };

    let access_proof = ZKProof {
        proof_type: ZKPType::Bulletproof,
        hash_function: ZKPHashFunction::MiMC,
        circuit_id: String::from_str(&env, "access_control"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"access_rights")],
        proof_data: build_bulletproof_for_zkproof(&env, &patient, &access_proof.vk_hash, &public_inputs_for_bulletproof(&env)),
        vk_hash: BytesN::from_array(&env, &[6u8; 32]),
        verification_gas: 30000u64,
        created_at: env.ledger().timestamp(),
    };

    client.register_circuit(
        &admin,
        &authenticity_proof.circuit_id,
        &ZKPType::SNARK,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &authenticity_proof.vk_hash,
        &BytesN::from_array(&env, &[3u8; 32]),
        &false,
    );
    client.register_circuit(
        &admin,
        &access_proof.circuit_id,
        &ZKPType::Bulletproof,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &access_proof.vk_hash,
        &BytesN::from_array(&env, &[4u8; 32]),
        &false,
    );

    client.create_medical_record_proof(
        &patient,
        &record_id,
        &authenticity_proof,
        &access_proof,
        &metadata_hash,
    );

    let proof = client.get_medical_record_proof(&patient, &record_id);
    assert!(proof.patient_id == patient);
    assert_eq!(proof.record_id, record_id);
    assert_eq!(proof.metadata_hash, metadata_hash);
    assert!(proof.is_verified);
}

#[test]
fn test_range_proof_age_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let prover = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[7u8; 32]);
    let encrypted_value = Bytes::from_slice(&env, b"encrypted_age");
    let vk_hash = BytesN::from_array(&env, &[8u8; 32]);
    let proof_data = build_bulletproof_range_proof_data(
        &env,
        &prover,
        &vk_hash,
        18,
        65,
        &encrypted_value,
    );

    client.create_range_proof(
        &prover,
        &proof_id,
        &encrypted_value,
        &18u64,
        &65u64,
        &proof_data,
        &vk_hash,
        &25000u64,
    );

    let range_proof = client.get_range_proof(&proof_id);
    assert!(range_proof.prover == prover);
    assert_eq!(range_proof.min_value, 18);
    assert_eq!(range_proof.max_value, 65);
    assert_eq!(range_proof.verification_gas, 25000);
}

#[test]
fn test_credential_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let holder = Address::generate(&env);
    let credential_type = String::from_str(&env, "medical_license");
    let issuer = Address::generate(&env);
    let encrypted_expiration = make_far_future_expiration(&env, env.ledger().timestamp());

    let validity_proof = ZKProof {
        proof_type: ZKPType::SNARK,
        hash_function: ZKPHashFunction::SHA256,
        circuit_id: String::from_str(&env, "credential_validity"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"credential_id")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[9u8; 32]),
        verification_gas: 60000u64,
        created_at: env.ledger().timestamp(),
    };

    let attribute_proof = ZKProof {
        proof_type: ZKPType::Bulletproof,
        hash_function: ZKPHashFunction::Poseidon,
        circuit_id: String::from_str(&env, "credential_attributes"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"attributes_commit")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[10u8; 32]),
        verification_gas: 35000u64,
        created_at: env.ledger().timestamp(),
    };

    client.register_circuit(
        &admin,
        &validity_proof.circuit_id,
        &ZKPType::SNARK,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &validity_proof.vk_hash,
        &BytesN::from_array(&env, &[101u8; 32]),
        &false,
    );
    client.register_circuit(
        &admin,
        &attribute_proof.circuit_id,
        &ZKPType::Bulletproof,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &attribute_proof.vk_hash,
        &BytesN::from_array(&env, &[102u8; 32]),
        &false,
    );

    client.create_credential_proof(
        &holder,
        &credential_type,
        &issuer,
        &validity_proof,
        &attribute_proof,
        &encrypted_expiration,
    );

    let proof = client.get_credential_proof(&holder, &credential_type);
    assert!(proof.holder == holder);
    assert!(proof.credential_type == credential_type);
    assert!(proof.issuer == issuer);
    assert!(proof.is_verified);
}

#[test]
fn test_recursive_zkp() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let base_prover = Address::generate(&env);
    let base_proof_id = BytesN::from_array(&env, &[11u8; 32]);
    let circuit_id = String::from_str(&env, "base_circuit");
    let vk_hash = BytesN::from_array(&env, &[12u8; 32]);

    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &2u32,
        &3u32,
        &100u32,
        &128u32,
        &vk_hash,
        &BytesN::from_array(&env, &[13u8; 32]),
        &false,
    );

    let base_inputs = vec![&env, Bytes::from_slice(&env, b"base_input")];
    let base_proof_data = build_snark_proof_data(&env, 64);
    client.submit_zkp(
        &base_prover,
        &base_proof_id,
        &ZKPType::SNARK,
        &ZKPHashFunction::Poseidon,
        &circuit_id,
        &base_inputs,
        &base_proof_data,
        &vk_hash,
        &40000u64,
    );

    let composer = Address::generate(&env);
    let recursive_proof = ZKProof {
        proof_type: ZKPType::Recursive,
        hash_function: ZKPHashFunction::Rescue,
        circuit_id: String::from_str(&env, "recursive_circuit"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"recursive_input")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[14u8; 32]),
        verification_gas: 85000u64,
        created_at: env.ledger().timestamp(),
    };

    // Compute the aggregated VK hash that the on-chain verifier expects for
    // a recursive proof composed over the registered base.
    let aggregated_vk = compute_aggregated_vk_for_test(
        &env,
        &vk_hash,
        &recursive_proof.vk_hash,
        &base_proof_data,
        3,
    );

    client.create_recursive_proof(
        &composer,
        &base_proof_id,
        &recursive_proof,
        &aggregated_vk,
        &3u32,
        &95000u64,
    );

    let gas_stats = client.get_gas_stats(&composer);
    assert!(gas_stats >= 95000);
}

#[test]
fn test_gas_efficiency_limits() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "gas_test_circuit");
    let vk_hash = BytesN::from_array(&env, &[15u8; 32]);

    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &1u32,
        &1u32,
        &50u32,
        &128u32,
        &vk_hash,
        &BytesN::from_array(&env, &[16u8; 32]),
        &false,
    );

    let submitter = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[17u8; 32]);
    let inputs = vec![&env, Bytes::from_slice(&env, b"input")];
    let proof_data = Bytes::from_slice(&env, b"proof");

    // Exceeds gas limit — should return contract error
    let result = client.try_submit_zkp(
        &submitter,
        &proof_id,
        &ZKPType::SNARK,
        &ZKPHashFunction::Poseidon,
        &circuit_id,
        &inputs,
        &proof_data,
        &vk_hash,
        &150000u64,
    );

    assert_eq!(result, Err(Ok(Error::GasLimitExceeded)));
}

#[test]
fn test_zkp_hash_function_performance() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "hash_perf_test");
    let vk_hash = BytesN::from_array(&env, &[18u8; 32]);

    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &2u32,
        &2u32,
        &100u32,
        &128u32,
        &vk_hash,
        &BytesN::from_array(&env, &[19u8; 32]),
        &false,
    );

    let submitter = Address::generate(&env);

    let hash_functions = [
        ZKPHashFunction::Poseidon,
        ZKPHashFunction::MiMC,
        ZKPHashFunction::SHA256,
        ZKPHashFunction::Rescue,
    ];

    let expected_gas: [u64; 4] = [50000, 45000, 80000, 55000];

    for (i, hash_function) in hash_functions.iter().enumerate() {
        let proof_id = BytesN::from_array(&env, &[(20 + i as u8); 32]);
        let inputs = vec![&env, Bytes::from_slice(&env, b"input")];
        let proof_data = build_snark_proof_data(&env, 64);

        client.submit_zkp(
            &submitter,
            &proof_id,
            &ZKPType::SNARK,
            hash_function,
            &circuit_id,
            &inputs,
            &proof_data,
            &vk_hash,
            &expected_gas[i],
        );

        let result = client.get_verification_result(&proof_id);
        assert!(result.is_valid);
        assert_eq!(result.gas_used, expected_gas[i]);
    }
}

#[test]
fn test_security_parameter_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "security_test");
    let vk_hash = BytesN::from_array(&env, &[21u8; 32]);

    // Too many public inputs — should fail
    let result = client.try_register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &60u32, // > 50 limit
        &10u32,
        &100u32,
        &128u32,
        &vk_hash,
        &BytesN::from_array(&env, &[22u8; 32]),
        &false,
    );

    assert_eq!(result, Err(Ok(Error::InvalidCircuit)));

    // Valid parameters — should succeed without panic
    let valid_circuit_id = String::from_str(&env, "valid_circuit");
    client.register_circuit(
        &admin,
        &valid_circuit_id,
        &ZKPType::SNARK,
        &25u32,
        &50u32,
        &5000u32,
        &256u32,
        &vk_hash,
        &BytesN::from_array(&env, &[23u8; 32]),
        &false,
    );
}

fn setup(env: &Env) -> (ZKPRegistryClient<'_>, Address) {
    let contract_id = env.register_contract(None, ZKPRegistry {});
    let client = ZKPRegistryClient::new(env, &contract_id);
    (client, contract_id)
}

// Helper used by `test_medical_record_authenticity_proof` when constructing
// a properly-formatted Bulletproof-format proof payload for a `ZKProof`
// (where the public inputs are an already-formed Vec<Bytes>).
fn build_bulletproof_for_zkproof(
    env: &Env,
    prover: &Address,
    vk_hash: &BytesN<32>,
    _public_inputs: &soroban_sdk::Vec<Bytes>,
) -> Bytes {
    // For test purposes we reuse the range-formatting helper but with
    // min=max=0 so the recomputed commitment depends only on the
    // prover/vk_hash, which is what the bulletproof-format check needs.
    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_slice(env, b"UZIMA_RANGE_V1"));
    payload.append(&prover.to_xdr(env));
    let arr: [u8; 32] = vk_hash.to_array();
    payload.append(&Bytes::from_slice(env, &arr));
    payload.append(&Bytes::from_slice(env, &[0u8; 8]));
    payload.append(&Bytes::from_slice(env, &[0u8; 8]));
    payload.append(&Bytes::from_slice(env, &[0u32.to_be_bytes().to_vec().as_slice()][0]));
    let commitment: BytesN<32> = env.crypto().sha256(&payload).into();
    let mut out = [0u8; 64];
    out[0] = crate::PROOF_FORMAT_VERSION_BULLETPROOF;
    let carr = commitment.to_array();
    for i in 0..32 {
        out[1 + i] = carr[i];
    }
    Bytes::from_slice(env, &out)
}

// Empty Vec<Bytes> helper; not actually used directly because we build
// commitment from prover/vk_hash only.
fn public_inputs_for_bulletproof(env: &Env) -> soroban_sdk::Vec<Bytes> {
    let v: soroban_sdk::Vec<Bytes> = vec![env];
    v
}

// Test helper: compute the aggregated VK hash that
// `compute_aggregated_vk_hash` would compute given a base VK, recursive VK,
// base proof payload, and composition depth.
fn compute_aggregated_vk_for_test(
    env: &Env,
    base_vk: &BytesN<32>,
    recursive_vk: &BytesN<32>,
    base_proof_data: &Bytes,
    composition_depth: u32,
) -> BytesN<32> {
    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_slice(env, b"UZIMA_AGG_VK_V1"));
    let arr1: [u8; 32] = base_vk.to_array();
    payload.append(&Bytes::from_slice(env, &arr1));
    let arr2: [u8; 32] = recursive_vk.to_array();
    payload.append(&Bytes::from_slice(env, &arr2));
    payload.append(base_proof_data);
    payload.append(&Bytes::from_slice(
        env,
        &composition_depth.to_be_bytes(),
    ));
    env.crypto().sha256(&payload).into()
}

// =============================================================================
// New tests covering the strict cryptographic verification behavior
// =============================================================================

#[test]
fn test_submit_zkp_rejects_wrong_vk_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "vk_test");
    let registered_vk = BytesN::from_array(&env, &[10u8; 32]);
    let _rogue_vk = BytesN::from_array(&env, &[99u8; 32]);
    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &2u32,
        &2u32,
        &100u32,
        &128u32,
        &registered_vk,
        &BytesN::from_array(&env, &[11u8; 32]),
        &false,
    );

    let submitter = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[12u8; 32]);
    let inputs = vec![&env, Bytes::from_slice(&env, b"a"), Bytes::from_slice(&env, b"b")];
    let proof_data = build_snark_proof_data(&env, 64);
    let rogue_vk = BytesN::from_array(&env, &[99u8; 32]);

    let result = client.try_submit_zkp(
        &submitter,
        &proof_id,
        &ZKPType::SNARK,
        &ZKPHashFunction::Poseidon,
        &circuit_id,
        &inputs,
        &proof_data,
        &rogue_vk,
        &50000u64,
    );
    assert_eq!(result, Err(Ok(Error::VkMismatch)));
}

#[test]
fn test_submit_zkp_rejects_wrong_public_input_count() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "pi_count_test");
    let vk_hash = BytesN::from_array(&env, &[20u8; 32]);
    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &3u32,
        &2u32,
        &100u32,
        &128u32,
        &vk_hash,
        &BytesN::from_array(&env, &[21u8; 32]),
        &false,
    );

    let submitter = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[22u8; 32]);
    // Only 2 public inputs supplied; circuit expects 3.
    let inputs = vec![&env, Bytes::from_slice(&env, b"a"), Bytes::from_slice(&env, b"b")];
    let proof_data = build_snark_proof_data(&env, 64);

    let result = client.try_submit_zkp(
        &submitter,
        &proof_id,
        &ZKPType::SNARK,
        &ZKPHashFunction::Poseidon,
        &circuit_id,
        &inputs,
        &proof_data,
        &vk_hash,
        &50000u64,
    );
    assert_eq!(result, Err(Ok(Error::InconsistentPublicInputCount)));
}

#[test]
fn test_submit_zkp_rejects_wrong_version_byte() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let circuit_id = String::from_str(&env, "vf_test");
    let vk_hash = BytesN::from_array(&env, &[30u8; 32]);
    client.register_circuit(
        &admin,
        &circuit_id,
        &ZKPType::SNARK,
        &1u32,
        &2u32,
        &100u32,
        &128u32,
        &vk_hash,
        &BytesN::from_array(&env, &[31u8; 32]),
        &false,
    );

    let submitter = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[32u8; 32]);
    let inputs = vec![&env, Bytes::from_slice(&env, b"a")];
    // Tampered version byte.
    let mut bad_proof_data = Bytes::from_slice(&env, &[0u8; 64]);
    bad_proof_data.set(0, 0x7F);
    let result = client.try_submit_zkp(
        &submitter,
        &proof_id,
        &ZKPType::SNARK,
        &ZKPHashFunction::Poseidon,
        &circuit_id,
        &inputs,
        &bad_proof_data,
        &vk_hash,
        &50000u64,
    );
    assert_eq!(result, Err(Ok(Error::InvalidProofFormat)));
}

#[test]
fn test_range_proof_rejects_tampered_commitment() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let prover = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[40u8; 32]);
    let encrypted_value = Bytes::from_slice(&env, b"opaque_blob");
    let vk_hash = BytesN::from_array(&env, &[41u8; 32]);
    let good_proof = build_bulletproof_range_proof_data(
        &env,
        &prover,
        &vk_hash,
        18,
        65,
        &encrypted_value,
    );

    // Tamper: flip one byte inside proof_data so the embedded commitment
    // no longer matches the recomputed one.
    let mut bad_proof = good_proof.clone();
    bad_proof.set(2, 0xFF);

    let result = client.try_create_range_proof(
        &prover,
        &proof_id,
        &encrypted_value,
        &18u64,
        &65u64,
        &bad_proof,
        &vk_hash,
        &25000u64,
    );
    assert_eq!(result, Err(Ok(Error::InconsistentCommitment)));
}

#[test]
fn test_range_proof_rejects_unregistered_vk() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let prover = Address::generate(&env);
    let proof_id = BytesN::from_array(&env, &[45u8; 32]);
    let encrypted_value = Bytes::from_slice(&env, b"x");
    let vk_hash = BytesN::from_array(&env, &[46u8; 32]);
    let proof_data = build_bulletproof_range_proof_data(
        &env,
        &prover,
        &vk_hash,
        1,
        100,
        &encrypted_value,
    );

    // Don't register any circuit first.
    let result = client.try_create_range_proof(
        &prover,
        &proof_id,
        &encrypted_value,
        &1u64,
        &100u64,
        &proof_data,
        &vk_hash,
        &25000u64,
    );
    assert_eq!(result, Err(Ok(Error::CircuitNotFound)));
}

#[test]
fn test_credential_rejects_expired_expiration() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let holder = Address::generate(&env);
    let credential_type = String::from_str(&env, "expired_license");
    let issuer = Address::generate(&env);

    // Build a credential that decrypts to a timestamp in the past.
    use crate::CRED_EXPIRATION_DOMAIN_TAG;
    use crate::DEFAULT_ISSUER_SALT;
    let past_ts: u64 = 1;
    let mut plaintext = [0u8; 16];
    for i in 0..8 {
        plaintext[i] = CRED_EXPIRATION_DOMAIN_TAG[i];
    }
    let ts_bytes = past_ts.to_be_bytes();
    for i in 0..8 {
        plaintext[8 + i] = ts_bytes[i];
    }
    let mut ciphertext = [0u8; 16];
    for i in 0..16 {
        ciphertext[i] = plaintext[i] ^ DEFAULT_ISSUER_SALT[i % DEFAULT_ISSUER_SALT.len()];
    }
    let expired_ciphertext = Bytes::from_slice(&env, &ciphertext);

    let validity_proof = ZKProof {
        proof_type: ZKPType::SNARK,
        hash_function: ZKPHashFunction::SHA256,
        circuit_id: String::from_str(&env, "cred_v"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"id")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[50u8; 32]),
        verification_gas: 60000u64,
        created_at: env.ledger().timestamp(),
    };
    let attribute_proof = ZKProof {
        proof_type: ZKPType::Bulletproof,
        hash_function: ZKPHashFunction::Poseidon,
        circuit_id: String::from_str(&env, "cred_a"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"id2")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[51u8; 32]),
        verification_gas: 35000u64,
        created_at: env.ledger().timestamp(),
    };

    client.register_circuit(
        &admin,
        &validity_proof.circuit_id,
        &ZKPType::SNARK,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &validity_proof.vk_hash,
        &BytesN::from_array(&env, &[52u8; 32]),
        &false,
    );
    client.register_circuit(
        &admin,
        &attribute_proof.circuit_id,
        &ZKPType::Bulletproof,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &attribute_proof.vk_hash,
        &BytesN::from_array(&env, &[53u8; 32]),
        &false,
    );

    env.ledger().with_mut(|li| li.timestamp = 1000);
    let result = client.try_create_credential_proof(
        &holder,
        &credential_type,
        &issuer,
        &validity_proof,
        &attribute_proof,
        &expired_ciphertext,
    );
    assert_eq!(result, Err(Ok(Error::CredentialExpired)));
}

#[test]
fn test_credential_rejects_tampered_ciphertext() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let holder = Address::generate(&env);
    let credential_type = String::from_str(&env, "tampered_license");
    let issuer = Address::generate(&env);
    // 16-byte blob that does NOT decode to the domain tag.
    let bad_ciphertext = Bytes::from_slice(&env, &[0u8; 16]);

    let validity_proof = ZKProof {
        proof_type: ZKPType::SNARK,
        hash_function: ZKPHashFunction::SHA256,
        circuit_id: String::from_str(&env, "cred_v2"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"id")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[60u8; 32]),
        verification_gas: 60000u64,
        created_at: env.ledger().timestamp(),
    };
    let attribute_proof = ZKProof {
        proof_type: ZKPType::Bulletproof,
        hash_function: ZKPHashFunction::Poseidon,
        circuit_id: String::from_str(&env, "cred_a2"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"id2")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[61u8; 32]),
        verification_gas: 35000u64,
        created_at: env.ledger().timestamp(),
    };
    client.register_circuit(
        &admin,
        &validity_proof.circuit_id,
        &ZKPType::SNARK,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &validity_proof.vk_hash,
        &BytesN::from_array(&env, &[62u8; 32]),
        &false,
    );
    client.register_circuit(
        &admin,
        &attribute_proof.circuit_id,
        &ZKPType::Bulletproof,
        &1u32,
        &2u32,
        &200u32,
        &128u32,
        &attribute_proof.vk_hash,
        &BytesN::from_array(&env, &[63u8; 32]),
        &false,
    );

    let result = client.try_create_credential_proof(
        &holder,
        &credential_type,
        &issuer,
        &validity_proof,
        &attribute_proof,
        &bad_ciphertext,
    );
    assert_eq!(result, Err(Ok(Error::InvalidExpirationCiphertext)));
}

#[test]
fn test_recursive_proof_rejects_missing_base() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let composer = Address::generate(&env);
    let bogus_base_id = BytesN::from_array(&env, &[0xAA; 32]);
    let recursive_proof = ZKProof {
        proof_type: ZKPType::Recursive,
        hash_function: ZKPHashFunction::Rescue,
        circuit_id: String::from_str(&env, "rec_missing"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"x")],
        proof_data: build_snark_proof_data(&env, 64),
        vk_hash: BytesN::from_array(&env, &[70u8; 32]),
        verification_gas: 85000u64,
        created_at: env.ledger().timestamp(),
    };
    let aggregated_vk = BytesN::from_array(&env, &[71u8; 32]);

    let result = client.try_create_recursive_proof(
        &composer,
        &bogus_base_id,
        &recursive_proof,
        &aggregated_vk,
        &3u32,
        &95000u64,
    );
    assert_eq!(result, Err(Ok(Error::BaseProofMissing)));
}
