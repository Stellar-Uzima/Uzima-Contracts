#![cfg(test)]

use crate::{MedicalRecordsContract, MedicalRecordsContractClient, Role};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

// This macro imports the WASM code for the current contract being tested
mod contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/medical_records.wasm"
    );
}

#[test]
fn test_migration_flow() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Deploy the contract initially
    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    // 2. Initialize with an Admin
    let admin = Address::generate(&env);
    client.initialize(&admin);

    // 3. To test "upgrade", we need valid WASM code.
    // In a real integration test, this would be the "new" V2 code.
    // For this unit test, we will re-upload the current code just to prove the mechanism works.

    // Note: If you haven't built the release WASM yet, this might fail.
    // A safer fallback for pure unit testing without external WASM files
    // is to register the contract again under a different ID and use that hash,
    // but Soroban's `update_current_contract_wasm` specifically requires a WASM hash.

    // ALTERNATIVE SAFE APPROACH FOR UNIT TESTS:
    // We can't easily get a valid WASM hash inside a unit test without compiling the WASM first.
    // However, since we just want to test that the 'upgrade' function executes successfully,
    // we can use a placeholder hash IF we mocked the deployer, but we can't easily mock the deployer internal checks.

    // CORRECT FIX: Use the install_contract_wasm method with the contract code.
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // 4. Attempt Upgrade as Admin
    // This should call migrate_data -> sets version to 1
    client.upgrade(&admin, &wasm_hash);

    // 5. Verify Non-Admin cannot upgrade
    let user = Address::generate(&env);
    let result = client.try_upgrade(&user, &wasm_hash);
    assert!(result.is_err());
}

#[test]
fn test_double_migration_check() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Upload the WASM code to the environment to get its hash
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // First Upgrade
    client.upgrade(&admin, &wasm_hash);

    // Second Upgrade (Should be safe and just update code, skipping migration block logic)
    client.upgrade(&admin, &wasm_hash);
}
