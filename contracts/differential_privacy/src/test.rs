use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

fn setup() -> (Env, DifferentialPrivacyContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, DifferentialPrivacyContract);
    let client = DifferentialPrivacyContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[test]
fn test_initialize_sets_admin() {
    let (env, client, admin) = setup();

    // Double initialization should fail
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

#[test]
fn test_initialize_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, DifferentialPrivacyContract);
    let client = DifferentialPrivacyContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    let other = Address::generate(&env);

    client.initialize(&admin);

    let result = client.try_initialize(&other);
    assert!(result.is_err());
}

// ============================================================================
// BUDGET TESTS
// ============================================================================

#[test]
fn test_create_budget() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);

    let budget_id = client.create_budget(&admin, &data_owner, &100);
    assert!(budget_id.len() > 0);

    let remaining = client.get_remaining_budget(&budget_id);
    assert_eq!(remaining, 100);
}

#[test]
fn test_create_budget_zero_epsilon() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);

    let result = client.try_create_budget(&admin, &data_owner, &0);
    assert!(result.is_err());
}

#[test]
fn test_create_multiple_budgets() {
    let (env, client, admin) = setup();
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);

    let budget1 = client.create_budget(&admin, &owner1, &50);
    let budget2 = client.create_budget(&admin, &owner2, &75);

    assert_ne!(budget1, budget2);
    assert_eq!(client.get_remaining_budget(&budget1), 50);
    assert_eq!(client.get_remaining_budget(&budget2), 75);
}

// ============================================================================
// LAPLACE NOISE TESTS
// ============================================================================

#[test]
fn test_add_laplace_noise() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &data_owner, &100);

    let query_id = BytesN::from_array(&env, &[1u8; 32]);
    let query = client.add_laplace_noise(
        &data_owner,
        &budget_id,
        &query_id,
        &DataType::Numerical,
        &100,
        &10,
    );

    assert_eq!(query.budget_id, budget_id);

    // Verify budget was decremented
    let remaining = client.get_remaining_budget(&budget_id);
    assert_eq!(remaining, 90);

    // Verify query was stored
    let stored = client.get_query(&query_id);
    assert!(stored.is_some());
}

#[test]
fn test_laplace_noise_insufficient_budget() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &data_owner, &5);

    let query_id = BytesN::from_array(&env, &[1u8; 32]);
    let result = client.try_add_laplace_noise(
        &data_owner,
        &budget_id,
        &query_id,
        &DataType::Numerical,
        &100,
        &10, // sensitivity > remaining budget
    );

    assert!(result.is_err());
}

#[test]
fn test_laplace_noise_zero_sensitivity() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &data_owner, &100);

    let query_id = BytesN::from_array(&env, &[1u8; 32]);
    let result = client.try_add_laplace_noise(
        &data_owner,
        &budget_id,
        &query_id,
        &DataType::Numerical,
        &100,
        &0, // Zero sensitivity should fail
    );

    assert!(result.is_err());
}

// ============================================================================
// GAUSSIAN NOISE TESTS
// ============================================================================

#[test]
fn test_add_gaussian_noise() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &data_owner, &200);

    let query_id = BytesN::from_array(&env, &[2u8; 32]);
    let query = client.add_gaussian_noise(
        &data_owner,
        &budget_id,
        &query_id,
        &DataType::Count,
        &500,
        &10,
    );

    assert_eq!(query.mechanism, NoiseMechanism::Gaussian);

    // Gaussian cost = 2x sensitivity = 20
    let remaining = client.get_remaining_budget(&budget_id);
    assert_eq!(remaining, 180);
}

#[test]
fn test_gaussian_noise_insufficient_budget() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &data_owner, &10);

    let query_id = BytesN::from_array(&env, &[3u8; 32]);
    let result = client.try_add_gaussian_noise(
        &data_owner,
        &budget_id,
        &query_id,
        &DataType::Count,
        &500,
        &10, // Cost = 20 > remaining budget = 10
    );

    assert!(result.is_err());
}

// ============================================================================
// BUDGET MANAGEMENT TESTS
// ============================================================================

#[test]
fn test_deactivate_budget() {
    let (env, client, admin) = setup();
    let data_owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &data_owner, &100);

    client.deactivate_budget(&admin, &budget_id);

    // Queries on deactivated budget should fail
    let query_id = BytesN::from_array(&env, &[4u8; 32]);
    let result = client.try_add_laplace_noise(
        &data_owner,
        &budget_id,
        &query_id,
        &DataType::Numerical,
        &100,
        &5,
    );
    assert!(result.is_err());
}

#[test]
fn test_deactivate_nonexistent_budget() {
    let (env, client, admin) = setup();
    let budget_id = BytesN::from_array(&env, &[99u8; 32]);

    let result = client.try_deactivate_budget(&admin, &budget_id);
    assert!(result.is_err());
}

// ============================================================================
// QUERY TESTS
// ============================================================================

#[test]
fn test_get_nonexistent_query() {
    let (env, client, _admin) = setup();
    let query_id = BytesN::from_array(&env, &[99u8; 32]);

    let result = client.get_query(&query_id);
    assert!(result.is_none());
}

#[test]
fn test_get_remaining_budget_nonexistent() {
    let (env, client, _admin) = setup();
    let budget_id = BytesN::from_array(&env, &[99u8; 32]);

    let result = client.try_get_remaining_budget(&budget_id);
    assert!(result.is_err());
}

// ============================================================================
// END-TO-END SCENARIO
// ============================================================================

#[test]
fn test_full_privacy_workflow() {
    let (env, client, admin) = setup();
    let researcher = Address::generate(&env);

    // Create budget
    let budget_id = client.create_budget(&admin, &researcher, &100);

    // Run multiple queries
    for i in 0..5 {
        let query_id = BytesN::from_array(&env, &[i as u8; 32]);
        let _query = client.add_laplace_noise(
            &researcher,
            &budget_id,
            &query_id,
            &DataType::Numerical,
            &(i * 100),
            &5,
        );
    }

    // Budget should be 100 - (5 * 5) = 75
    let remaining = client.get_remaining_budget(&budget_id);
    assert_eq!(remaining, 75);
}

#[test]
fn test_mixed_laplace_gaussian_workflow() {
    let (env, client, admin) = setup();
    let researcher = Address::generate(&env);

    let budget_id = client.create_budget(&admin, &researcher, &200);

    // Laplace query (cost = 10)
    let _ = client.add_laplace_noise(
        &researcher,
        &budget_id,
        &BytesN::from_array(&env, &[1u8; 32]),
        &DataType::Numerical,
        &100,
        &10,
    );

    // Gaussian query (cost = 20)
    let _ = client.add_gaussian_noise(
        &researcher,
        &budget_id,
        &BytesN::from_array(&env, &[2u8; 32]),
        &DataType::Count,
        &200,
        &10,
    );

    // Budget should be 200 - 10 - 20 = 170
    assert_eq!(client.get_remaining_budget(&budget_id), 170);
}
