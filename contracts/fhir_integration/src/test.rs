#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

fn setup(env: &Env) -> (FHIRIntegrationContractClient, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, FHIRIntegrationContract);
    (FHIRIntegrationContractClient::new(env, &id), id)
}

#[test]
fn initialize_smoke_test() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let medical_records_contract = Address::generate(&env);
    assert!(client.initialize(&admin, &medical_records_contract));
}

