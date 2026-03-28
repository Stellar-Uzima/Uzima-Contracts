#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{vec, Address, BytesN, Env, String};

fn setup(env: &Env) -> (IoTDeviceManagementClient, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, IoTDeviceManagement);
    let client = IoTDeviceManagementClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    (client, admin)
}

fn make_bytes32(env: &Env, val: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = val;
    BytesN::from_array(env, &bytes)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    // Calling initialize again should fail
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(IoTError::AlreadyInitialized)));
}

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    client.pause(&admin);
    // set_role should fail when paused
    let user = Address::generate(&env);
    let result = client.try_set_role(&admin, &user, &Role::Operator);
    assert_eq!(result, Err(Ok(IoTError::ContractPaused)));
    client.unpause(&admin);
    // Should work after unpause
    client.set_role(&admin, &user, &Role::Operator);
}

#[test]
fn test_pause_not_admin() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let non_admin = Address::generate(&env);
    let result = client.try_pause(&non_admin);
    assert_eq!(result, Err(Ok(IoTError::NotAdmin)));
}

#[test]
fn test_set_role() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let user = Address::generate(&env);
    client.set_role(&admin, &user, &Role::Operator);
    let role = client.get_role(&user);
    assert_eq!(role, Role::Operator);
}
