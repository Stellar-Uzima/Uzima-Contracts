use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String};

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
    let proof_data = Bytes::from_slice(&env, b"zkp_proof_data");

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
        proof_data: Bytes::from_slice(&env, b"authenticity_proof"),
        vk_hash: BytesN::from_array(&env, &[5u8; 32]),
        verification_gas: 45000u64,
        created_at: env.ledger().timestamp(),
    };

    let access_proof = ZKProof {
        proof_type: ZKPType::Bulletproof,
        hash_function: ZKPHashFunction::MiMC,
        circuit_id: String::from_str(&env, "access_control"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"access_rights")],
        proof_data: Bytes::from_slice(&env, b"access_proof"),
        vk_hash: BytesN::from_array(&env, &[6u8; 32]),
        verification_gas: 30000u64,
        created_at: env.ledger().timestamp(),
    };

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
    let proof_data = Bytes::from_slice(&env, b"range_proof_data");
    let vk_hash = BytesN::from_array(&env, &[8u8; 32]);

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
    let encrypted_expiration = Bytes::from_slice(&env, b"encrypted_timestamp");

    let validity_proof = ZKProof {
        proof_type: ZKPType::SNARK,
        hash_function: ZKPHashFunction::SHA256,
        circuit_id: String::from_str(&env, "credential_validity"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"credential_id")],
        proof_data: Bytes::from_slice(&env, b"validity_proof"),
        vk_hash: BytesN::from_array(&env, &[9u8; 32]),
        verification_gas: 60000u64,
        created_at: env.ledger().timestamp(),
    };

    let attribute_proof = ZKProof {
        proof_type: ZKPType::Bulletproof,
        hash_function: ZKPHashFunction::Poseidon,
        circuit_id: String::from_str(&env, "credential_attributes"),
        public_inputs: vec![&env, Bytes::from_slice(&env, b"attributes_commit")],
        proof_data: Bytes::from_slice(&env, b"attribute_proof"),
        vk_hash: BytesN::from_array(&env, &[10u8; 32]),
        verification_gas: 35000u64,
        created_at: env.ledger().timestamp(),
    };

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
    let base_proof_data = Bytes::from_slice(&env, b"base_proof");
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
        proof_data: Bytes::from_slice(&env, b"recursive_proof_data"),
        vk_hash: BytesN::from_array(&env, &[14u8; 32]),
        verification_gas: 85000u64,
        created_at: env.ledger().timestamp(),
    };

    let aggregated_vk = Bytes::from_slice(&env, b"aggregated_vk");

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
        let proof_data = Bytes::from_slice(&env, b"proof");

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
