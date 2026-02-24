#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
use super::*;
use crate::errors::Error;
use soroban_sdk::testutils::{Address as _, Events, Ledger};
use soroban_sdk::{symbol_short, vec, Address, Env, String, Symbol, TryFromVal, Vec};

fn create_contract(env: &Env) -> (MedicalRecordsContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordsContract);

    let client = MedicalRecordsContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_add_and_get_record() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let diagnosis = String::from_str(&env, "Common cold");
    let treatment = String::from_str(&env, "Rest and fluids");
    let is_confidential = false;
    let tags = vec![&env, String::from_str(&env, "respiratory")];
    let category = String::from_str(&env, "Modern");
    let treatment_type = String::from_str(&env, "Medication");

    // Initialize and set roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);
    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx");
    let initial_event_count = env.events().all().len();

    let record_id = client.add_record(
        &doctor,
        &patient,
        &diagnosis,
        &treatment,
        &is_confidential,
        &tags,
        &category,
        &treatment_type,
        &data_ref,
    );

    // Verify events were emitted
    let events_after_add = env.events().all();
    assert!(events_after_add.len() > initial_event_count);

    // Check for record creation events
    let record_events_count = events_after_add
        .iter()
        .filter(|e| {
            if e.1.len() < 2 {
                return false;
            }
            let topic = e.1.get(1).unwrap();
            let sym = Symbol::try_from_val(&env, &topic).unwrap();
            sym == symbol_short!("REC_NEW")
        })
        .count();
    assert_eq!(record_events_count, 1);

    // Get the record as patient
    let record = client.get_record(&patient, &record_id);
    assert_eq!(record.patient_id, patient);
    assert_eq!(record.diagnosis, diagnosis);
    assert_eq!(record.treatment, treatment);
    assert!(!record.is_confidential);

    // Verify record access event was emitted
    let events_after_get = env.events().all();
    let access_events_count = events_after_get
        .iter()
        .filter(|e| {
            if e.1.len() < 2 {
                return false;
            }
            let topic = e.1.get(1).unwrap();
            let sym = Symbol::try_from_val(&env, &topic).unwrap();
            sym == symbol_short!("REC_ACC")
        })
        .count();
    assert_eq!(access_events_count, 1);
}

#[test]
fn test_empty_data_ref() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Empty data_ref should fail
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, ""),
    );

    assert_eq!(result, Err(Ok(Error::EmptyDataRef)));
}

#[test]
fn test_data_ref_too_short() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Data ref shorter than 10 chars should fail
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "Qm123"),
    );

    assert_eq!(result, Err(Ok(Error::InvalidDataRefLength)));
}

#[test]
fn test_data_ref_too_long() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Create a string longer than 200 characters (201 chars)
    let long_ref = String::from_str(&env, "Qmaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &long_ref,
    );

    assert_eq!(result, Err(Ok(Error::InvalidDataRefLength)));
}

#[test]
fn test_data_ref_boundary_min_length() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Exactly 10 chars (should pass)
    let min_ref = String::from_str(&env, "Qm12345678");
    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &min_ref,
    );

    let record = client.get_record(&patient, &record_id);
    assert_eq!(record.patient_id, patient);
}

#[test]
fn test_data_ref_boundary_max_length() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Exactly 200 characters
    let max_ref = String::from_str(&env, "Qmaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"); // 200 chars

    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &max_ref,
    );

    let record = client.get_record(&patient, &record_id);
    assert_eq!(record.patient_id, patient);
}

#[test]
fn test_get_patient_records() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let _diagnosis = String::from_str(&env, "Common cold");
    let _treatment = String::from_str(&env, "Rest and fluids");
    let _is_confidential = false;
    let _tags = vec![&env, String::from_str(&env, "respiratory")];
    let _category = String::from_str(&env, "Modern");
    let _treatment_type = String::from_str(&env, "Medication");

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add multiple records for the same patient
    let record_id1 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &false,
        &vec![&env, String::from_str(&env, "herbal")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let record_id2 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &true,
        &vec![&env, String::from_str(&env, "spiritual")],
        &String::from_str(&env, "Spiritual"),
        &String::from_str(&env, "Prayer"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Patient can access both records
    client.get_record(&patient, &record_id1);
    client.get_record(&patient, &record_id2);
}

#[test]
fn test_role_based_access() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let _diagnosis = String::from_str(&env, "Common cold");
    let _treatment = String::from_str(&env, "Rest and fluids");
    let _is_confidential = false;
    let _tags = vec![&env, String::from_str(&env, "respiratory")];
    let _category = String::from_str(&env, "Modern");
    let _treatment_type = String::from_str(&env, "Medication");

    // Admin manages user roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Doctor adds a confidential record
    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &true,
        &vec![&env, String::from_str(&env, "spiritual")],
        &String::from_str(&env, "Spiritual"),
        &String::from_str(&env, "Prayer"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );
    // Patient tries to access the record (should succeed)
    client.get_record(&patient, &record_id);

    // Doctor (creator) tries to access the record (should succeed)
    client.get_record(&doctor, &record_id);

    // Admin tries to access the record (should succeed)
    client.get_record(&admin, &record_id);
}

#[test]
fn test_deactivate_user() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    // Admin manages user roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Deactivate the doctor
    client.deactivate_user(&admin, &doctor);

    // Try to add a record as the deactivated doctor (should fail)
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Cold"),
        &String::from_str(&env, "Rest"),
        &false,
        &vec![&env, String::from_str(&env, "herbal")],
        &String::from_str(&env, "General"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn test_pause_unpause_blocks_sensitive_functions_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize and set up roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add a record (not paused)
    let _record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "herbal")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Pause the contract
    client.pause(&admin);

    // Mutating functions should be blocked when paused
    let result = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Blocked"),
        &String::from_str(&env, "Blocked"),
        &false,
        &vec![&env],
        &String::from_str(&env, "General"),
        &String::from_str(&env, "General"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    assert_eq!(result, Err(Ok(Error::ContractPaused)));
}

#[test]
fn test_pause_unpause_blocks_sensitive_functions() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize and set up roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add a record (not paused)
    let _record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "herbal")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Pause the contract
    client.pause(&admin);

    // Unpause
    assert!(client.unpause(&admin));

    // Now mutating calls should succeed
    assert!(client.manage_user(&admin, &Address::generate(&env), &Role::Doctor));
    let _r3 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis3"),
        &String::from_str(&env, "Treatment3"),
        &false,
        &vec![&env, String::from_str(&env, "herbal")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );
}

#[test]
fn test_monotonic_record_ids() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize and set roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add multiple records and verify IDs are monotonically increasing
    let record_id1 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &false,
        &vec![&env, String::from_str(&env, "tag1")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Type1"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let record_id2 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &false,
        &vec![&env, String::from_str(&env, "tag2")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Type2"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let record_id3 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 3"),
        &String::from_str(&env, "Treatment 3"),
        &false,
        &vec![&env, String::from_str(&env, "tag3")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Type3"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Verify IDs are monotonically increasing
    assert_eq!(record_id1, 1);
    assert_eq!(record_id2, 2);
    assert_eq!(record_id3, 3);
    assert!(record_id2 > record_id1);
    assert!(record_id3 > record_id2);
}

#[test]
fn test_unique_record_ids() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor1 = Address::generate(&env);
    let doctor2 = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize and set roles
    client.manage_user(&admin, &doctor1, &Role::Doctor);
    client.manage_user(&admin, &doctor2, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add records from different doctors
    let record_id1 = client.add_record(
        &doctor1,
        &patient,
        &String::from_str(&env, "Diagnosis A"),
        &String::from_str(&env, "Treatment A"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "TypeA"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let record_id2 = client.add_record(
        &doctor2,
        &patient,
        &String::from_str(&env, "Diagnosis B"),
        &String::from_str(&env, "Treatment B"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "TypeB"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Verify all IDs are unique
    assert_ne!(record_id1, record_id2);
}

#[test]
fn test_record_ordering() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);

    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize and set roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add records in sequence
    let mut record_ids: Vec<u64> = Vec::new(&env);
    for _i in 0..5 {
        let id = client.add_record(
            &doctor,
            &patient,
            &String::from_str(&env, "Diagnosis"),
            &String::from_str(&env, "Treatment"),
            &false,
            &vec![&env, String::from_str(&env, "tag")],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "Type"),
            &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        );
        record_ids.push_back(id);
    }

    // Verify ordering is preserved
    for i in 1..record_ids.len() {
        assert!(record_ids.get(i).unwrap() > record_ids.get(i - 1).unwrap());
    }
}

/*
#[test]
fn test_record_counter_isolation() {
    // ...
}
*/

/*
#[test]
fn test_get_history_pagination_and_access() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let doctor1 = Address::generate(&env);
    let doctor2 = Address::generate(&env);
    let patient = Address::generate(&env);
    let diagnosis1 = String::from_str(&env, "Diagnosis 1");
    let treatment1 = String::from_str(&env, "Treatment 1");
    let diagnosis2 = String::from_str(&env, "Diagnosis 2");
    let treatment2 = String::from_str(&env, "Treatment 2");
    let diagnosis3 = String::from_str(&env, "Diagnosis 3");
    let treatment3 = String::from_str(&env, "Treatment 3");

    // Initialize and set roles
    client.manage_user(&admin, &doctor1, &Role::Doctor);
    client.manage_user(&admin, &doctor2, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add records with different doctors and confidentiality
    let _ = client.add_record(
        &doctor1,
        &patient,
        &diagnosis1,
        &treatment1,
        &false, // non-confidential
        &vec![&env, String::from_str(&env, "tag1")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let _ = client.add_record(
        &doctor1,
        &patient,
        &diagnosis2,
        &treatment2,
        &true, // confidential
        &vec![&env, String::from_str(&env, "tag2")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    let record_id3 = client.add_record(
        &doctor1,
        &patient,
        &diagnosis3,
        &treatment3,
        &false, // non-confidential
        &vec![&env, String::from_str(&env, "tag3")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Surgery"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Patient gets full history (page 0, size 3) - should get all 3
    let history = client.get_history(&patient, &patient, &0u32, &3u32);
    assert_eq!(history.len(), 3);

    let history_doc2 = client.get_history(&doctor2, &patient, &0u32, &1u32);
    assert_eq!(history_doc2.len(), 1);

    let history_doc2_page2 = client.get_history(&doctor2, &patient, &2u32, &1u32);
    assert_eq!(history_doc2_page2.len(), 1);
    assert_eq!(history_doc2_page2.get(0u32).unwrap().0, record_id3);

    let history_doc2_full = client.get_history(&doctor2, &patient, &0u32, &3u32);
    assert_eq!(history_doc2_full.len(), 2);

    let history_admin = client.get_history(&admin, &patient, &0u32, &3u32);
    assert_eq!(history_admin.len(), 3);

    let empty_page = client
        .mock_all_auths()
        .get_history(&patient, &patient, &3u32, &1u32);
    assert_eq!(empty_page.len(), 0);
}
*/

/*
#[test]
fn test_ai_integration_points() {
    // ...
}
*/

/*
#[test]
fn test_ai_validation() {
    // ...
}
*/

#[test]
fn test_get_record_count_getter() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Initially zero records
    assert_eq!(client.get_record_count(), 0u64);

    // Add first record
    let _ = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &false,
        &vec![&env, String::from_str(&env, "tag1")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Type1"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Add second record
    let _ = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &false,
        &vec![&env, String::from_str(&env, "tag2")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Type2"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    assert_eq!(client.get_record_count(), 2u64);
}

// ============================================================================
// Rate Limiting Tests
// ============================================================================

#[test]
fn test_rate_limit_add_record_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Set config: max 2 calls per window for doctor
    client.set_rate_limit_config(
        &admin,
        &1u32, // OP_ADD_RECORD
        &RateLimitConfig {
            doctor_max_calls: 2,
            patient_max_calls: 0,
            admin_max_calls: 0,
            window_secs: 3600,
        },
    );

    // Call 1 - success
    client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "D1"),
        &String::from_str(&env, "T1"),
        &false,
        &soroban_sdk::vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Med"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Call 2 - success
    client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "D2"),
        &String::from_str(&env, "T2"),
        &false,
        &soroban_sdk::vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Med"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXy"),
    );

    // Call 3 - fails
    let res = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "D3"),
        &String::from_str(&env, "T3"),
        &false,
        &soroban_sdk::vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Med"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXz"),
    );
    assert_eq!(res, Err(Ok(Error::RateLimitExceeded)));
}

#[test]
fn test_rate_limit_resets_after_window() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Config: 1 call per window of 100 seconds
    client.set_rate_limit_config(
        &admin,
        &1u32,
        &RateLimitConfig {
            doctor_max_calls: 1,
            patient_max_calls: 1,
            admin_max_calls: 1,
            window_secs: 100,
        },
    );

    // At time 1000
    env.ledger().set_timestamp(1000);

    // Call 1
    client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "D1"),
        &String::from_str(&env, "T1"),
        &false,
        &soroban_sdk::vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Med"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Call 2 immediately fails
    let res = client.try_add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "D2"),
        &String::from_str(&env, "T2"),
        &false,
        &soroban_sdk::vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Med"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );
    assert_eq!(res, Err(Ok(Error::RateLimitExceeded)));

    // Advance time past window (1101 > 1000 + 100)
    env.ledger().set_timestamp(1101);

    // Call 3 succeeds
    client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "D3"),
        &String::from_str(&env, "T3"),
        &false,
        &soroban_sdk::vec![&env],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Med"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );
}

#[test]
fn test_rate_limit_different_roles() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let other_doctor = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);

    // OP_MANAGE_USER = 2
    client.set_rate_limit_config(
        &admin,
        &2u32,
        &RateLimitConfig {
            doctor_max_calls: 1,
            patient_max_calls: 0,
            admin_max_calls: 2,
            window_secs: 3600,
        },
    );

    // Admin can manage 2 users
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &other_doctor, &Role::Doctor);
    
    // 3rd time fails for admin
    let res = client.try_manage_user(&admin, &Address::generate(&env), &Role::Patient);
    assert_eq!(res, Err(Ok(Error::RateLimitExceeded)));
}

#[test]
fn test_rate_limit_admin_bypass() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let trusted_service = Address::generate(&env);
    let patient = Address::generate(&env);
    
    // Give trusted service Doctor permissions
    client.manage_user(&admin, &trusted_service, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Config: 1 call limit
    client.set_rate_limit_config(
        &admin,
        &1u32, // OP_ADD_RECORD
        &RateLimitConfig {
            doctor_max_calls: 1,
            patient_max_calls: 1,
            admin_max_calls: 1,
            window_secs: 3600,
        },
    );

    // Grant bypass
    client.set_rate_limit_bypass(&admin, &trusted_service, &true);

    // Because of bypass, trusted_service can make > 1 calls
    for i in 0..3 {
        client.add_record(
            &trusted_service,
            &patient,
            &String::from_str(&env, "D"),
            &String::from_str(&env, "T"),
            &false,
            &soroban_sdk::vec![&env],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "Med"),
            &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        );
    }
}
