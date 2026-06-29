//! Storage-read regression baselines for cross_chain_bridge.
#![allow(clippy::unwrap_used)]
extern crate std;

use super::*;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env, String};

fn measure_cpu<F: FnOnce()>(env: &Env, f: F) -> u64 {
    env.budget().reset_unlimited();
    f();
    env.budget().cpu_instruction_cost()
}

fn print_delta(name: &str, before: u64, after: u64) {
    let saved = before.saturating_sub(after);
    let reduction_pct = if before == 0 {
        0.0
    } else {
        (saved as f64 * 100.0) / before as f64
    };
    std::println!(
        "[STORAGE-BENCH] {} before={} after={} saved={} reduction_pct={:.2}",
        name,
        before,
        after,
        saved,
        reduction_pct
    );
}

fn create_contract(
    env: &Env,
) -> (
    CrossChainBridgeContractClient<'_>,
    Address,
    Address,
    Address,
    Address,
) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrossChainBridgeContract);
    let client = CrossChainBridgeContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let medical_contract = Address::generate(env);
    let identity_contract = Address::generate(env);
    let access_contract = Address::generate(env);
    client.initialize(
        &admin,
        &medical_contract,
        &identity_contract,
        &access_contract,
    );
    (
        client,
        admin,
        medical_contract,
        identity_contract,
        access_contract,
    )
}

fn generate_keypair() -> (VerifyingKey, SigningKey) {
    let mut rng = rand::thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    (verifying_key, signing_key)
}

fn make_public_key(env: &Env, vk: &VerifyingKey) -> BytesN<32> {
    BytesN::from_array(env, &vk.to_bytes())
}

fn create_sig(env: &Env, signing_key: &SigningKey, data: &BytesN<32>, nonce: u64) -> BytesN<64> {
    let mut payload = Bytes::new(env);
    payload.extend_from_array(&data.to_array());
    payload.extend_from_array(&nonce.to_be_bytes());
    let hash = env.crypto().sha256(&payload);
    let sig = signing_key.sign(&hash.to_array());
    BytesN::from_array(env, &sig.to_bytes())
}

fn setup_validator(
    env: &Env,
    client: &CrossChainBridgeContractClient<'_>,
    admin: &Address,
) -> (Address, SigningKey) {
    let (vk, sk) = generate_keypair();
    let public_key = make_public_key(env, &vk);
    let validator = Address::generate(env);
    client.add_validator(admin, &validator, &public_key, &1000);
    (validator, sk)
}

fn submit_message_scenario(
    env: &Env,
) -> (
    CrossChainBridgeContractClient<'_>,
    Address,
    SubmitMessageRequest,
) {
    let (client, admin, _medical, _identity, _access) = create_contract(env);
    let (validator, sk) = setup_validator(env, &client, &admin);
    let message_id = BytesN::from_array(env, &[11u8; 32]);
    let request = SubmitMessageRequest {
        message_id: message_id.clone(),
        source_chain: ChainId::Ethereum,
        dest_chain: ChainId::Stellar,
        sender: String::from_str(env, "0xbench-sender"),
        recipient: Address::generate(env),
        payload_type: MessageType::RecordSync,
        payload: String::from_str(env, "{\"record_id\":1}"),
        nonce: 1,
        signature: BytesN::from_array(env, &[3u8; 64]),
        v_signature: create_sig(env, &sk, &message_id, 1),
        v_nonce: 1,
    };
    (client, validator, request)
}

fn rollback_scenario(env: &Env) -> (CrossChainBridgeContractClient<'_>, Address, BytesN<32>) {
    let (client, admin, _medical, _identity, _access) = create_contract(env);
    let op_id = BytesN::from_array(env, &[29u8; 32]);
    client.create_operation(&admin, &op_id, &OperationType::TokenTransfer, &admin);
    (client, admin, op_id)
}

#[test]
fn bench_storage_add_supported_chain() {
    let env_before = Env::default();
    let (client_before, admin_before, _medical_before, _identity_before, _access_before) =
        create_contract(&env_before);
    let before = measure_cpu(&env_before, || {
        client_before.add_supported_chain(&admin_before, &ChainId::Avalanche);
    });

    let env_after = Env::default();
    let (client_after, admin_after, _medical_after, _identity_after, _access_after) =
        create_contract(&env_after);
    let after = measure_cpu(&env_after, || {
        client_after.add_supported_chain(&admin_after, &ChainId::Avalanche);
    });

    print_delta("cross_chain_bridge::add_supported_chain", before, after);
    assert!(after <= before);
}

#[test]
fn bench_storage_submit_message() {
    let env_before = Env::default();
    let (client_before, validator_before, request_before) = submit_message_scenario(&env_before);
    let before = measure_cpu(&env_before, || {
        client_before.submit_message(&validator_before, &request_before);
    });

    let env_after = Env::default();
    let (client_after, validator_after, request_after) = submit_message_scenario(&env_after);
    let after = measure_cpu(&env_after, || {
        client_after.submit_message(&validator_after, &request_after);
    });

    print_delta("cross_chain_bridge::submit_message", before, after);
    assert!(after <= before);
}

#[test]
fn bench_storage_initiate_rollback() {
    let env_before = Env::default();
    let (client_before, admin_before, op_before) = rollback_scenario(&env_before);
    let before = measure_cpu(&env_before, || {
        client_before.initiate_rollback(
            &admin_before,
            &op_before,
            &RollbackOpType::MessageRollback,
            &String::from_str(&env_before, "pending"),
            &String::from_str(&env_before, "bench"),
        );
    });

    let env_after = Env::default();
    let (client_after, admin_after, op_after) = rollback_scenario(&env_after);
    let after = measure_cpu(&env_after, || {
        client_after.initiate_rollback(
            &admin_after,
            &op_after,
            &RollbackOpType::MessageRollback,
            &String::from_str(&env_after, "pending"),
            &String::from_str(&env_after, "bench"),
        );
    });

    print_delta("cross_chain_bridge::initiate_rollback", before, after);
    assert!(after <= before);
}
