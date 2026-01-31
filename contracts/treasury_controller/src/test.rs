use crate::{ProposalType, TreasuryControllerContract, TreasuryControllerContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Bytes, Env,
};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TreasuryControllerContract);
    let client = TreasuryControllerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let signer = Address::generate(&env);

    client.initialize(
        &admin,
        &token,
        &vec![&env, signer],
        &1,
        &3600, // 1 hour
    );
}

#[test]
fn test_create_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TreasuryControllerContract);
    let client = TreasuryControllerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let signer = Address::generate(&env);
    let target = Address::generate(&env);

    client.initialize(&admin, &token, &vec![&env, signer.clone()], &1, &3600);

    let id = client.create_proposal(
        &signer,
        &ProposalType::Transfer,
        &target,
        &1000,
        &3600,
        &Bytes::new(&env),
    );

    assert_eq!(id, 0);
}

#[test]
fn test_approve_and_execute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TreasuryControllerContract);
    let client = TreasuryControllerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let signer = Address::generate(&env);
    let target = Address::generate(&env);

    client.initialize(&admin, &token, &vec![&env, signer.clone()], &1, &3600);

    let id = client.create_proposal(
        &signer,
        &ProposalType::Transfer,
        &target,
        &1000,
        &3600,
        &Bytes::new(&env),
    );

    client.approve_proposal(&signer, &id);

    // Advance time
    env.ledger().set_timestamp(env.ledger().timestamp() + 3601);

    let res = client.execute_proposal(&signer, &id);
    assert!(res);
}
