use soroban_sdk::{Address, Env, String, Vec};
use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role};

#[test]
fn test_validation_input_constraints() {
    let env = Env::default();
    env.mock_all_auths();

    // Initialize contract
    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Test 1: Empty diagnosis (should fail)
    let res = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, ""), // Empty diagnosis
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );
    assert!(res.is_err() || res.unwrap().is_err());

    // Test 2: Invalid category
    let res = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "InvalidCategory"), // Invalid
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );
    assert!(res.is_err() || res.unwrap().is_err());

    // Test 3: Same address for patient and doctor (should fail)
    // Note: The doctor is the caller in add_record, so we pass doctor as caller and doctor as patient
    let res = client.try_add_record(
        &doctor,
        &doctor, // Patient same as doctor
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
    );
    assert!(res.is_err() || res.unwrap().is_err()); 

    // Test 4: Invalid data ref (too short)
    let res = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "Short"), // Tool short
    );
    assert!(res.is_err() || res.unwrap().is_err());
}

#[test]
fn test_pagination_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let patient = Address::generate(&env);
    
    // Test 0 page size (should fail)
    let res = client.try_get_history(&admin, &patient, &0u32, &0u32);
    assert!(res.is_err() || res.unwrap().is_err());
    
    // Test too large page size (should fail)
    let res = client.try_get_history(&admin, &patient, &0u32, &101u32);
    assert!(res.is_err() || res.unwrap().is_err());
}
