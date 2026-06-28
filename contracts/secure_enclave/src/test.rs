use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

#[test]
fn test_registration_and_attestation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, SecureEnclaveContract);
    let client = SecureEnclaveContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let caller = Address::generate(&env);
    let node_id = BytesN::from_array(&env, &[1; 32]);
    let public_key = BytesN::from_array(&env, &[2; 32]);
    let quote = Bytes::from_array(&env, &[0; 64]);

    client.register_enclave(
        &caller,
        &node_id,
        &CloudProvider::AWSNitro,
        &quote,
        &public_key,
    );

    // Verify
    client.verify_attestation(&admin, &node_id, &true);
}

#[test]
fn test_submit_and_complete_task() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, SecureEnclaveContract);
    let client = SecureEnclaveContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    // Register node
    let caller = Address::generate(&env);
    let node_id = BytesN::from_array(&env, &[1; 32]);
    let public_key = BytesN::from_array(&env, &[2; 32]);
    let quote = Bytes::from_array(&env, &[0; 64]);

    client.register_enclave(
        &caller,
        &node_id,
        &CloudProvider::IntelSGX,
        &quote,
        &public_key,
    );
    client.verify_attestation(&admin, &node_id, &true);

    // Submit task
    let submitter = Address::generate(&env);
    let task_id = BytesN::from_array(&env, &[3; 32]);
    let payload_hash = BytesN::from_array(&env, &[4; 32]);

    client.submit_task(&submitter, &task_id, &payload_hash, &false);
    client.assign_task(&admin, &task_id, &node_id);

    // Complete task
    let result_bytes = Bytes::from_array(&env, &[9, 9, 9]);
    let node_auth = Address::generate(&env); // representing node address
    client.complete_task(&node_auth, &task_id, &result_bytes, &None);
}
