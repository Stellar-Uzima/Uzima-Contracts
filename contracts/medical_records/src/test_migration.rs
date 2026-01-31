use crate::{MedicalRecordsContract, MedicalRecordsContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_migration_admin_check() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let dummy_hash = BytesN::<32>::from_array(&env, &[0u8; 32]);

    // Verify security guard: Non-Admin should fail
    let user = Address::generate(&env);
    let result = client.try_upgrade(&user, &dummy_hash);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Not authorized")]
fn test_migration_admin_check_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let dummy_hash = BytesN::<32>::from_array(&env, &[0u8; 32]);
    let user = Address::generate(&env);

    client.upgrade(&user, &dummy_hash);
}
