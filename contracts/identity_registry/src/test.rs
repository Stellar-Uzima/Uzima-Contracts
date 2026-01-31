use crate::{IdentityRegistry, IdentityRegistryClient, Service};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn create_contract(env: &Env) -> (IdentityRegistryClient<'_>, Address) {
    let contract_id = env.register_contract(None, IdentityRegistry);
    let client = IdentityRegistryClient::new(env, &contract_id);
    let owner = Address::generate(env);
    (client, owner)
}

#[test]
fn test_create_and_resolve_did() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, owner) = create_contract(&env);

    let initial_key = String::from_str(&env, "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH");
    let services: Vec<Service> = Vec::new(&env);

    let did_string = client.create_did(&owner, &initial_key, &services);

    let doc = client.resolve_did(&owner);
    assert_eq!(doc.id, did_string);
    assert_eq!(doc.version, 1);
    // FIXED: Changed assert_eq!(..., false) to assert!(!...)
    assert!(!doc.deactivated);
}

#[test]
fn test_deactivate_did() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, owner) = create_contract(&env);

    let initial_key = String::from_str(&env, "key1");
    let services: Vec<Service> = Vec::new(&env);
    client.create_did(&owner, &initial_key, &services);

    client.deactivate_did(&owner);

    let doc = client.resolve_did(&owner);
    // FIXED: Changed assert_eq!(..., true) to assert!(...)
    assert!(doc.deactivated);
}
