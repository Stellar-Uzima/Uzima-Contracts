use crate::{
    types::{FederatedRound, ModelMetadata},
    AiAnalyticsContract, AiAnalyticsContractClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup() -> (Env, AiAnalyticsContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, AiAnalyticsContract);
    let client = AiAnalyticsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.mock_all_auths().initialize(&admin);
    (env, client, admin)
}

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[test]
fn test_initialize() {
    let (_env, client, _admin) = setup();

    // Verify contract is initialized (no panic = success)
}

#[test]
fn test_double_initialize() {
    let (env, client, _admin) = setup();
    let admin2 = Address::generate(&env);

    let result = client.mock_all_auths().try_initialize(&admin2);
    assert!(result.is_err());
}

// ============================================================================
// FEDERATED ROUND TESTS
// ============================================================================

#[test]
fn test_start_round() {
    let (env, client, admin) = setup();
    let base_model = BytesN::from_array(&env, &[1u8; 32]);

    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2, &1);
    assert!(round_id > 0);

    let round: FederatedRound = client.get_round(&round_id).unwrap();
    assert!(!round.is_finalized);
}

#[test]
fn test_start_round_unauthorized() {
    let (env, client, _admin) = setup();
    let other = Address::generate(&env);
    let base_model = BytesN::from_array(&env, &[1u8; 32]);

    let result = client
        .mock_all_auths()
        .try_start_round(&other, &base_model, &2, &1);
    assert!(result.is_err());
}

#[test]
fn test_start_multiple_rounds() {
    let (env, client, admin) = setup();
    let model1 = BytesN::from_array(&env, &[1u8; 32]);
    let model2 = BytesN::from_array(&env, &[2u8; 32]);

    let round1 = client.mock_all_auths().start_round(&admin, &model1, &2, &1);
    let round2 = client.mock_all_auths().start_round(&admin, &model2, &3, &2);

    assert_ne!(round1, round2);
    assert!(client.get_round(&round1).is_some());
    assert!(client.get_round(&round2).is_some());
}

// ============================================================================
// SUBMIT UPDATE TESTS
// ============================================================================

#[test]
fn test_submit_update() {
    let (env, client, admin) = setup();
    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2, &1);

    let participant = Address::generate(&env);
    let update_hash = BytesN::from_array(&env, &[2u8; 32]);

    assert!(client
        .mock_all_auths()
        .submit_update(&participant, &round_id, &update_hash, &10));
}

#[test]
fn test_submit_update_nonexistent_round() {
    let (env, client, _admin) = setup();
    let participant = Address::generate(&env);
    let update_hash = BytesN::from_array(&env, &[2u8; 32]);

    let result = client
        .mock_all_auths()
        .try_submit_update(&participant, &999, &update_hash, &10);
    assert!(result.is_err());
}

#[test]
fn test_multiple_participants_submit_updates() {
    let (env, client, admin) = setup();
    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &3, &1);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    let p3 = Address::generate(&env);

    assert!(client.mock_all_auths().submit_update(
        &p1,
        &round_id,
        &BytesN::from_array(&env, &[10u8; 32]),
        &10
    ));
    assert!(client.mock_all_auths().submit_update(
        &p2,
        &round_id,
        &BytesN::from_array(&env, &[20u8; 32]),
        &20
    ));
    assert!(client.mock_all_auths().submit_update(
        &p3,
        &round_id,
        &BytesN::from_array(&env, &[30u8; 32]),
        &15
    ));
}

// ============================================================================
// FINALIZE ROUND TESTS
// ============================================================================

#[test]
fn test_finalize_round() {
    let (env, client, admin) = setup();
    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &1, &1);

    let participant = Address::generate(&env);
    client.mock_all_auths().submit_update(
        &participant,
        &round_id,
        &BytesN::from_array(&env, &[2u8; 32]),
        &10,
    );

    let new_model = BytesN::from_array(&env, &[99u8; 32]);
    assert!(client.mock_all_auths().finalize_round(
        &admin,
        &round_id,
        &new_model,
        &String::from_str(&env, "Final model"),
        &String::from_str(&env, "ipfs://metrics"),
        &String::from_str(&env, "ipfs://fairness"),
    ));

    let round: FederatedRound = client.get_round(&round_id).unwrap();
    assert!(round.is_finalized);

    let model: ModelMetadata = client.get_model(&new_model).unwrap();
    assert_eq!(model.round_id, round_id);
}

#[test]
fn test_finalize_round_unauthorized() {
    let (env, client, admin) = setup();
    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &1, &1);

    let participant = Address::generate(&env);
    client.mock_all_auths().submit_update(
        &participant,
        &round_id,
        &BytesN::from_array(&env, &[2u8; 32]),
        &10,
    );

    let other = Address::generate(&env);
    let new_model = BytesN::from_array(&env, &[99u8; 32]);
    let result = client.mock_all_auths().try_finalize_round(
        &other,
        &round_id,
        &new_model,
        &String::from_str(&env, "Hacked"),
        &String::from_str(&env, ""),
        &String::from_str(&env, ""),
    );
    assert!(result.is_err());
}

// ============================================================================
// QUERY TESTS
// ============================================================================

#[test]
fn test_get_nonexistent_round() {
    let (_env, client, _admin) = setup();
    assert!(client.get_round(&999).is_none());
}

#[test]
fn test_get_nonexistent_model() {
    let (env, client, _admin) = setup();
    let model_id = BytesN::from_array(&env, &[99u8; 32]);
    assert!(client.get_model(&model_id).is_none());
}

// ============================================================================
// END-TO-END FEDERATED LEARNING SCENARIO
// ============================================================================

#[test]
fn test_full_federated_learning_workflow() {
    let (env, client, admin) = setup();

    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2, &1);

    let participant1 = Address::generate(&env);
    let participant2 = Address::generate(&env);
    let update_hash1 = BytesN::from_array(&env, &[2u8; 32]);
    let update_hash2 = BytesN::from_array(&env, &[3u8; 32]);

    // Submit updates from both participants
    assert!(client
        .mock_all_auths()
        .submit_update(&participant1, &round_id, &update_hash1, &10));
    assert!(client
        .mock_all_auths()
        .submit_update(&participant2, &round_id, &update_hash2, &20));

    // Finalize round
    let new_model = BytesN::from_array(&env, &[4u8; 32]);
    assert!(client.mock_all_auths().finalize_round(
        &admin,
        &round_id,
        &new_model,
        &String::from_str(&env, "Aggregated model v1"),
        &String::from_str(&env, "ipfs://metrics/v1"),
        &String::from_str(&env, "ipfs://fairness/v1"),
    ));

    // Verify round state
    let round: FederatedRound = client.get_round(&round_id).unwrap();
    assert!(round.is_finalized);

    // Verify model metadata
    let model: ModelMetadata = client.get_model(&new_model).unwrap();
    assert_eq!(model.round_id, round_id);
}

#[test]
fn test_finalize_round_insufficient_participants() {
    let (env, client, admin) = setup();
    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    // Require 2 participants but only submit 1
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2, &1);

    let participant = Address::generate(&env);
    client.mock_all_auths().submit_update(
        &participant,
        &round_id,
        &BytesN::from_array(&env, &[2u8; 32]),
        &10,
    );

    let new_model = BytesN::from_array(&env, &[99u8; 32]);
    let result = client.mock_all_auths().try_finalize_round(
        &admin,
        &round_id,
        &new_model,
        &String::from_str(&env, "Too early"),
        &String::from_str(&env, ""),
        &String::from_str(&env, ""),
    );
    assert!(result.is_err());
}
