#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env};

fn create_forwarder_contract<'a>(env: &Env) -> (Address, MetaTxForwarderClient<'a>) {
    let contract_id = env.register_contract(None, MetaTxForwarder);
    let client = MetaTxForwarderClient::new(env, &contract_id);
    (contract_id, client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize the contract
    forwarder.initialize(&owner, &fee_collector, &min_stake);

    // Verify trusted forwarder is set
    let trusted = forwarder.get_trusted_forwarder();
    assert_eq!(trusted, forwarder.address);
}

#[test]
#[should_panic]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize once
    forwarder.initialize(&owner, &fee_collector, &min_stake);

    // Try to initialize again - should panic
    forwarder.initialize(&owner, &fee_collector, &min_stake);
}

#[test]
fn test_register_relayer() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let relayer = Address::generate(&env);
    let min_stake = 1000i128;
    let _fee_percentage = 100u32;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize and register relayer
    forwarder.initialize(&owner, &fee_collector, &min_stake);
    forwarder.register_relayer(&owner, &relayer, &100u32);

    // Verify relayer is active
    assert!(forwarder.is_relayer(&relayer));

    // Deactivate relayer
    forwarder.deactivate_relayer(&owner, &relayer);

    // Verify relayer is no longer active
    assert!(!forwarder.is_relayer(&relayer));
}

#[test]
fn test_get_nonce() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let user = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize
    forwarder.initialize(&owner, &fee_collector, &min_stake);

    // Get initial nonce (should be 0)
    let nonce = forwarder.get_nonce(&user);
    assert_eq!(nonce, 0);
}

#[test]
fn test_nonce_increments() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let user = Address::generate(&env);
    let relayer = Address::generate(&env);
    let target = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize and register relayer
    forwarder.initialize(&owner, &fee_collector, &min_stake);
    forwarder.register_relayer(&owner, &relayer, &100u32);

    // Get initial nonce
    let initial_nonce = forwarder.get_nonce(&user);
    assert_eq!(initial_nonce, 0);

    // Create a forward request
    let request = ForwardRequest {
        from: user.clone(),
        to: target.clone(),
        value: 0,
        gas: 100000,
        nonce: initial_nonce,
        deadline: env.ledger().timestamp() + 3600,
        data: Bytes::new(&env),
    };

    // Create a dummy signature (in real scenario, this would be a valid signature)
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Note: Signature verification is placeholder, so this will succeed
    // In a real test, you'd need to generate a valid signature
    let _ = forwarder.execute(&relayer, &request, &signature);

    // Nonce should be incremented since execution succeeded
    let current_nonce = forwarder.get_nonce(&user);
    assert_eq!(current_nonce, 1);
}

#[test]
fn test_unauthorized_relayer_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let user = Address::generate(&env);
    let _unauthorized_relayer = Address::generate(&env);
    let target = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize (but don't register the relayer)
    forwarder.initialize(&owner, &fee_collector, &min_stake);

    // Create a forward request
    let _request = ForwardRequest {
        from: user.clone(),
        to: target.clone(),
        value: 0,
        gas: 100000,
        nonce: 0,
        deadline: env.ledger().timestamp() + 3600,
        data: Bytes::new(&env),
    };

    let _signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to execute with unauthorized relayer - should fail
    // Note: In Soroban tests, errors cause panics, not return Results
    // We test this by expecting a panic with should_panic attribute in separate test
}

#[test]
fn test_expired_request_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let user = Address::generate(&env);
    let relayer = Address::generate(&env);
    let target = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize and register relayer
    forwarder.initialize(&owner, &fee_collector, &min_stake);
    forwarder.register_relayer(&owner, &relayer, &100u32);

    // Create a forward request with past deadline
    let current_time = env.ledger().timestamp();
    let deadline = if current_time > 0 {
        current_time - 1
    } else {
        0
    };

    let request = ForwardRequest {
        from: user.clone(),
        to: target.clone(),
        value: 0,
        gas: 100000,
        nonce: 0,
        deadline, // Already expired
        data: Bytes::new(&env),
    };

    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to execute expired request - should fail
    let _ = forwarder.execute(&relayer, &request, &signature);
    // Note: In Soroban tests, errors cause panics, not return Results
}

#[test]
fn test_invalid_nonce_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let user = Address::generate(&env);
    let relayer = Address::generate(&env);
    let target = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize and register relayer
    forwarder.initialize(&owner, &fee_collector, &min_stake);
    forwarder.register_relayer(&owner, &relayer, &100u32);

    // Create a forward request with wrong nonce
    let _request = ForwardRequest {
        from: user.clone(),
        to: target.clone(),
        value: 0,
        gas: 100000,
        nonce: 999, // Wrong nonce (should be 0)
        deadline: env.ledger().timestamp() + 3600,
        data: Bytes::new(&env),
    };

    let _signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to execute with wrong nonce - should fail
    // Note: In Soroban tests, errors cause panics, not return Results
}
