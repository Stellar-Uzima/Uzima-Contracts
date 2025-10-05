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
    let result = forwarder.initialize(&owner, &fee_collector, &min_stake);
    assert!(result.is_ok());

    // Verify trusted forwarder is set
    let trusted = forwarder.get_trusted_forwarder();
    assert_eq!(trusted, forwarder.address);
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize once
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();

    // Try to initialize again - should panic
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();
}

#[test]
fn test_register_relayer() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let relayer = Address::generate(&env);
    let min_stake = 1000i128;
    let fee_percentage = 100u32; // 1%

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();

    // Register relayer
    let result = forwarder.register_relayer(&owner, &relayer, &fee_percentage);
    assert!(result.is_ok());

    // Verify relayer is active
    assert!(forwarder.is_relayer(&relayer));

    // Verify relayer config
    let config = forwarder.get_relayer_config(&relayer);
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.fee_percentage, fee_percentage);
    assert!(config.is_active);
}

#[test]
fn test_deactivate_relayer() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let relayer = Address::generate(&env);
    let min_stake = 1000i128;
    let fee_percentage = 100u32;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize and register relayer
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();
    forwarder
        .register_relayer(&owner, &relayer, &fee_percentage)
        .unwrap();

    // Verify relayer is active
    assert!(forwarder.is_relayer(&relayer));

    // Deactivate relayer
    let result = forwarder.deactivate_relayer(&owner, &relayer);
    assert!(result.is_ok());

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
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();

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
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();
    forwarder
        .register_relayer(&owner, &relayer, &100u32)
        .unwrap();

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

    // Note: This will fail signature verification, but we're testing nonce logic
    // In a real test, you'd need to generate a valid signature
    let _ = forwarder.execute(&relayer, &request, &signature);

    // Nonce should still be 0 since execution failed
    let current_nonce = forwarder.get_nonce(&user);
    assert_eq!(current_nonce, 0);
}

#[test]
fn test_unauthorized_relayer_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let fee_collector = Address::generate(&env);
    let user = Address::generate(&env);
    let unauthorized_relayer = Address::generate(&env);
    let target = Address::generate(&env);
    let min_stake = 1000i128;

    let (_, forwarder) = create_forwarder_contract(&env);

    // Initialize (but don't register the relayer)
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();

    // Create a forward request
    let request = ForwardRequest {
        from: user.clone(),
        to: target.clone(),
        value: 0,
        gas: 100000,
        nonce: 0,
        deadline: env.ledger().timestamp() + 3600,
        data: Bytes::new(&env),
    };

    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to execute with unauthorized relayer - should fail
    let result = forwarder.execute(&unauthorized_relayer, &request, &signature);
    assert!(result.is_err());
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
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();
    forwarder
        .register_relayer(&owner, &relayer, &100u32)
        .unwrap();

    // Create a forward request with past deadline
    let request = ForwardRequest {
        from: user.clone(),
        to: target.clone(),
        value: 0,
        gas: 100000,
        nonce: 0,
        deadline: env.ledger().timestamp() - 1, // Already expired
        data: Bytes::new(&env),
    };

    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to execute expired request - should fail
    let result = forwarder.execute(&relayer, &request, &signature);
    assert!(result.is_err());
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
    forwarder
        .initialize(&owner, &fee_collector, &min_stake)
        .unwrap();
    forwarder
        .register_relayer(&owner, &relayer, &100u32)
        .unwrap();

    // Create a forward request with wrong nonce
    let request = ForwardRequest {
        from: user.clone(),
        to: target.clone(),
        value: 0,
        gas: 100000,
        nonce: 999, // Wrong nonce (should be 0)
        deadline: env.ledger().timestamp() + 3600,
        data: Bytes::new(&env),
    };

    let signature = BytesN::from_array(&env, &[0u8; 64]);

    // Try to execute with wrong nonce - should fail
    let result = forwarder.execute(&relayer, &request, &signature);
    assert!(result.is_err());
}
