// internal
use crate::{MedicalRecordsContract, MedicalRecordsContractClient, MockRbac, MockRbacClient, RbacRole};

// external crates
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_migration_admin_check() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);

    client.initialize(&admin, &rbac_id);

    let dummy_hash = BytesN::<32>::from_array(&env, &[0u8; 32]);

    // Verify security guard: Non-Admin should fail
    let user = Address::generate(&env);
    let result = client.try_upgrade(&user, &dummy_hash, &2u32);
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_migration_admin_check_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);

    client.initialize(&admin, &rbac_id);

    let dummy_hash = BytesN::<32>::from_array(&env, &[0u8; 32]);
    let user = Address::generate(&env);

    client.upgrade(&user, &dummy_hash, &2u32);
}
