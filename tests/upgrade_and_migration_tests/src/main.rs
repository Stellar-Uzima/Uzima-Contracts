//! tests/upgrade_and_migration_tests.rs

#![cfg(test)]

extern crate std;

use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::{Address as _, BytesN as _, Ledger, LedgerInfo},
    Address, BytesN, Env, Symbol, Val, Vec,
};

use crate::integration_framework::{
    get_contract_wasm, initialize_upgradeable_contract, UpgradeableContract,
};
use upgradeability::{migration::Migratable, UpgradeError};

// A simplified contract for testing migration logic.
#[contract]
pub struct MigratableContract;

// V1 of the contract state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DataKeyV1 {
    Counter = 0,
    Admin = 1,
}

// V2 of the contract state, adding a new field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DataKeyV2 {
    Counter = 0,
    Admin = 1,
    Message = 2, // New field in V2
}

const V1_WASM: &[u8] = get_contract_wasm("migratable_v1");
const V2_WASM: &[u8] = get_contract_wasm("migratable_v2");

#[contractimpl]
impl Migratable for MigratableContract {
    fn get_contract_version(env: &Env) -> u32 {
        env.storage().instance().get(&"version").unwrap_or(1)
    }

    fn set_contract_version(env: &Env, new_version: u32) {
        env.storage().instance().set(&"version", &new_version);
    }

    fn migrate_data(env: &Env, from_version: u32) {
        if from_version < 2 {
            // Migrate from V1 to V2: Add a default message.
            let default_message = "Hello, world!";
            env.storage()
                .persistent()
                .set(&DataKeyV2::Message, &default_message);
        }
    }
}

fn setup_test_environment<'a>() -> (Env, Address, UpgradeableContract<'a>) {
    let env = Env::default();
    env.ledger().set(LedgerInfo {
        timestamp: 12345,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
    });

    let admin = Address::generate(&env);
    let contract = initialize_upgradeable_contract(&env, V1_WASM, &admin);
    (env, admin, contract)
}

#[test]
fn test_successful_upgrade_and_migration() {
    let (env, admin, contract) = setup_test_environment();

    // Set a value in V1.
    let initial_counter: u32 = 123;
    contract
        .client
        .call(&symbol_short!("set_ctr"), &(&initial_counter,));

    // Upgrade to V2.
    let new_wasm_hash = env.deployer().upload(V2_WASM);
    contract.upgrade(&admin, &new_wasm_hash, 2, "Upgrade to V2");

    // Verify that the counter value is preserved.
    let counter_v2: u32 = contract.client.call(&symbol_short!("get_ctr"), &());
    assert_eq!(counter_v2, initial_counter);

    // Verify that the new field was added during migration.
    let message: String = contract.client.call(&symbol_short!("get_msg"), &());
    assert_eq!(message, "Hello, world!");
}

#[test]
fn test_unauthorized_upgrade_fails() {
    let (env, _, contract) = setup_test_environment();
    let non_admin = Address::generate(&env);

    let new_wasm_hash = env.deployer().upload(V2_WASM);
    let result = contract.try_upgrade(&non_admin, &new_wasm_hash, 2, "Unauthorized");

    assert_eq!(result, Err(Ok(UpgradeError::NotAuthorized)));
}

#[test]
fn test_rollback_to_previous_version() {
    let (env, admin, contract) = setup_test_environment();

    // Upgrade to V2 first.
    let new_wasm_hash = env.deployer().upload(V2_WASM);
    contract.upgrade(&admin, &new_wasm_hash, 2, "Upgrade to V2");

    // Verify we are on V2.
    let version_v2: u32 = contract.client.call(&symbol_short!("version"), &());
    assert_eq!(version_v2, 2);

    // Rollback to V1.
    contract.rollback(&admin);

    // Verify we are back on V1.
    let version_v1: u32 = contract.client.call(&symbol_short!("version"), &());
    assert_eq!(version_v1, 1);

    // Verify that V2 functionality is gone.
    let result: Result<String, _> = contract.client.try_call(&symbol_short!("get_msg"), &());
    assert!(result.is_err());
}