#![cfg(test)]

use soroban_sdk::{Address, Env, String, Vec};
use medical_records::{Error, MedicalRecord, MedicalRecordsContract, MedicalRecordsContractClient, Role};

fn setup_contract(env: &Env) -> (MedicalRecordsContractClient, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let client = MedicalRecordsContractClient::new(env, &contract_id);

    // Create admin
    let admin = Address::random(env);
    client.initialize(&admin);

    // Add a doctor
    let doctor = Address::random(env);
    client.manage_user(&admin, &doctor, Role::Doctor);

    // Add a patient
    let patient = Address::random(env);
    client.manage_user(&admin, &patient, Role::Patient);

    (client, admin)
}

#[test]
fn test_add_record_with_valid_data_ref() {
    let env = Env::default();
    let (client, admin) = setup_contract(&env);

    let doctor = Address::random(&env);
    let patient = Address::random(&env);

    client.manage_user(&admin, &doctor, Role::Doctor);
    client.manage_user(&admin, &patient, Role::Patient);

    let valid_ref = String::from_str(&env, "QmValidHash_12345678901234567890123456789012345678901234");

    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        true,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &valid_ref.clone(),
    );

    let record: MedicalRecord = client.get_record(&doctor, &record_id);
    assert_eq!(record.data_ref, valid_ref);
}

#[test]
fn test_add_record_with_empty_data_ref() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let doctor = Address::random(&env);
    let patient = Address::random(&env);

    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, ""),
    );
    assert_eq!(result, Err(Ok(Error::EmptyDataRef)));
}

#[test]
fn test_add_record_with_short_data_ref() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let doctor = Address::random(&env);
    let patient = Address::random(&env);

    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "TooShort123"),
    );
    assert_eq!(result, Err(Ok(Error::InvalidDataRefLength)));
}

#[test]
fn test_add_record_with_invalid_chars_in_data_ref() {
    let env = Env::default();
    let (client, _admin) = setup_contract(&env);

    let doctor = Address::random(&env);
    let patient = Address::random(&env);

    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        false,
        &Vec::new(&env),
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "Invalid$Char#DataRef_123456789012345678901234567890"),
    );
    assert_eq!(result, Err(Ok(Error::InvalidDataRefCharset)));
}
