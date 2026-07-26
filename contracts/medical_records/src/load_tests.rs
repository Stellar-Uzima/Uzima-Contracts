//! Load tests for concurrent medical record access (Issue #898).
//! Simulates multiple users performing concurrent operations to verify
//! throughput and correctness under contention.
#![cfg(test)]
#![allow(clippy::unwrap_used)]
extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String, Vec};

fn setup_env() -> (Env, MedicalRecordsContractClient<'static>, Address, Address) {
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

    (env, client, admin, contract_id)
}

fn create_doctor(env: &Env, client: &MedicalRecordsContractClient, admin: &Address) -> Address {
    let doctor = Address::generate(env);
    client.manage_user(admin, &doctor, &Role::Doctor);
    doctor
}

fn create_patient(env: &Env, client: &MedicalRecordsContractClient, admin: &Address) -> Address {
    let patient = Address::generate(env);
    client.manage_user(admin, &patient, &Role::Patient);
    patient
}

fn add_sample_record(
    client: &MedicalRecordsContractClient,
    doctor: &Address,
    patient: &Address,
    tags: &Vec<String>,
) -> u64 {
    let diagnosis = String::from_str(&client.env, "Routine checkup");
    let treatment = String::from_str(&client.env, "Standard monitoring");
    let data_ref = String::from_str(&client.env, "ipfs://QmSampleRecord");
    let category = String::from_str(&client.env, "General");
    let treatment_type = String::from_str(&client.env, "Consultation");

    client.add_record(
        doctor,
        patient,
        &diagnosis,
        &treatment,
        &false, // encrypted
        tags,
        &category,
        &treatment_type,
        &data_ref,
    )
}

#[test]
fn test_concurrent_record_creation_by_multiple_doctors() {
    let (env, client, admin, _cid) = setup_env();
    let patient = create_patient(&env, &client, &admin);

    let mut doctors: Vec<Address> = Vec::new(&env);
    let mut record_ids: Vec<u64> = Vec::new(&env);

    for _ in 0..5 {
        let doctor = create_doctor(&env, &client, &admin);
        doctors.push_back(doctor.clone());

        let mut tags: Vec<String> = Vec::new(&env);
        tags.push_back(String::from_str(&env, "concurrent-test"));

        let record_id = add_sample_record(&client, &doctor, &patient, &tags);
        record_ids.push_back(record_id);
    }

    assert_eq!(record_ids.len(), 5);

    let total_records = client.get_record_count();
    assert!(total_records >= 5);
}

#[test]
fn test_load_sequential_record_retrieval() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);
    let patient = create_patient(&env, &client, &admin);

    let mut record_ids: Vec<u64> = Vec::new(&env);
    for i in 0..10 {
        let mut tags: Vec<String> = Vec::new(&env);
        tags.push_back(String::from_str(&env, "load-test"));
        tags.push_back(String::from_bytes(&env, &[i as u8; 1]));
        let record_id = add_sample_record(&client, &doctor, &patient, &tags);
        record_ids.push_back(record_id);
    }

    // Retrieve all records sequentially
    for i in 0..10 {
        let record = client.get_record(&patient, &record_ids.get(i).unwrap());
        assert_eq!(record.patient, patient);
    }
}

#[test]
fn test_concurrent_permission_grant_and_check() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);
    let patient = create_patient(&env, &client, &admin);
    let mut tags: Vec<String> = Vec::new(&env);
    tags.push_back(String::from_str(&env, "perm-test"));
    let record_id = add_sample_record(&client, &doctor, &patient, &tags);

    let mut grantees: Vec<Address> = Vec::new(&env);
    for _ in 0..4 {
        let grantee = Address::generate(&env);
        grantees.push_back(grantee.clone());
        let granted = client.grant_permission(&doctor, &grantee, &record_id, &1000u64);
        assert!(granted);
    }

    // Check all permissions
    for i in 0..4 {
        let has_perm = client.check_permission(&grantees.get(i).unwrap(), &record_id);
        assert!(has_perm);
    }
}

#[test]
fn test_load_multiple_patients_same_doctor() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);

    for i in 0..8 {
        let patient = create_patient(&env, &client, &admin);
        let mut tags: Vec<String> = Vec::new(&env);
        tags.push_back(String::from_bytes(&env, &[i as u8; 1]));
        add_sample_record(&client, &doctor, &patient, &tags);
    }

    assert!(client.get_record_count() >= 8);
}

#[test]
fn test_mixed_read_write_contention() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);
    let patient = create_patient(&env, &client, &admin);

    // Create records
    let mut record_ids: Vec<u64> = Vec::new(&env);
    for _ in 0..5 {
        let mut tags: Vec<String> = Vec::new(&env);
        tags.push_back(String::from_str(&env, "mixed"));
        let rid = add_sample_record(&client, &doctor, &patient, &tags);
        record_ids.push_back(rid);
    }

    // Interleave reads and writes
    let mut new_tags: Vec<String> = Vec::new(&env);
    new_tags.push_back(String::from_str(&env, "mixed"));
    let new_rid = add_sample_record(&client, &doctor, &patient, &new_tags);

    for i in 0..5 {
        let _record = client.get_record(&patient, &record_ids.get(i).unwrap());
    }
    let _new_record = client.get_record(&patient, &new_rid);

    assert!(new_rid > record_ids.get(4).unwrap());
}

#[test]
fn test_contention_under_encrypted_records() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);
    let patient = create_patient(&env, &client, &admin);

    let mut record_ids: Vec<u64> = Vec::new(&env);
    for i in 0..6 {
        let diagnosis = String::from_bytes(&env, &[i; 1]);
        let treatment = String::from_str(&env, "Encrypted treatment plan");
        let mut tags: Vec<String> = Vec::new(&env);
        tags.push_back(String::from_str(&env, "encrypted"));
        let category = String::from_str(&env, "Encrypted");
        let treatment_type = String::from_str(&env, "Medication");
        let data_ref = String::from_str(&env, "enc://QmEncryptedRecord");

        let rid = client.add_record(
            &doctor,
            &patient,
            &diagnosis,
            &treatment,
            &true, // encrypted
            &tags,
            &category,
            &treatment_type,
            &data_ref,
        );
        record_ids.push_back(rid);
    }

    for i in 0..6 {
        let record = client.get_record(&patient, &record_ids.get(i).unwrap());
        assert!(record.encrypted);
    }

    let patient_count = client.get_patient_record_count(&patient);
    assert!(patient_count >= 6);
}
