#![cfg(test)]


use super::*;
use soroban_sdk::testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke};
use soroban_sdk::{log, Address, Env, String, Vec};

extern crate std;
use std::format;

fn create_contract(env: &Env) -> (MedicalRecordsContractClient, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordsContract);

    let client = MedicalRecordsContractClient::new(env, &contract_id);
    let admin = Address::generate(&env);
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
    let record_id = client.add_record(&doctor, &patient, &diagnosis, &treatment,& is_confidential, &tags, &category, &treatment_type);

    // Get the record as patient
    let retrieved_record = client.get_record(&patient, &record_id);
    assert!(retrieved_record.is_some());
    let record = retrieved_record.unwrap();
    assert_eq!(record.patient_id, patient);
    assert_eq!(record.diagnosis, diagnosis);
    assert_eq!(record.treatment, treatment);
    assert_eq!(record.is_confidential, false);
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
        );

    let record_id2 = client.add_record(
        &doctor,
        &patient,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &true,
        &vec![&env,String::from_str(&env, "spiritual")],
        &String::from_str(&env, "Spiritual"),
        &String::from_str(&env, "Prayer"),
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
        &vec![&env,String::from_str(&env, "spiritual")],
        &String::from_str(&env, "Spiritual"),
        &String::from_str(&env, "Prayer"),
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

// #[test]
// fn test_deactivate_user() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let (client, admin) = create_contract(&env);
//     let doctor = Address::generate(&env);
//     let patient = Address::generate(&env);
//     // Admin manages user roles
//     client.manage_user(&admin, &doctor, &Role::Doctor);
//     client.manage_user(&admin, &patient, &Role::Patient);

//     // Deactivate the doctor
//    let res =  client.deactivate_user(&admin, &doctor);

//     let result  = client.add_record(
//         &doctor,
//         &patient,
//         &String::from_str(&env, "Cold"),
//         &String::from_str(&env, "Rest"),
//         &false,
//         &vec![&env, String::from_str(&env, "herbal")],
//         &String::from_str(&env, "Traditional"),
//         &String::from_str(&env, "Herbal Therapy"),
//     );
//     assert!(result.is_err());

//     // Reactivate the doctor
//     assert!(client.manage_user(&admin, &doctor, &Role::Doctor));

//     // Add a record as the reactivated doctor (should succeed)
//     let record_id = client.add_record(
//             &doctor,
//             &patient,
//             &String::from_str(&env, "Cold"),
//             &String::from_str(&env, "Rest"),
//             &false,
//             &vec![&env, String::from_str(&env, "herbal")],
//             &String::from_str(&env, "Traditional"),
//             &String::from_str(&env, "Herbal Therapy"),
//         );
//     assert!(record_id > 0);
// }

// #[test]
// fn test_pause_unpause_blocks_sensitive_functions() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let (client, admin) = create_contract(&env);
//     let doctor = Address::generate(&env);
//     let patient = Address::generate(&env);

//     // Initialize and set up roles
//     client.manage_user(&admin, &doctor, &Role::Doctor);
//     client.manage_user(&admin, &patient, &Role::Patient);

//     // Add a record (not paused)
//     let _record_id = client.add_record(
//             &doctor,
//             &patient,
//             &String::from_str(&env, "Diagnosis"),
//             &String::from_str(&env, "Treatment"),
//             &false,
//             &vec![&env, String::from_str(&env, "herbal")],
//             &String::from_str(&env, "Traditional"),
//             &String::from_str(&env, "Herbal Therapy")
//     );

//     // Pause the contract
//     assert!(client.pause(&admin));

//     // Mutating functions should be blocked when paused
//     let r1 = client.try_manage_user(&admin, &Address::generate(&env), &Role::Doctor);
//     assert!(r1.is_err());
//     let r2 = client.try_add_record(
//             &doctor,
//             &patient,
//             &String::from_str(&env, "Diagnosis2"),
//             &String::from_str(&env, "Treatment2"),
//             &false,
//             &vec![&env, String::from_str(&env, "herbal")],
//             &String::from_str(&env, "Traditional"),
//             &String::from_str(&env, "Herbal Therapy"),
//         );
//     assert!(r2.is_err());

//     // Unpause
//     assert!(client.unpause(&admin));

//     // Now mutating calls should succeed
//     assert!(client.manage_user(&admin, &Address::generate(&env), &Role::Doctor));
//     let r3 = client.add_record(
//             &doctor,
//             &patient,
//             &String::from_str(&env, "Diagnosis3"),
//             &String::from_str(&env, "Treatment3"),
//             &false,
//             &vec![&env, String::from_str(&env, "herbal")],
//             &String::from_str(&env, "Traditional"),
//             &String::from_str(&env, "Herbal Therapy"),
//         );
//     assert!(r3 > 0);
// }

// #[test]
// fn test_recovery_timelock_and_multisig() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let (client, admin1) = create_contract(&env);

//     let admin2 = Address::generate(&env);
//     let token = Address::generate(&env);
//     let recipient = Address::generate(&env);

//     // Initialize and add second admin
//     client.manage_user(&admin1, &admin2, &Role::Admin);

//     // Propose recovery by admin1
//     let proposal_id = client.propose_recovery(&admin1, &token, &recipient, &100i128);
//     assert!(proposal_id > 0);

//     // Approve by admin2
//     assert!(client.approve_recovery(&admin2, &proposal_id));

//     // Try execute before timelock elapsed -> should error
//     let res = client.execute_recovery(&admin1, &proposal_id);
//     // assert!(res.is_err());

//     // // Advance time beyond timelock
//     // let now = env.ledger().timestamp();
//     // env.ledger().with_mut(|l| {
//     //     l.timestamp = now + TIMELOCK_SECS + 1;
//     // });

//     // // Execute should succeed now
//     // assert!(client.execute_recovery(&admin1, &proposal_id));
// }

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
                &String::from_str(&env, &format!("Diagnosis {}", i)),
                &String::from_str(&env, &format!("Treatment {}", i)),
                &false,
                &vec![&env, String::from_str(&env, "tag")],
                &String::from_str(&env, "Modern"),
                &String::from_str(&env, "Type"),
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

    let (client, admin) = create_contract(&env);    let doctor = Address::generate(&env);
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
        );

    // Create a recovery proposal (also uses the counter)
    let proposal_id = client
        .mock_all_auths()
        .propose_recovery(&admin, &Address::generate(&env), &Address::generate(&env), &100i128);

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

    let empty_page = client.mock_all_auths().get_history(&patient, &patient, &3u32, &1u32);
    assert_eq!(empty_page.len(), 0);
}