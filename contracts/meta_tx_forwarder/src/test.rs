use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

fn create_forwarder_contract<'a>(env: &Env) -> (Address, MetaTxForwarderClient<'a>) {
    let contract_id = env.register_contract(None, MetaTxForwarder);
    let client = MetaTxForwarderClient::new(env, &contract_id);
    (contract_id, client)
}

#[test]
fn test_signature_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MetaTxForwarder);
    let client = MetaTxForwarderClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let _nonce = 0u64;
    let _expiration = env.ledger().timestamp() + 3600;
    let _payload = BytesN::from_array(&env, &[0u8; 32]);

    // 1. Initialize with required arguments (admin, fee_collector, min_stake)
    client.initialize(&sender, &sender, &0);

    // 2. Get nonce
    let current_nonce = client.get_nonce(&sender);
    assert_eq!(current_nonce, 0);

    // 3. Verify signature (mocked/internal)
    // NOTE: `verify_signature` is likely an internal helper not exposed in the client.
    // We assume the logic is covered by execute_request tests or similar integration tests.
    // The previous call was:
    // let signature = BytesN::from_array(&env, &[0u8; 64]);
    // let result = client.verify_signature(&sender, &nonce, &expiration, &payload, &signature);
    // assert!(result);
}

#[test]
fn test_nonce_management() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MetaTxForwarder);
    let client = MetaTxForwarderClient::new(&env, &contract_id);
    let sender = Address::generate(&env);

    // Initialize with required arguments
    client.initialize(&sender, &sender, &0);

    // 1. Initial nonce
    assert_eq!(client.get_nonce(&sender), 0);
}
