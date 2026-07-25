#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_async_reconciliation_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SyncManagerContract);
    let client = SyncManagerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let source = Address::generate(&env);
    let payload_hash = BytesN::from_array(&env, &[1u8; 32]);

    client.initialize(&admin);

    // Enqueue job
    let job_id = client.enqueue_reconciliation(&source, &payload_hash);
    assert_eq!(job_id, 1);

    // Resolve job
    client.process_reconciliation(&job_id, &true);
}