// tests/integration/mod.rs
use soroban_sdk::{Address, Env, String};

pub mod medical_records_tests {
    use super::*;
    use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient};
    use soroban_sdk::{
        testutils::{Address as _, MockAuth, MockAuthInvoke},
    };

    #[test]
    fn test_full_medical_record_workflow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        // Setup test data
        let doctor = Address::generate(&env);
        let patient_id = String::from_str(&env, "patient_001");
        let doctor_id = String::from_str(&env, "doctor_001");
        let diagnosis = String::from_str(&env, "Hypertension");
        let treatment = String::from_str(&env, "ACE inhibitor medication");

        // Initialize contract
        assert!(client.initialize());

        // Add a medical record
        let record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "add_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .add_record(
                &doctor,
                &patient_id,
                &doctor_id,
                &diagnosis,
                &treatment,
                &false,
            );

        // Verify record was added
        let record = client.get_record(&record_id).expect("Record should exist");
        assert_eq!(record.patient_id, patient_id);
        assert_eq!(record.diagnosis, diagnosis);

        // Update the record
        let new_diagnosis = String::from_str(&env, "Hypertension Stage 2");
        let new_treatment = String::from_str(&env, "Combination therapy");

        let success = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "update_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .update_record(&doctor, &record_id, &new_diagnosis, &new_treatment);

        assert!(success);

        // Verify update
        let updated_record = client.get_record(&record_id).expect("Record should exist");
        assert_eq!(updated_record.diagnosis, new_diagnosis);
        assert_eq!(updated_record.treatment, new_treatment);

        // Check patient records list
        let patient_records = client.get_patient_records(&patient_id);
        assert_eq!(patient_records.len(), 1);
        assert_eq!(patient_records.get(0).unwrap(), record_id);
    }

    #[test]
    fn test_unauthorized_update_fails() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let doctor = Address::generate(&env);
        let unauthorized_user = Address::generate(&env);
        let patient_id = String::from_str(&env, "patient_002");
        let doctor_id = String::from_str(&env, "doctor_002");

        // Add a record as authorized doctor
        let record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "add_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .add_record(
                &doctor,
                &patient_id,
                &doctor_id,
                &String::from_str(&env, "Initial diagnosis"),
                &String::from_str(&env, "Initial treatment"),
                &false,
            );

        // Try to update as unauthorized user
        let success = client
            .mock_auths(&[MockAuth {
                address: &unauthorized_user,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "update_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .update_record(
                &unauthorized_user,
                &record_id,
                &String::from_str(&env, "Unauthorized diagnosis"),
                &String::from_str(&env, "Unauthorized treatment"),
            );

        assert!(!success);
    }

    #[test]
    fn test_multiple_patients() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let doctor = Address::generate(&env);
        let patient1_id = String::from_str(&env, "patient_001");
        let patient2_id = String::from_str(&env, "patient_002");
        let doctor_id = String::from_str(&env, "doctor_001");

        // Add records for different patients
        let _record1 = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "add_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .add_record(
                &doctor,
                &patient1_id,
                &doctor_id,
                &String::from_str(&env, "Diagnosis for patient 1"),
                &String::from_str(&env, "Treatment for patient 1"),
                &false,
            );

        let _record2 = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "add_record",
                    args: (),
                    sub_invokes: &[],
                },
            }])
            .add_record(
                &doctor,
                &patient2_id,
                &doctor_id,
                &String::from_str(&env, "Diagnosis for patient 2"),
                &String::from_str(&env, "Treatment for patient 2"),
                &true,
            );

        // Verify each patient has their own records
        let patient1_records = client.get_patient_records(&patient1_id);
        let patient2_records = client.get_patient_records(&patient2_id);

        assert_eq!(patient1_records.len(), 1);
        assert_eq!(patient2_records.len(), 1);
        assert_ne!(patient1_records.get(0).unwrap(), patient2_records.get(0).unwrap());
    }
}

// tests/unit/mod.rs
#[cfg(test)]
mod unit_tests {
    use soroban_sdk::{Env, String};

    #[test]
    fn test_string_operations() {
        let env = Env::default();
        let test_string = String::from_str(&env, "test_patient_id");
        assert_eq!(test_string.len(), 15);
    }

    #[test]
    fn test_environment_setup() {
        let env = Env::default();
        assert!(env.ledger().timestamp() > 0);
        assert!(env.ledger().sequence() > 0);
    }
}