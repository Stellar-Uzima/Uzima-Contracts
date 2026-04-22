use crate::{
    types::{FederatedRound, ModelMetadata},
    AiAnalyticsContract, AiAnalyticsContractClient,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

#[test]
fn test_federated_round_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AiAnalyticsContract);
    let client = AiAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let participant1 = Address::generate(&env);
    let participant2 = Address::generate(&env);

    client.mock_all_auths().initialize(&admin);

    let base_model = BytesN::from_array(&env, &[1u8; 32]);
    let round_id = client
        .mock_all_auths()
        .start_round(&admin, &base_model, &2u32, &1u32);

    let update_hash1 = BytesN::from_array(&env, &[2u8; 32]);
    let update_hash2 = BytesN::from_array(&env, &[3u8; 32]);

    assert!(client.mock_all_auths().submit_update(
        &participant1,
        &round_id,
        &update_hash1,
        &10u32
    ));
    assert!(client.mock_all_auths().submit_update(
        &participant2,
        &round_id,
        &update_hash2,
        &20u32
    ));

    let new_model = BytesN::from_array(&env, &[4u8; 32]);
    assert!(client.mock_all_auths().finalize_round(
        &admin,
        &round_id,
        &new_model,
        &String::from_str(&env, "Test model"),
        &String::from_str(&env, "ipfs://metrics"),
        &String::from_str(&env, "ipfs://fairness"),
    ));

    let stored_round: FederatedRound = client.get_round(&round_id).unwrap();
    assert!(stored_round.is_finalized);

    let stored_model: ModelMetadata = client.get_model(&new_model).unwrap();
    assert_eq!(stored_model.round_id, round_id);
}
