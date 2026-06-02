use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, Address, ContractTemplateClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, ContractTemplate);
    let client = ContractTemplateClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, admin, client)
}

#[test]
fn test_initialize() {
    let (_, _, client) = setup();
    let admin2 = Address::generate(&client.env);
    assert_eq!(client.try_initialize(&admin2), Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_update_data_as_admin() {
    let (env, admin, client) = setup();
    let data = String::from_str(&env, "hello");
    assert!(client.try_update_data(&admin, &data).is_ok());
    let stored = client.get_data().unwrap();
    assert_eq!(stored.value, data);
}

#[test]
fn test_update_data_unauthorized() {
    let (env, _, client) = setup();
    let other = Address::generate(&env);
    let data = String::from_str(&env, "hack");
    assert_eq!(client.try_update_data(&other, &data), Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_transfer_admin() {
    let (env, admin, client) = setup();
    let new_admin = Address::generate(&env);
    assert!(client.try_transfer_admin(&new_admin).is_ok());
    assert_eq!(client.get_admin(), new_admin);

    let data = String::from_str(&env, "old");
    assert_eq!(client.try_update_data(&admin, &data), Err(Ok(Error::Unauthorized)));
}
