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
