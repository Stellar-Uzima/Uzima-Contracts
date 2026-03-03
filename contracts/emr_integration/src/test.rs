#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

fn setup(env: &Env) -> (EMRIntegrationContractClient, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, EMRIntegrationContract);
    (EMRIntegrationContractClient::new(env, &id), id)
}

#[test]
fn initialize_smoke_test() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    let fhir_contract = Address::generate(&env);
    assert!(client.initialize(&admin, &fhir_contract));
}

