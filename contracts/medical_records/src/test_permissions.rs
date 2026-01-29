#![cfg(test)]

use crate::{MedicalRecordsContract, MedicalRecordsContractClient, Permission};
use soroban_sdk::{
    testutils::{Address as _, Events},
    vec, Address, Env, String, Symbol,
};

#[test]
fn test_permission_grant_revoke_check() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    // Initialize
    client.initialize(&admin);

    // Test: User cannot create record without permission
    let res = client.try_add_record(
        &user,
        &patient,
        &String::from_str(&env, "Flu"),
        &String::from_str(&env, "Rest"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Standard"),
        &String::from_str(&env, "QmHash12345"),
    );
    assert!(res.is_err()); // access denied (Error::NotAuthorized)

    // Test: Admin grants CreateRecord permission to user
    client.grant_permission(&admin, &user, &Permission::CreateRecord, &0, &false);

    // Test: User can now create record
    let res = client.try_add_record(
        &user,
        &patient,
        &String::from_str(&env, "Flu"),
        &String::from_str(&env, "Rest"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Standard"),
        &String::from_str(&env, "QmHash12345"),
    );
    assert!(res.is_ok());

    // Test: Admin revokes permission
    client.revoke_permission(&admin, &user, &Permission::CreateRecord);

    // Test: User cannot create record anymore
    let res = client.try_add_record(
        &user,
        &patient,
        &String::from_str(&env, "Flu"),
        &String::from_str(&env, "Rest"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Standard"),
        &String::from_str(&env, "QmHash12345"),
    );
    assert!(res.is_err());
}

#[test]
fn test_permission_delegation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let manager = Address::generate(&env);
    let user = Address::generate(&env);
    let patient = Address::generate(&env);

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    // Admin grants DelegatePermission to manager
    client.grant_permission(
        &admin,
        &manager,
        &Permission::DelegatePermission,
        &0,
        &false, // Manager cannot delegate the delegation itself (strict hierarchy 1 level)
    );

    // Manager grants CreateRecord to user
    client.grant_permission(&manager, &user, &Permission::CreateRecord, &0, &false);

    // User tries to create record
    let res = client.try_add_record(
        &user,
        &patient,
        &String::from_str(&env, "Flu"),
        &String::from_str(&env, "Rest"),
        &false,
        &vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Standard"),
        &String::from_str(&env, "QmHash12345"),
    );
    assert!(res.is_ok());

    // User checks if they can delegate? No they don't have DelegatePermission.
    let res = client.try_grant_permission(&user, &patient, &Permission::ReadRecord, &0, &false);
    assert!(res.is_err());
}
