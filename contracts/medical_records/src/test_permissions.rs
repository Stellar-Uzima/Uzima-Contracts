#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

extern crate std;

// internal
use crate::{MedicalRecordsContract, MedicalRecordsContractClient, Permission, MockRbac, MockRbacClient, RbacRole};

// external crates
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

#[test]
fn test_permission_grant_revoke_check() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let patient = Address::generate(&env);
    let user = Address::generate(&env);

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);

    // Initialize
    client.initialize(&admin, &rbac_id);

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
    assert!(res.is_err()); // access denied (Error::Unauthorized)

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

    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);

    client.initialize(&admin, &rbac_id);

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

#[test]
fn test_access_attribute_issue_revoke_and_epoch_rotation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);

    client.initialize(&admin, &rbac_id);
    client.manage_user(&admin, &doctor, &crate::Role::Doctor);

    assert!(client.issue_access_attribute(
        &admin,
        &doctor,
        &String::from_str(&env, "region"),
        &String::from_str(&env, "KE"),
        &0,
        &true,
    ));

    let attrs = client.get_user_access_attributes(&doctor);
    assert_eq!(attrs.len(), 1);
    assert!(attrs.get(0).unwrap().is_active);
    assert_eq!(attrs.get(0).unwrap().epoch, 1);

    assert!(client.revoke_access_attribute(
        &admin,
        &doctor,
        &String::from_str(&env, "region"),
        &String::from_str(&env, "KE"),
    ));

    let attrs = client.get_user_access_attributes(&doctor);
    assert!(!attrs.get(0).unwrap().is_active);

    let epoch = client.get_access_attribute_epoch(
        &String::from_str(&env, "region"),
        &String::from_str(&env, "KE"),
    );
    assert_eq!(epoch, 2);
}

#[test]
fn test_storage_read_optimization_snapshots() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);

    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);

    client.initialize(&admin, &rbac_id);
    let registry = Address::generate(&env);
    client.set_crypto_registry(&admin, &registry);
    client.manage_user(&admin, &doctor, &crate::Role::Doctor);
    client.manage_user(&admin, &patient, &crate::Role::Patient);

    // Snapshot 1: issue_access_attribute CPU cost
    let start_issue = env.budget().cpu_instruction_cost();
    client.issue_access_attribute(
        &admin,
        &doctor,
        &String::from_str(&env, "region"),
        &String::from_str(&env, "KE"),
        &0,
        &true,
    );
    let cpu_issue = env.budget().cpu_instruction_cost() - start_issue;
    std::println!("SNAPSHOT [medical_records - issue_access_attribute]: CPU={}", cpu_issue);

    // Snapshot 2: revoke_access_attribute CPU cost
    let start_revoke = env.budget().cpu_instruction_cost();
    client.revoke_access_attribute(
        &admin,
        &doctor,
        &String::from_str(&env, "region"),
        &String::from_str(&env, "KE"),
    );
    let cpu_revoke = env.budget().cpu_instruction_cost() - start_revoke;
    std::println!("SNAPSHOT [medical_records - revoke_access_attribute]: CPU={}", cpu_revoke);

    // Snapshot 3: add_encrypted_record CPU cost (calls can_view_encrypted_record/role validation)
    let start_encrypt = env.budget().cpu_instruction_cost();
    let mut envs = vec![&env];
    envs.push_back(crate::KeyEnvelope {
        recipient: patient.clone(),
        key_version: 1,
        algorithm: crate::EnvelopeAlgorithm::X25519,
        wrapped_key: soroban_sdk::Bytes::from_slice(&env, &[1u8; 32]),
        pq_wrapped_key: None,
    });
    client.add_encrypted_record(
        &doctor,
        &patient,
        &true,
        &vec![&env, String::from_str(&env, "KE")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmHash12345"),
        &soroban_sdk::BytesN::from_array(&env, &[0u8; 32]),
        &envs,
    );
    let cpu_encrypt = env.budget().cpu_instruction_cost() - start_encrypt;
    std::println!("SNAPSHOT [medical_records - add_encrypted_record]: CPU={}", cpu_encrypt);
}

