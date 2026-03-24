use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn setup() -> (Env, DifferentialPrivacyContractClient<'static>, Address) {
    let env = Env::default();
    let id = Address::generate(&env);
    env.register_contract(&id, DifferentialPrivacyContract);
    let client = DifferentialPrivacyContractClient::new(&env, &id);
    (env, client, id)
}

#[test]
fn test_initialize_and_create_budget() {
    let (env, client, _id) = setup();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let data_owner = Address::generate(&env);

    // Initialize
    client.initialize(&admin);

    // Create budget with epsilon = 100
    let epsilon_total = 100u64;
    let budget_id = client.create_budget(&admin, &data_owner, &epsilon_total);

    // Verify budget was created
    let remaining = client.get_remaining_budget(&budget_id);
    assert_eq!(remaining, epsilon_total);
}
