use soroban_sdk::{Env, Address, testutils::{Address as _}, String, vec};
use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};

#[test]
fn test_cross_chain_record_bridge_sync() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let record_hash = String::from_str(&env, "QmXoypizvW4C6vNbkCvM2EmxFTANf8wXmmE7DWjhx");

    // Initialize and set up users
    assert!(client.initialize(&admin));
    assert!(client.manage_user(&admin, &doctor, &Role::Doctor));
    assert!(client.manage_user(&admin, &patient, &Role::Patient));

    // Test successful record creation (cross-chain compatible)
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Cross-Chain Sync: Diabetes Follow-up"),
        &String::from_str(&env, "Remote monitoring"),
        &false,
        &vec![&env, String::from_str(&env, "cross-chain"), String::from_str(&env, "sync")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Telemedicine"),
        &record_hash,
    );

    assert!(result.is_ok(), "Cross-chain compatible record should be accepted");
}

#[test]
fn test_bridge_error_invalid_data() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    assert!(client.initialize(&admin));
    assert!(client.manage_user(&admin, &doctor, &Role::Doctor));
    assert!(client.manage_user(&admin, &patient, &Role::Patient));

    // Test: empty diagnosis should fail
    let res = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, ""),
        &String::from_str(&env, "Treatment"),
        &false,
        &soroban_sdk::vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );

    assert!(res.is_err(), "Should fail with empty diagnosis");
}
