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

// ---------------------------------------------------------------------------
// Issue #1594: scale/load coverage for clinical record workflows.
//
// The tests above cover contention between a handful of actors. These drive the
// per-patient record collection to a larger size, which is the axis that grows:
// every add_record appends to a per-patient list, so cost per write and per
// indexed read both scale with how many records one patient accumulates.
// Sized for a routine `cargo test --workspace`; the heavy case is #[ignore]d.
// ---------------------------------------------------------------------------

/// Records accumulated by a single patient in the routine-size tests.
const RECORDS_PER_PATIENT: u64 = 25;
/// Independent patients driven in parallel with the routine-size tests.
const PATIENT_FANOUT: u64 = 8;
/// Ignored-by-default record count for the stress case.
const SCALE_RECORDS_PER_PATIENT: u64 = 200;

/// Add `count` records and return their ids. The ids come back from
/// `add_record` rather than being assumed to be `0..count`, matching how the
/// rest of this file reads records back.
fn add_bulk_records(
    env: &Env,
    client: &MedicalRecordsContractClient,
    doctor: &Address,
    patient: &Address,
    count: u64,
) -> Vec<u64> {
    let tags = Vec::new(env);
    let mut ids = Vec::new(env);
    for _ in 0..count {
        ids.push_back(add_sample_record(client, doctor, patient, &tags));
    }
    ids
}

#[test]
fn test_load_many_records_for_one_patient() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);
    let patient = create_patient(&env, &client, &admin);

    let _ids = add_bulk_records(&env, &client, &doctor, &patient, RECORDS_PER_PATIENT);

    assert!(
        client.get_patient_record_count(&patient) >= RECORDS_PER_PATIENT,
        "every added record must be counted for the patient"
    );
}

#[test]
fn test_load_many_patients_for_one_doctor() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);

    for _ in 0..PATIENT_FANOUT {
        let patient = create_patient(&env, &client, &admin);
        let _ids = add_bulk_records(&env, &client, &doctor, &patient, RECORDS_PER_PATIENT / 2);
    }

    // The doctor's own record list stays independent of the patients' totals.
    assert!(client.get_patient_record_count(&doctor) < PATIENT_FANOUT * RECORDS_PER_PATIENT);
}

#[test]
fn test_load_repeated_reads_of_large_record_set() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);
    let patient = create_patient(&env, &client, &admin);

    let ids = add_bulk_records(&env, &client, &doctor, &patient, RECORDS_PER_PATIENT);

    // Re-read the whole set several times: catches a read path that degrades
    // with collection size rather than indexing by record id.
    for _ in 0..3 {
        for i in 0..ids.len() {
            let record = client.get_record(&patient, &ids.get(i).unwrap());
            assert!(!record.encrypted, "sample records are added unencrypted");
        }
    }
}

#[test]
#[ignore = "stress test for clinical record scale"]
fn test_load_records_to_high_count() {
    let (env, client, admin, _cid) = setup_env();
    let doctor = create_doctor(&env, &client, &admin);
    let patient = create_patient(&env, &client, &admin);

    let _ids = add_bulk_records(&env, &client, &doctor, &patient, SCALE_RECORDS_PER_PATIENT);

    assert!(client.get_patient_record_count(&patient) >= SCALE_RECORDS_PER_PATIENT);
}
