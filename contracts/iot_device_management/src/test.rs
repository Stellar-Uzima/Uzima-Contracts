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

fn register_manufacturer(
    env: &Env,
    client: &IoTDeviceManagementClient,
    admin: &Address,
    id_byte: u8,
) -> BytesN<32> {
    let mfr_id = make_bytes32(env, id_byte);
    let cert = make_bytes32(env, id_byte.wrapping_add(100));
    let name = String::from_str(env, "TestManufacturer");
    client.register_manufacturer(admin, &mfr_id, &name, &cert);
    mfr_id
}

#[test]
fn test_register_manufacturer() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let mfr = client.get_manufacturer(&mfr_id);
    assert_eq!(mfr.is_active, true);
    assert_eq!(mfr.device_count, 0);
}

#[test]
fn test_register_manufacturer_duplicate() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    let cert = make_bytes32(&env, 200);
    let name = String::from_str(&env, "Dup");
    let result = client.try_register_manufacturer(&admin, &mfr_id, &name, &cert);
    assert_eq!(result, Err(Ok(IoTError::ManufacturerAlreadyRegistered)));
}

#[test]
fn test_deactivate_manufacturer() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    client.initialize(&admin);
    let mfr_id = register_manufacturer(&env, &client, &admin, 1);
    client.deactivate_manufacturer(&admin, &mfr_id);
    let mfr = client.get_manufacturer(&mfr_id);
    assert_eq!(mfr.is_active, false);
}
