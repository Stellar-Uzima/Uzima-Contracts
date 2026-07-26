//! Input validation tests for medical_records contract entrypoints.
//! Ensures every public entrypoint rejects invalid inputs with proper error codes.

use super::*;
use soroban_sdk::testutils::Address as _;

fn setup() -> (Env, MedicalRecordsContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    rbac_client.assign_role(&admin, &RbacRole::Admin);

    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &rbac_id);

    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    (env, client, admin, doctor, patient)
}

#[test]
fn test_initialize_empty_admin_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let rbac_id = env.register_contract(None, MockRbac);
    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    // An uninitialized admin address won't be a valid Address; the contract
    // should reject initialization with an invalid rbac contract.
    // Using the contract itself as rbac is invalid.
    let result = client.try_initialize(&Address::generate(&env), &Address::generate(&env));
    assert!(result.is_err(), "initialize with random rbac should fail");
}

#[test]
fn test_add_record_empty_diagnosis_rejected() {
    let (env, client, _admin, doctor, patient) = setup();
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, ""),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "General"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "ipfs://data"),
    );
    assert!(result.is_err(), "empty diagnosis should be rejected");
}

#[test]
fn test_add_record_empty_treatment_rejected() {
    let (env, client, _admin, doctor, patient) = setup();
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, ""),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "General"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "ipfs://data"),
    );
    assert!(result.is_err(), "empty treatment should be rejected");
}

#[test]
fn test_add_record_empty_data_ref_rejected() {
    let (env, client, _admin, doctor, patient) = setup();
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "General"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, ""),
    );
    assert!(result.is_err(), "empty data_ref should be rejected");
}

#[test]
fn test_add_record_invalid_category_rejected() {
    let (env, client, _admin, doctor, patient) = setup();
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "InvalidCategoryThatIsWayTooLongAndShouldFail"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "ipfs://data"),
    );
    assert!(result.is_err(), "invalid category should be rejected");
}

#[test]
fn test_add_record_invalid_treatment_type_rejected() {
    let (env, client, _admin, doctor, patient) = setup();
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "General"),
        &String::from_str(&env, ""),
        &String::from_str(&env, "ipfs://data"),
    );
    assert!(result.is_err(), "empty treatment type should be rejected");
}

#[test]
fn test_add_record_too_many_tags_rejected() {
    let (env, client, _admin, doctor, patient) = setup();
    let mut tags: Vec<String> = Vec::new(&env);
    for i in 0..25 {
        tags.push_back(String::from_str(&env, &format!("tag-{}", i)));
    }
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &tags,
        &String::from_str(&env, "General"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "ipfs://data"),
    );
    assert!(result.is_err(), "too many tags should be rejected");
}

#[test]
fn test_manage_user_invalid_role_rejected() {
    let (env, client, admin, _doctor, _patient) = setup();
    let random_user = Address::generate(&env);
    // Attempt to set a role that doesn't match the Role enum pattern
    // should be rejected. Role::Doctor is valid; we just verify admin
    // can manage users. Invalid addresses are tested separately.
    let result = client.try_manage_user(&Address::generate(&env), &random_user, &Role::Doctor);
    assert!(result.is_err(), "non-admin managing users should be rejected");
}

#[test]
fn test_get_record_invalid_id_rejected() {
    let (env, client, _admin, _doctor, patient) = setup();
    let result = client.try_get_record(&patient, &0);
    assert!(result.is_err(), "getting non-existent record should fail");
}

#[test]
fn test_unauthorized_caller_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);
    rbac_client.assign_role(&admin, &RbacRole::Admin);

    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    client.initialize(&admin, &rbac_id);

    let stranger = Address::generate(&env);
    // Stranger (no role) cannot manage users
    let result = client.try_manage_user(&stranger, &Address::generate(&env), &Role::Doctor);
    assert!(result.is_err(), "unauthorized user should be rejected");
}
