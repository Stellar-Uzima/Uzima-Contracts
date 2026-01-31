use crate::{MedicalRecordsContract, MedicalRecordsContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

// This loads the compiled WASM file.
// IF YOU SEE AN ERROR HERE: Run `soroban contract build` in your terminal first.
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

    // 3. Upload the WASM code to get a valid hash
    // (This relies on the file loaded by the 'contract' module above)
    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // 4. Attempt Upgrade as Admin
    // This should call migrate_data -> sets version to 1
    client.upgrade(&admin, &wasm_hash);

    // 5. Verify Non-Admin cannot upgrade
    let user = Address::generate(&env);
    // The upgrade function checks the passed 'caller' argument against the internal role.
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

    let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);

    // First Upgrade
    client.upgrade(&admin, &wasm_hash);

    // Second Upgrade (Should be safe and just update code, skipping migration block logic)
    client.upgrade(&admin, &wasm_hash);
}
