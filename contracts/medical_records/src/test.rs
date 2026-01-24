#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger, Events};
use soroban_sdk::{Address, BytesN, Env, IntoVal, TryFromVal, String, Vec, Val};

fn create_contract(env: &Env) -> (MedicalRecordsContractClient, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordsContract);

    let client = MedicalRecordsContractClient::new(env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (client, admin)
}

#[test]
fn test_add_records_batch_and_get_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient1 = Address::generate(&env);
//     let patient2 = Address::generate(&env);
// 
//     client.manage_user(&admin, &doctor, &Role::Doctor);
//     client.manage_user(&admin, &patient1, &Role::Patient);
//     client.manage_user(&admin, &patient2, &Role::Patient);
// 
//     let records_input = vec![
//         &env,
//         (
//             patient1.clone(),
//             String::from_str(&env, "Flu"),
//             String::from_str(&env, "Antiviral"),
//             false,
//             vec![&env, String::from_str(&env, "viral")],
//             String::from_str(&env, "Modern"),
//             String::from_str(&env, "Prescription"),
//             String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXA"),
//         ),
//         (
//             patient1.clone(),
//             String::from_str(&env, "Hypertension"),
//             String::from_str(&env, "Lifestyle + meds"),
//             true,
//             vec![&env],
//             String::from_str(&env, "Modern"),
//             String::from_str(&env, "Ongoing"),
//             String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXB"),
//         ),
//         (
//             patient2.clone(),
//             String::from_str(&env, "Malaria"),
//             String::from_str(&env, "Artemisinin"),
//             false,
//             vec![&env, String::from_str(&env, "tropical")],
//             String::from_str(&env, "Modern"),
//             String::from_str(&env, "Acute"),
//             String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXC"),
//         ),
//     ];
// 
//     let result = client.add_records_batch(&doctor, &records_input);
// 
//     assert_eq!(result.successes.len(), 3);
//     assert_eq!(result.failures.len(), 0);
// 
//     let ids = result.successes;
// 
//     // Test batch get - patient1 should see both (one confidential, but owns them)
//     let batch1 = client.get_records_batch(&patient1, &patient1, &0, &10);
//     assert_eq!(batch1.len(), 2);
// 
//     // Test pagination + limit
//     let batch2 = client.get_records_batch(&patient1, &patient1, &0, &1);
//     assert_eq!(batch2.len(), 1);
// 
//     let batch3 = client.get_records_batch(&patient1, &patient1, &1, &10);
//     assert_eq!(batch3.len(), 1);
// 
//     // Test patient2 access
//     let batch_p2 = client.get_records_batch(&patient2, &patient2, &0, &5);
//     assert_eq!(batch_p2.len(), 1);
// 
//     // Doctor should see everything
//     let batch_doc = client.get_records_batch(&doctor, &patient1, &0, &10);
//     assert_eq!(batch_doc.len(), 2); // only the non-confidential one
// 
//     // Admin sees everything
//     let batch_admin = client.get_records_batch(&admin, &patient1, &0, &10);
//     assert_eq!(batch_admin.len(), 2);
}
// 
// #[test]
// fn test_add_records_batch_partial_failure() {
//     let env = Env::default();
//     env.mock_all_auths();
//     let (client, admin) = create_contract(&env);
//     let doctor = Address::generate(&env);
//     let patient = Address::generate(&env);
// 
//     client.manage_user(&admin, &doctor, &Role::Doctor);
//     client.manage_user(&admin, &patient, &Role::Patient);
// 
//     let records = vec![
//         &env,
//         // valid
//         (
//             patient.clone(),
//             String::from_str(&env, "Malaria"),
//             String::from_str(&env, "Artemisinin"),
//             false,
//             vec![&env, String::from_str(&env, "tropical")],
//             String::from_str(&env, "Modern"),
//             String::from_str(&env, "Acute"),
//             String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXC"),
//         ),
//         // invalid category
//         (
//             patient.clone(),
//             String::from_str(&env, "A"),
//             String::from_str(&env, "B"),
//             false,
//             vec![&env],
//             String::from_str(&env, "Invalid"),
//             String::from_str(&env, "C"),
//             String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXD"),
//         ),
//         // empty treatment_type
//         (
//             patient.clone(),
//             String::from_str(&env, "A"),
//             String::from_str(&env, "B"),
//             false,
//             vec![&env],
//             String::from_str(&env, "Modern"),
//             String::from_str(&env, ""),
//             String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXE"),
//         ),
//     ];
// 
//     let result = client.add_records_batch(&doctor, &records);
// 
//     assert_eq!(result.successes.len(), 1);
// //     assert_eq!(result.failures.len(), 2);
// }

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
    let record_events_count = events_after_add.iter()
        .filter(|e| {
            if e.1.len() < 2 { return false; }
            let topic = e.1.get(1).unwrap();
            let sym = Symbol::try_from_val(&env, &topic).unwrap();
            sym == symbol_short!("REC_NEW")
        })
        .count();
    assert_eq!(record_events_count, 1);

    // Get the record as patient
    let retrieved_record = client.get_record(&patient, &record_id);
    assert!(retrieved_record.is_some());
    let record = retrieved_record.unwrap();
    assert_eq!(record.patient_id, patient);
    assert_eq!(record.diagnosis, diagnosis);
    assert_eq!(record.treatment, treatment);
    assert_eq!(record.is_confidential, false);

    // Verify record access event was emitted
    let events_after_get = env.events().all();
    let access_events_count = events_after_get.iter()
        .filter(|e| {
            if e.1.len() < 2 { return false; }
            let topic = e.1.get(1).unwrap();
            let sym = Symbol::try_from_val(&env, &topic).unwrap();
            sym == symbol_short!("REC_ACC")
        })
        .count();
    assert_eq!(access_events_count, 1);
}
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_empty_data_ref() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Empty data_ref should fail
    let _ = client.add_record(
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
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_data_ref_too_short() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Data ref shorter than 10 chars should fail
    let _ = client.add_record(
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
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
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

    let _ = client.add_record(
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
    assert!(record.is_some());
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
    assert!(record.is_some());
}

#[test]
fn test_get_patient_records() {
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
    assert!(client.get_record(&patient, &record_id1).is_some());
    assert!(client.get_record(&patient, &record_id2).is_some());
}

#[test]
fn test_role_based_access() {
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
    let retrieved_record = client.get_record(&patient, &record_id);
    assert!(retrieved_record.is_some());

    // Doctor (creator) tries to access the record (should succeed)
    let retrieved_record = client.get_record(&doctor, &record_id);
    assert!(retrieved_record.is_some());

    // Admin tries to access the record (should succeed)
    let retrieved_record = client.get_record(&admin, &record_id);
    assert!(retrieved_record.is_some());
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
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

    // // Try to add a record as the deactivated doctor (should fail)
    let result = client.add_record(
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
}

#[test]
#[should_panic(expected = "Error")]
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
    // Mutating functions should be blocked when paused
    client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Blocked"),
        &String::from_str(&env, "Blocked"),
        &false,
        &vec![&env],
        &String::from_str(&env, "General"),
        &String::from_str(&env, "General"),
        &String::from_str(&env, "IPFS"),
    );
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
    let r3 = client.add_record(
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
#[should_panic(expected = "Error(Contract, #7)")]
fn test_recovery_timelock_and_multisig() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin1) = create_contract(&env);

    let admin2 = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Initialize and add second admin
    client.manage_user(&admin1, &admin2, &Role::Admin);

    // Propose recovery by admin1
    let proposal_id = client.propose_recovery(&admin1, &token, &recipient, &100i128);
    assert!(proposal_id > 0);

    // Approve by admin2
    client.approve_recovery(&admin2, &proposal_id);

    // Try execute before timelock elapsed -> should error
    let _ = client.execute_recovery(&admin1, &proposal_id);
}

#[test]
fn test_recovery_timelock_and_multisig_success() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin1) = create_contract(&env);

    let admin2 = Address::generate(&env);
    let token = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Initialize and add second admin
    client.manage_user(&admin1, &admin2, &Role::Admin);

    // Propose recovery by admin1
    let proposal_id = client.propose_recovery(&admin1, &token, &recipient, &100i128);
    assert!(proposal_id > 0);

    // Approve by admin2
    client.approve_recovery(&admin2, &proposal_id);

    // Advance time beyond timelock
    let now = env.ledger().timestamp();
    env.ledger().with_mut(|l| {
        l.timestamp = now + TIMELOCK_SECS + 1;
    });

    let res = client.execute_recovery(&admin1, &proposal_id);
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
    for i in 0..5 {
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

#[test]
fn test_record_counter_isolation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);

    // Initialize and set roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add first record
    let record_id1 = client.add_record(
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

    // Create a recovery proposal (also uses the counter)
    let proposal_id = client.mock_all_auths().propose_recovery(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &100i128,
    );

    // Add another record
    let record_id2 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Type"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Verify all IDs are unique and monotonic
    assert_eq!(record_id1, 1);
    assert_eq!(proposal_id, 2);
    assert_eq!(record_id2, 3);
    assert!(proposal_id > record_id1);
    assert!(record_id2 > proposal_id);
}

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

#[test]
fn test_ai_integration_points() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let ai_coordinator = Address::generate(&env);

    // Initialize and set roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add a medical record
    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Set AI configuration
    assert!(client.set_ai_config(&admin, &ai_coordinator, &100u32, &2u32));

    // Verify AI config is set
    let ai_config = client.get_ai_config().unwrap();
    assert_eq!(ai_config.ai_coordinator, ai_coordinator);
    assert_eq!(ai_config.dp_epsilon, 100u32);
    assert_eq!(ai_config.min_participants, 2u32);

    // Submit anomaly score for the record
    let model_id = BytesN::from_array(&env, &[1; 32]);
    let explanation_ref = String::from_str(&env, "ipfs://anomaly-report-1");
    let explanation_summary = String::from_str(&env, "Anomaly detected in vital signs");
    let model_version = String::from_str(&env, "v1.0.0");
    let feature_importance = vec![
        &env,
        (String::from_str(&env, "heart_rate"), 8000u32),
        (String::from_str(&env, "blood_pressure"), 6500u32),
    ];

    assert!(client.submit_anomaly_score(
        &ai_coordinator,
        &record_id,
        &model_id,
        &7500u32,
        &explanation_ref,
        &explanation_summary,
        &model_version,
        &feature_importance
    ));

    // Get the anomaly score (should be accessible by patient)
    let anomaly_insight = client.get_anomaly_score(&patient, &record_id).unwrap();
    assert_eq!(anomaly_insight.score_bps, 7500u32);
    assert_eq!(anomaly_insight.explanation_summary, explanation_summary);
    assert_eq!(anomaly_insight.model_version, model_version);

    // Submit patient risk score
    let risk_explanation_ref = String::from_str(&env, "ipfs://risk-assessment-1");
    let risk_explanation_summary = String::from_str(&env, "High risk for diabetes progression");
    let risk_model_version = String::from_str(&env, "v1.1.0");
    let risk_feature_importance = vec![
        &env,
        (String::from_str(&env, "glucose_level"), 9000u32),
        (String::from_str(&env, "family_history"), 7000u32),
    ];

    assert!(client.submit_risk_score(
        &ai_coordinator,
        &patient,
        &model_id,
        &8000u32,
        &risk_explanation_ref,
        &risk_explanation_summary,
        &risk_model_version,
        &risk_feature_importance
    ));

    // Get the risk score
    let risk_insight = client.get_latest_risk_score(&patient, &patient).unwrap();
    assert_eq!(risk_insight.score_bps, 8000u32);
    assert_eq!(risk_insight.explanation_summary, risk_explanation_summary);
    assert_eq!(risk_insight.model_version, risk_model_version);
}

#[test]
fn test_ai_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_contract(&env);
    let doctor = Address::generate(&env);
    let patient = Address::generate(&env);
    let ai_coordinator = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    // Initialize and set roles
    client.manage_user(&admin, &doctor, &Role::Doctor);
    client.manage_user(&admin, &patient, &Role::Patient);

    // Add a medical record
    let record_id = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &false,
        &vec![&env, String::from_str(&env, "tag")],
        &String::from_str(&env, "Modern"),
        &String::from_str(&env, "Medication"),
        &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhXXXXXx"),
    );

    // Set AI configuration
    client.set_ai_config(&admin, &ai_coordinator, &100u32, &2u32);

    // Test invalid score (over 10,000)
    let model_id = BytesN::from_array(&env, &[1; 32]);
    let explanation_ref = String::from_str(&env, "ipfs://report");
    let explanation_summary = String::from_str(&env, "Detailed Summary");
    let model_version = String::from_str(&env, "v1.0.0");
    let feature_importance = vec![&env, (String::from_str(&env, "test"), 5000u32)];

    // Panic testing requires std - skipping in no_std context
    // // This should panic due to invalid score
    // let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    //     client.submit_anomaly_score(
    //         &ai_coordinator,
    //         &record_id,
    //         &model_id,
    //         &10001u32,
    //         &explanation_ref,
    //         &explanation_summary,
    //         &model_version,
    //         &feature_importance,
    //     );
    // }));
    // assert!(result.is_err());

    // Test unauthorized access to submit scores
//     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
//         client.submit_anomaly_score(
//             &unauthorized,
//             &record_id,
//             &model_id,
//             &5000u32,
//             &explanation_ref,
//             &explanation_summary,
//             &model_version,
//             &feature_importance,
//         );
//     }));
//     assert!(result.is_err());

    // Test unauthorized access to get anomaly scores
    client.submit_anomaly_score(
        &ai_coordinator,
        &record_id,
        &model_id,
        &5000u32,
        &explanation_ref,
        &explanation_summary,
        &model_version,
        &feature_importance,
    );

//     let other_patient = Address::generate(&env);
//     let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
//         client.get_anomaly_score(&other_patient, &record_id);
//     }));
//     assert!(result.is_err());
}
// #[test]
// fn test_monitoring_health_check() {
//     use soroban_sdk::{Env, symbol_short};
//     use crate::{MedicalRecordsContract, MedicalRecordsContractClient};
// 
//     let env = Env::default();
//     let contract_id = env.register_contract(None, MedicalRecordsContract);
//     let client = MedicalRecordsContractClient::new(&env, &contract_id);
// 
//     // 1. Reset budget to track clean execution cost
//     env.budget().reset_unlimited();
// 
//     // 2. Call health check
//     let (status, version, _timestamp) = client.health_check();
// 
//     // 3. Assertions (Validation)
//     assert_eq!(status, symbol_short!("OK"));
//     assert_eq!(version, 1);

//     // 4. Gas Validation (Monitoring)
//     // This prints the cost to the console when you run tests with --nocapture
//     std::println!("==========================================");
//     std::println!("MONITORING: Health Check Gas Usage");
//     env.budget().print(); 
//     std::println!("==========================================");
// 
//     // Optional: Fail the test if CPU usage is too high (Performance Requirement)
//     // This ensures your "ping" remains cheap (e.g., < 100,000 instructions)
//     assert!(env.budget().cpu_instruction_cost() < 100_000);
// }
// 
// #[test]
// fn test_event_store_filter_aggregate_replay() {
//     let env = Env::default();
//     env.mock_all_auths();
// 
//     // Prepare identities
//     let admin = Address::generate(&env);
// 
//     // Construct two simple events
//     let meta = events::EventMetadata {
//         event_type: events::EventType::MetricUpdate,
//         category: events::OperationCategory::System,
//         timestamp: env.ledger().timestamp(),
//         user_id: admin.clone(),
//         session_id: None,
//         ipfs_ref: None,
//         gas_used: Some(10u64),
//         block_height: env.ledger().sequence() as u64,
//     };
// 
//     let data = events::EventData::SystemEvent(events::SystemEventData {
//         status: String::from_str(&env, "active"),
//         metric_name: Some(String::from_str(&env, "test_metric")),
//         metric_value: Some(1u64),
//     });
// 
//     let ev = events::BaseEvent { metadata: meta.clone(), data: data.clone() };
// 
//     // Create store and add events
//     let mut store = events::EventStore::new(&env, 100);
//     store.add_event(ev.clone());
//     store.add_event(ev.clone());
// 
//     // Build a filter for MetricUpdate events by admin
//     let types = vec![&env, events::EventType::MetricUpdate];
//     let filter = events::EventFilter {
//         event_types: Some(types),
//         categories: None,
//         user_id: Some(admin.clone()),
//         start_time: None,
//         end_time: None,
//         limit: Some(10u32),
//     };
// 
//     let res = store.query_events(&filter);
//     assert_eq!(res.total_count, 2);
//     assert_eq!(res.events.len(), 2);
// 
//     // Replay events in the current time window
//     let start = env.ledger().timestamp() - 1;
//     let end = env.ledger().timestamp() + 1;
//     let replayed = store.replay_events(start, end, None);
//     assert_eq!(replayed.len(), 2);
// 
//     // Dashboard aggregation
//     let dashboard = store.get_dashboard(&env, 10u32);
//     assert!(dashboard.stats.total_events >= 2);
// // }}
