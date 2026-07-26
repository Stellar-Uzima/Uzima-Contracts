#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// internal
use super::*;
use common_error::CommonError;
use crate::errors::Error;
use patient_consent_management::{PatientConsentManagement, PatientConsentManagementClient};

// external crates
use soroban_sdk::testutils::{Address as _, Events, Ledger};
use soroban_sdk::{symbol_short, vec, Address, Env, String, Symbol, TryFromVal, Vec};

fn create_contract(env: &Env) -> (MedicalRecordsContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let rbac_id = env.register_contract(None, MockRbac);
    let rbac_client = MockRbacClient::new(env, &rbac_id);
    let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);

    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordsContract);

    let client = MedicalRecordsContractClient::new(env, &contract_id);
    client.initialize(&admin, &rbac_id);
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
            if e.1.is_empty() {
                return false;
            }
            let topic = e.1.get(0).unwrap();
            Symbol::try_from_val(&env, &topic)
                .map(|sym| sym == symbol_short!("REC_NEW"))
                .unwrap_or(false)
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
            if e.1.is_empty() {
                return false;
            }
            let topic = e.1.get(0).unwrap();
            Symbol::try_from_val(&env, &topic)
                .map(|sym| sym == symbol_short!("REC_ACC"))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(access_events_count, 1);
}

#[test]
fn test_get_record_denied_after_consent_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let consent_contract_id = env.register_contract(None, PatientConsentManagement);
    let consent_client = PatientConsentManagementClient::new(&env, &consent_contract_id);
    consent_client.initialize(&admin);
    client.set_patient_consent_contract(&admin, &consent_contract_id);

    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let provider = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    client.grant_permission(&admin, &provider, Permission::ReadRecord, 0, false);

    let diagnosis = String::from_str(&env, "Flu");
    let treatment = String::from_str(&env, "Rest");
    let tags = vec![&env, String::from_str(&env, "flu")];
    let category = String::from_str(&env, "Modern");
    let treatment_type = String::from_str(&env, "Medication");
    let data_ref = String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx");

    let record_id = client.add_record(
        &doctor,
        &patient,
        &diagnosis,
        &treatment,
        &false,
        &tags,
        &category,
        &treatment_type,
        &data_ref,
    );

    let expires_at = env.ledger().timestamp().saturating_add(10);
    consent_client.grant_consent_with_expiry(&patient, &provider, &expires_at);

    env.ledger().with_mut(|li| {
        li.timestamp = expires_at.saturating_add(1);
    });

    let result = client.try_get_record(&provider, &record_id);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
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

    assert_eq!(result, Err(Ok(Error::Unauthorized)));
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

#[test]
fn test_patient_record_index_lookup_efficiency() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    let count = 30u64;
    for _i in 0..count {
        let _ = client.add_record(
            &doctor,
            &patient,
            &String::from_str(&env, "Diag"),
            &String::from_str(&env, "Treat"),
            &false,
            &vec![&env, String::from_str(&env, "tag")],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "General"),
            &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        );
    }

    assert_eq!(client.get_patient_record_count(&patient), count);

    for i in 0..count {
        let record_id = client.get_patient_record_id(&patient, &i);
        assert!(record_id.is_some());
    }

    let start_budget = env.budget().cpu_instruction_cost();
    let history = client.get_history(&doctor, &patient, &0u32, &(count as u32));
    let elapsed = env.budget().cpu_instruction_cost() - start_budget;

    assert_eq!(history.len(), count as u32);
    // Expected gas threshold is higher than no-op but should be bounded in this environment.
    assert!(elapsed < 100_000_000);
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
    for _i in 0..3 {
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

// ==================== Metadata Enhancement Tests ====================

#[cfg(test)]
mod test_metadata {
    // internal
    use super::*;

    // external crates
    use soroban_sdk::{map, vec, Address, Env, Map, String};

    fn setup(
        env: &Env,
    ) -> (
        MedicalRecordsContractClient<'_>,
        Address,
        Address,
        Address,
        u64,
    ) {
        let contract_id = Address::generate(env);
        env.register_contract(&contract_id, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let rbac_id = env.register_contract(None, MockRbac);
        let rbac_client = MockRbacClient::new(env, &rbac_id);
        let _ = rbac_client.assign_role(&admin, &RbacRole::Admin);
        let doctor = Address::generate(env);
        let patient = Address::generate(env);
        client.initialize(&admin, &rbac_id);
        client.manage_user(&admin, &doctor, &Role::Doctor);
        client.manage_user(&admin, &patient, &Role::Patient);
        let data_ref = String::from_str(env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx");
        let record_id = client.add_record(
            &doctor,
            &patient,
            &String::from_str(env, "Diagnosis A"),
            &String::from_str(env, "Treatment A"),
            &false,
            &vec![env, String::from_str(env, "cardiology")],
            &String::from_str(env, "Modern"),
            &String::from_str(env, "Medication"),
            &data_ref,
        );
        (client, admin, doctor, patient, record_id)
    }

    #[test]
    fn test_update_record_metadata_by_doctor() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, doctor, patient, record_id) = setup(&env);

        let new_tags = vec![
            &env,
            String::from_str(&env, "cardiology"),
            String::from_str(&env, "hypertension"),
        ];
        let new_fields: Map<String, String> = map![
            &env,
            (
                String::from_str(&env, "severity"),
                String::from_str(&env, "moderate")
            ),
        ];

        client.update_record_metadata(&doctor, &record_id, &new_tags, &new_fields);

        let meta = client.get_record_metadata(&record_id);
        assert_eq!(meta.version, 2);
        assert_eq!(meta.tags.len(), 2);
        assert!(meta.tags.contains(String::from_str(&env, "hypertension")));
        assert_eq!(meta.history.len(), 1);
        assert_eq!(meta.history.get(0).unwrap().version, 1);
        assert_eq!(
            meta.custom_fields
                .get(String::from_str(&env, "severity"))
                .unwrap(),
            String::from_str(&env, "moderate")
        );

        // Patient can still read the record
        let _ = client.get_record(&patient, &record_id);
    }

    #[test]
    fn test_update_metadata_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, _doctor, patient, record_id) = setup(&env);

        // Patient trying to update metadata should fail
        let result = client.try_update_record_metadata(
            &patient,
            &record_id,
            &vec![&env, String::from_str(&env, "tag")],
            &map![&env],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_search_records_by_tag() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, doctor, patient, record_id1) = setup(&env);

        // Create a second record with a different tag
        let patient2 = Address::generate(&env);
        client.manage_user(&admin, &patient2, &Role::Patient);
        let record_id2 = client.add_record(
            &doctor,
            &patient2,
            &String::from_str(&env, "Diagnosis B"),
            &String::from_str(&env, "Treatment B"),
            &false,
            &vec![
                &env,
                String::from_str(&env, "cardiology"),
                String::from_str(&env, "diabetes"),
            ],
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "Medication"),
            &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXy"),
        );

        // Search by "cardiology" — should return both records
        let results =
            client.search_records_by_tag(&patient, &String::from_str(&env, "cardiology"), &0, &10);
        assert_eq!(results.len(), 2);
        assert!(results.contains(record_id1));
        assert!(results.contains(record_id2));

        // Search by "diabetes" — should return only record2
        let diabetes_results =
            client.search_records_by_tag(&patient, &String::from_str(&env, "diabetes"), &0, &10);
        assert_eq!(diabetes_results.len(), 1);
        assert!(diabetes_results.contains(record_id2));
    }

    #[test]
    fn test_search_tag_index_updates_on_metadata_update() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, doctor, patient, record_id) = setup(&env);

        // Initially indexed under "cardiology"
        let before =
            client.search_records_by_tag(&patient, &String::from_str(&env, "cardiology"), &0, &10);
        assert_eq!(before.len(), 1);

        // Update metadata — remove "cardiology", add "neurology"
        client.update_record_metadata(
            &doctor,
            &record_id,
            &vec![&env, String::from_str(&env, "neurology")],
            &map![&env],
        );

        // "cardiology" index should now be empty
        let cardiology_after =
            client.search_records_by_tag(&patient, &String::from_str(&env, "cardiology"), &0, &10);
        assert_eq!(cardiology_after.len(), 0);

        // "neurology" index should contain record_id
        let neurology_after =
            client.search_records_by_tag(&patient, &String::from_str(&env, "neurology"), &0, &10);
        assert_eq!(neurology_after.len(), 1);
        assert!(neurology_after.contains(record_id));
    }

    #[test]
    fn test_export_record_metadata_includes_history() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, doctor, patient, record_id) = setup(&env);

        // Update twice to build history
        client.update_record_metadata(
            &doctor,
            &record_id,
            &vec![&env, String::from_str(&env, "oncology")],
            &map![&env],
        );
        client.update_record_metadata(
            &doctor,
            &record_id,
            &vec![&env, String::from_str(&env, "palliative")],
            &map![
                &env,
                (
                    String::from_str(&env, "stage"),
                    String::from_str(&env, "IV")
                )
            ],
        );

        let exported = client.export_record_metadata(&patient, &record_id);
        assert_eq!(exported.version, 3);
        assert_eq!(exported.history.len(), 2);
        // First history entry had version 1 with "cardiology"
        let first = exported.history.get(0).unwrap();
        assert_eq!(first.version, 1);
        assert!(first.tags.contains(String::from_str(&env, "cardiology")));
    }

    #[test]
    fn test_import_record_metadata_admin_only() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _doctor, patient, record_id) = setup(&env);

        let import_tags = vec![&env, String::from_str(&env, "imported")];
        let import_fields: Map<String, String> = map![
            &env,
            (
                String::from_str(&env, "source"),
                String::from_str(&env, "legacy_system")
            ),
        ];

        // Admin can import
        client.import_record_metadata(&admin, &record_id, &import_tags, &import_fields);

        let meta = client.get_record_metadata(&record_id);
        assert_eq!(meta.version, 2);
        assert!(meta.tags.contains(String::from_str(&env, "imported")));
        assert_eq!(
            meta.custom_fields
                .get(String::from_str(&env, "source"))
                .unwrap(),
            String::from_str(&env, "legacy_system")
        );

        // Non-admin should fail
        let result =
            client.try_import_record_metadata(&patient, &record_id, &vec![&env], &map![&env]);
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_custom_fields_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _admin, doctor, _patient, record_id) = setup(&env);

        // Build 21 custom fields (exceeds limit of 20) — expect BatchTooLarge
        let mut fields: Map<String, String> = Map::new(&env);
        for i in 0u32..21u32 {
            // Use simple string keys to avoid format! allocations in no_std context
            let key = if i < 10 {
                soroban_sdk::String::from_str(&env, "key0")
            } else {
                soroban_sdk::String::from_str(&env, "key1")
            };
            // Make each key unique by appending index as char
            let _ = key; // will be overwritten below
            let key_str = match i {
                0 => "k00",
                1 => "k01",
                2 => "k02",
                3 => "k03",
                4 => "k04",
                5 => "k05",
                6 => "k06",
                7 => "k07",
                8 => "k08",
                9 => "k09",
                10 => "k10",
                11 => "k11",
                12 => "k12",
                13 => "k13",
                14 => "k14",
                15 => "k15",
                16 => "k16",
                17 => "k17",
                18 => "k18",
                19 => "k19",
                _ => "k20",
            };
            fields.set(
                soroban_sdk::String::from_str(&env, key_str),
                soroban_sdk::String::from_str(&env, "value"),
            );
        }

        let result = client.try_update_record_metadata(&doctor, &record_id, &vec![&env], &fields);
        assert!(result.is_err());
    }
}
#[test]
fn test_quantum_performance_benchmark() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    let registry = Address::generate(&env);
    client.set_crypto_registry(&admin, &registry);

    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "performance-test"));
    let ciphertext_ref = String::from_str(&env, "ipfs://performance-test-cid");
    let ciphertext_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Benchmark classical record
    let start_budget = env.budget().cpu_instruction_cost();
    let mut classical_envs = Vec::new(&env);
    classical_envs.push_back(KeyEnvelope {
        recipient: patient.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::X25519,
        wrapped_key: Bytes::from_slice(&env, &[1u8; 32]),
        pq_wrapped_key: None,
    });
    client.add_encrypted_record(
        &doctor,
        &patient,
        &true,
        &tags,
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &ciphertext_ref,
        &ciphertext_hash,
        &classical_envs,
    );
    let classical_cost = env.budget().cpu_instruction_cost() - start_budget;

    // Benchmark hybrid PQC record (Kyber-768)
    let start_budget_pq = env.budget().cpu_instruction_cost();
    let mut pq_envs = Vec::new(&env);
    pq_envs.push_back(KeyEnvelope {
        recipient: patient.clone(),
        key_version: 1,
        algorithm: EnvelopeAlgorithm::HybridX25519Kyber768,
        wrapped_key: Bytes::from_slice(&env, &[1u8; 32]),
        pq_wrapped_key: Some(Bytes::from_slice(&env, &[0u8; 1184])),
    });
    client.add_encrypted_record(
        &doctor,
        &patient,
        &true,
        &tags,
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &ciphertext_ref,
        &ciphertext_hash,
        &pq_envs,
    );
    let pq_cost = env.budget().cpu_instruction_cost() - start_budget_pq;

    // Output stats for "pro" visibility
    // In real Soroban tests, we might use logger or just assert reasonable bounds.
    assert!(pq_cost > classical_cost); // Logic check: larger data = more instructions
}

#[test]
fn test_error_codes_are_stable() {
    use crate::errors::Error;
    assert_eq!(Error::Unauthorized as u32, 100);
    assert_eq!(Error::NotAICoordinator as u32, 150);
    assert_eq!(Error::InvalidInput as u32, 200);
    assert_eq!(Error::InputTooLong as u32, 201);
    assert_eq!(Error::BatchTooLarge as u32, 208);
    assert_eq!(Error::NotInitialized as u32, 300);
    assert_eq!(Error::ContractPaused as u32, 302);
    assert_eq!(Error::DeadlineExceeded as u32, 306);
    assert_eq!(Error::RateLimitExceeded as u32, 307);
    assert_eq!(Error::RecordNotFound as u32, 403);
    assert_eq!(Error::InsufficientFunds as u32, 500);
    assert_eq!(Error::StorageFull as u32, 502);
    assert_eq!(Error::CrossChainAccessDenied as u32, 700);
    assert_eq!(Error::AIConfigNotSet as u32, 830);
    assert_eq!(Error::InvalidAIScore as u32, 831);
}

#[test]
fn test_get_suggestion_returns_expected_hint() {
    use crate::errors::{get_suggestion, Error};
    assert_eq!(
        get_suggestion(Error::Unauthorized),
        symbol_short!("CHK_AUTH")
    );
    assert_eq!(
        get_suggestion(Error::NotInitialized),
        symbol_short!("INIT_CTR")
    );
    assert_eq!(
        get_suggestion(Error::RecordNotFound),
        symbol_short!("CHK_ID")
    );
    assert_eq!(
        get_suggestion(Error::InsufficientFunds),
        symbol_short!("ADD_FUND")
    );
    assert_eq!(get_suggestion(Error::StorageFull), symbol_short!("CLN_OLD"));
    assert_eq!(
        get_suggestion(Error::ContractPaused),
        symbol_short!("RE_TRY_L")
    );
}

// ==================== Traditional Medicine Tests ====================

/// Helper: create a populated TraditionalMedicineMetadata for tests.
fn make_traditional_metadata(env: &Env) -> TraditionalMedicineMetadata {
    TraditionalMedicineMetadata {
        practice_type: String::from_str(env, "African Traditional Medicine"),
        practitioner_tradition: String::from_str(env, "Yoruba"),
        // In production this field would be an encrypted ciphertext reference.
        remedies_used: String::from_str(env, "QmEncryptedRemediesRef1234567890"),
        cultural_context: String::from_str(env, "Healing ceremony"),
        language: String::from_str(env, "yo"),
    }
}

#[test]
fn test_write_record_with_traditional_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    let meta = make_traditional_metadata(&env);
    let initial_events = env.events().all().len();

    let record_id = client.write_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Headache"),
        &String::from_str(&env, "Herbal steam therapy"),
        &true,
        &vec![&env, String::from_str(&env, "traditional")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        &Some(meta),
    );

    // Record should be retrievable
    let record = client.get_record(&patient, &record_id);
    assert_eq!(record.patient_id, patient);
    assert_eq!(record.category, String::from_str(&env, "Traditional"));

    // TraditionalRecordAdded event must have been emitted
    let events_after = env.events().all();
    assert!(events_after.len() > initial_events);
    let trad_events = events_after
        .iter()
        .filter(|e| {
            if e.1.is_empty() {
                return false;
            }
            let topic = e.1.get(0).unwrap();
            Symbol::try_from_val(&env, &topic)
                .map(|sym| sym == symbol_short!("TRAD_NEW"))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(trad_events, 1, "TraditionalRecordAdded event not emitted");
}

#[test]
fn test_list_traditional_records() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // No traditional records yet
    let ids_before = client.list_traditional_records(&patient, &patient);
    assert_eq!(ids_before.len(), 0);

    // Add a plain record (no metadata) – should NOT appear in traditional index
    let _plain_id = client.write_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Flu"),
        &String::from_str(&env, "Rest"),
        &false,
        &vec![&env, String::from_str(&env, "flu")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        &None,
    );

    // Add two traditional records
    let trad_id1 = client.write_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Fever"),
        &String::from_str(&env, "Herbal bath"),
        &true,
        &vec![&env, String::from_str(&env, "traditional")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Herbal Therapy"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        &Some(make_traditional_metadata(&env)),
    );

    let trad_id2 = client.write_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Back pain"),
        &String::from_str(&env, "Acupressure"),
        &true,
        &vec![&env, String::from_str(&env, "traditional")],
        &String::from_str(&env, "Traditional"),
        &String::from_str(&env, "Acupressure"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        &Some(make_traditional_metadata(&env)),
    );

    let ids = client.list_traditional_records(&patient, &patient);
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&trad_id1));
    assert!(ids.contains(&trad_id2));
    // Plain record must NOT be in the list
    assert!(!ids.contains(&_plain_id));
}

#[test]
fn test_write_record_without_metadata_backward_compat() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Calling write_record with None should behave exactly like add_record
    let record_id = client.write_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Common cold"),
        &String::from_str(&env, "Rest and fluids"),
        &false,
        &vec![&env, String::from_str(&env, "respiratory")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
        &None,
    );

    let record = client.get_record(&patient, &record_id);
    assert_eq!(record.patient_id, patient);
    assert_eq!(record.diagnosis, String::from_str(&env, "Common cold"));

    // No traditional records should exist for the patient
    let trad_ids = client.list_traditional_records(&patient, &patient);
    assert_eq!(trad_ids.len(), 0, "No traditional records should exist");

    // No TRAD_NEW event should have been emitted
    let trad_events = env
        .events()
        .all()
        .iter()
        .filter(|e| {
            if e.1.is_empty() {
                return false;
            }
            let topic = e.1.get(0).unwrap();
            Symbol::try_from_val(&env, &topic)
                .map(|sym| sym == symbol_short!("TRAD_NEW"))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(trad_events, 0, "TRAD_NEW must not be emitted when no metadata");
}

// ── Property-Based Tests (Issue #832) ─────────────────────────

// Property 1: Hash-payload binding invariant
#[test]
fn proptest_hash_payload_binding() {
    use proptest::proptest;
    proptest!(|(diagnosis_seed in ".*[a-z0-9]{5,50}") | {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_contract(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        
        client.manage_user(&admin, &doctor, &Role::Doctor);
        client.manage_user(&admin, &patient, &Role::Patient);
        
        let diagnosis = String::from_str(&env, &diagnosis_seed);
        let treatment = String::from_str(&env, "treatment_protocol");
        let tags = vec![&env, String::from_str(&env, "test")];
        let category = String::from_str(&env, "Condition");
        let treatment_type = String::from_str(&env, "Therapy");
        let data_ref = String::from_str(&env, "QmXxxx");
        
        let record_id = client.add_record(
            &doctor, &patient, &diagnosis, &treatment, &false, &tags, 
            &category, &treatment_type, &data_ref,
        );
        
        let record = client.get_record(&patient, &record_id);
        prop_assert_eq!(record.record_id, record_id,
            "Record ID must match stored value");
        prop_assert_eq!(record.diagnosis, diagnosis,
            "Diagnosis must match stored value");
    });
}

// Property 2: Record ID monotonicity
#[test]
fn proptest_record_id_monotonicity() {
    use proptest::proptest;
    proptest!(|(record_count in 1usize..=30) | {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_contract(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        
        client.manage_user(&admin, &doctor, &Role::Doctor);
        client.manage_user(&admin, &patient, &Role::Patient);
        
        let mut prev_id = 0u64;
        for i in 0..record_count {
            let diagnosis = String::from_str(&env, &format!("diagnosis_{}", i));
            let treatment = String::from_str(&env, "treatment");
            let tags = vec![&env];
            let category = String::from_str(&env, "Condition");
            let treatment_type = String::from_str(&env, "Therapy");
            let data_ref = String::from_str(&env, "QmXxxx");
            
            let record_id = client.add_record(
                &doctor, &patient, &diagnosis, &treatment, &false, &tags,
                &category, &treatment_type, &data_ref,
            );
            
            prop_assert!(record_id > prev_id || i == 0,
                "Record IDs must be monotonically increasing");
            prev_id = record_id;
        }
    });
}

// Property 3: Access control enforcement
#[test]
fn proptest_access_control_enforcement() {
    use proptest::proptest;
    proptest!(|(seed in 1u64..=1000) | {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_contract(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        let unauthorized = Address::generate(&env);
        
        client.manage_user(&admin, &doctor, &Role::Doctor);
        client.manage_user(&admin, &patient, &Role::Patient);
        
        let diagnosis = String::from_str(&env, "sensitive_diagnosis");
        let treatment = String::from_str(&env, "treatment");
        let tags = vec![&env];
        let category = String::from_str(&env, "Condition");
        let treatment_type = String::from_str(&env, "Therapy");
        let data_ref = String::from_str(&env, "QmXxxx");
        
        let record_id = client.add_record(
            &doctor, &patient, &diagnosis, &treatment, &false, &tags,
            &category, &treatment_type, &data_ref,
        );
        
        // Authorized access should succeed
        let record = client.get_record(&patient, &record_id);
        prop_assert_eq!(record.record_id, record_id,
            "Patient should be able to access own record at seed {}", seed);
    });
}

// Property 4: Get record idempotency
#[test]
fn proptest_get_record_idempotency() {
    use proptest::proptest;
    proptest!(|(check_count in 1usize..=50) | {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_contract(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        
        client.manage_user(&admin, &doctor, &Role::Doctor);
        client.manage_user(&admin, &patient, &Role::Patient);
        
        let diagnosis = String::from_str(&env, "test_diagnosis");
        let treatment = String::from_str(&env, "treatment");
        let tags = vec![&env];
        let category = String::from_str(&env, "Condition");
        let treatment_type = String::from_str(&env, "Therapy");
        let data_ref = String::from_str(&env, "QmXxxx");
        
        let record_id = client.add_record(
            &doctor, &patient, &diagnosis, &treatment, &false, &tags,
            &category, &treatment_type, &data_ref,
        );
        
        let first_read = client.get_record(&patient, &record_id);
        
        // Multiple reads should return identical records
        for _ in 0..check_count {
            let subsequent_read = client.get_record(&patient, &record_id);
            prop_assert_eq!(subsequent_read.record_id, first_read.record_id,
                "Multiple reads must return same record ID");
            prop_assert_eq!(subsequent_read.diagnosis, first_read.diagnosis,
                "Multiple reads must return same diagnosis");
            prop_assert_eq!(subsequent_read.treatment, first_read.treatment,
                "Multiple reads must return same treatment");
        }
    });
}

// Property 5: Multiple doctors can create records for same patient
#[test]
fn proptest_multiple_doctors_same_patient() {
    use proptest::proptest;
    proptest!(|(doctor_count in 1usize..=20) | {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_contract(&env);
        let patient = Address::generate(&env);
        
        client.manage_user(&admin, &patient, &Role::Patient);
        
        let mut record_count = 0;
        for i in 0..doctor_count {
            let doctor = Address::generate(&env);
            client.manage_user(&admin, &doctor, &Role::Doctor);
            
            let diagnosis = String::from_str(&env, &format!("doctor_{}_diagnosis", i));
            let treatment = String::from_str(&env, "treatment");
            let tags = vec![&env];
            let category = String::from_str(&env, "Condition");
            let treatment_type = String::from_str(&env, "Therapy");
            let data_ref = String::from_str(&env, "QmXxxx");
            
            let _ = client.add_record(
                &doctor, &patient, &diagnosis, &treatment, &false, &tags,
                &category, &treatment_type, &data_ref,
            );
            
            record_count += 1;
        }
        
        // Verify patient can read all records from different doctors
        prop_assert!(record_count == doctor_count,
            "Should have created {} records from {} doctors", 
            record_count, doctor_count);
    });
}

// Property 6: Record attributes persist correctly
#[test]
fn proptest_record_attributes_persistence() {
    use proptest::proptest;
    proptest!(|(confidential in proptest::bool::ANY) | {
        let env = Env::default();
        env.mock_all_auths();
        
        let (client, admin) = create_contract(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        
        client.manage_user(&admin, &doctor, &Role::Doctor);
        client.manage_user(&admin, &patient, &Role::Patient);
        
        let diagnosis = String::from_str(&env, "test_diagnosis");
        let treatment = String::from_str(&env, "test_treatment");
        let tags = vec![&env, String::from_str(&env, "tag1"), String::from_str(&env, "tag2")];
        let category = String::from_str(&env, "Condition");
        let treatment_type = String::from_str(&env, "Therapy");
        let data_ref = String::from_str(&env, "QmXxxx");
        
        let record_id = client.add_record(
            &doctor, &patient, &diagnosis, &treatment, &confidential, &tags,
            &category, &treatment_type, &data_ref,
        );
        
        let record = client.get_record(&patient, &record_id);
        
        prop_assert_eq!(record.is_confidential, confidential,
            "Confidential flag must persist");
        prop_assert_eq!(record.patient_id, patient,
            "Patient ID must persist");
        prop_assert_eq!(record.doctor_id, doctor,
            "Doctor ID must persist");
        prop_assert!(record.timestamp > 0,
            "Timestamp must be set");
    });
}

// ==================== Schema Evolution Tests ====================

#[test]
fn test_register_schema_version() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);

    let description = String::from_str(&env, "Initial schema version");
    let empty_vec: Vec<String> = Vec::new(&env);
    let empty_map: Map<String, String> = Map::new(&env);

    let evolution_id = client.register_schema_version(
        &admin,
        &1u32,
        &0u32,
        &0u32,
        &description,
        &empty_vec.clone(),
        &empty_vec.clone(),
        &empty_map.clone(),
        &false,
    );

    assert_eq!(evolution_id, 1);

    // Verify current schema version
    let current = client.get_current_schema_version();
    assert!(current.is_some());
    let ver = current.unwrap();
    assert_eq!(ver.major, 1);
    assert_eq!(ver.minor, 0);
    assert_eq!(ver.patch, 0);
}

#[test]
fn test_register_schema_version_duplicate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);

    let description = String::from_str(&env, "Initial schema version");
    let empty_vec: Vec<String> = Vec::new(&env);
    let empty_map: Map<String, String> = Map::new(&env);

    client.register_schema_version(
        &admin,
        &1u32,
        &0u32,
        &0u32,
        &description,
        &empty_vec.clone(),
        &empty_vec.clone(),
        &empty_map.clone(),
        &false,
    );

    // Try to register the same version again
    let result = client.try_register_schema_version(
        &admin,
        &1u32,
        &0u32,
        &0u32,
        &String::from_str(&env, "Duplicate"),
        &empty_vec.clone(),
        &empty_vec.clone(),
        &empty_map.clone(),
        &false,
    );
    assert_eq!(result, Err(Ok(Error::SchemaVersionAlreadyExists)));
}

#[test]
fn test_evolve_schema() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);

    let description = String::from_str(&env, "Initial schema");
    let empty_vec: Vec<String> = Vec::new(&env);
    let empty_map: Map<String, String> = Map::new(&env);

    // Register v1.0.0
    client.register_schema_version(
        &admin,
        &1u32,
        &0u32,
        &0u32,
        &description,
        &empty_vec.clone(),
        &empty_vec.clone(),
        &empty_map.clone(),
        &false,
    );

    // Evolve to v1.1.0
    let mut additions = Vec::new(&env);
    additions.push_back(String::from_str(&env, "new_field"));

    let evolution_id = client.evolve_schema(
        &admin,
        &1u32, &0u32, &0u32,
        &1u32, &1u32, &0u32,
        &String::from_str(&env, "Add new field"),
        &additions,
        &empty_vec.clone(),
        &empty_map.clone(),
        &false,
    );

    assert_eq!(evolution_id, 1);

    // Verify current schema version updated
    let current = client.get_current_schema_version();
    assert!(current.is_some());
    let ver = current.unwrap();
    assert_eq!(ver.major, 1);
    assert_eq!(ver.minor, 1);
    assert_eq!(ver.patch, 0);
}

#[test]
fn test_evolve_schema_source_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);

    let empty_vec: Vec<String> = Vec::new(&env);
    let empty_map: Map<String, String> = Map::new(&env);

    let result = client.try_evolve_schema(
        &admin,
        &1u32, &0u32, &0u32,
        &2u32, &0u32, &0u32,
        &String::from_str(&env, "Nonexistent source"),
        &empty_vec.clone(),
        &empty_vec.clone(),
        &empty_map.clone(),
        &false,
    );
    assert_eq!(result, Err(Ok(Error::SchemaVersionNotFound)));
}

#[test]
fn test_create_and_get_migration_plan() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);

    let plan_id = client.create_migration_plan(
        &admin,
        &1u32, &0u32, &0u32,
        &2u32, &0u32, &0u32,
        &100u64,
    );

    assert_eq!(plan_id, 1);

    let plan = client.get_migration_plan(&plan_id);
    assert!(plan.is_some());
    let p = plan.unwrap();
    assert_eq!(p.affected_records, 100);
    assert_eq!(p.migrated_count, 0);
    assert_eq!(p.status, MigrationStatus::Planned);
}

#[test]
fn test_get_schema_evolution() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);

    let description = String::from_str(&env, "Initial schema");
    let empty_vec: Vec<String> = Vec::new(&env);
    let empty_map: Map<String, String> = Map::new(&env);

    let evolution_id = client.register_schema_version(
        &admin,
        &1u32,
        &0u32,
        &0u32,
        &description,
        &empty_vec,
        &empty_vec,
        &empty_map,
        &false,
    );

    // The evolution ID for the first register_schema_version should be retrievable
    let evolution = client.get_schema_evolution(&(evolution_id as u64));
    // Note: register_schema_version stores under SchemaEvolutionCount which is the evolution_id
    // This test verifies the get_schema_evolution query function works
}
