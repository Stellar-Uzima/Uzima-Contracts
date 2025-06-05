#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec};

#[derive(Clone)]
#[contracttype]
pub struct MedicalRecord {
    pub patient_id: String,
    pub doctor_id: String,
    pub timestamp: u64,
    pub diagnosis: String,
    pub treatment: String,
    pub is_confidential: bool,
}

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    /// Initialize the contract
    pub fn initialize(env: Env) -> bool {
        // Contract initialization logic
        true
    }

    /// Add a new medical record
    pub fn add_record(
        env: Env,
        caller: Address,
        patient_id: String,
        doctor_id: String,
        diagnosis: String,
        treatment: String,
        is_confidential: bool,
    ) -> u64 {
        // Verify caller authorization
        caller.require_auth();

        let timestamp = env.ledger().timestamp();
        let record_id = env.ledger().sequence();

        let record = MedicalRecord {
            patient_id: patient_id.clone(),
            doctor_id,
            timestamp,
            diagnosis,
            treatment,
            is_confidential,
        };

        // Store the record
        env.storage()
            .persistent()
            .set(&record_id, &record);

        // Update patient's record list
        let mut patient_records: Vec<u64> = env
            .storage()
            .persistent()
            .get(&patient_id)
            .unwrap_or(Vec::new(&env));
        
        patient_records.push_back(record_id);
        env.storage()
            .persistent()
            .set(&patient_id, &patient_records);

        record_id
    }

    /// Get a medical record by ID
    pub fn get_record(env: Env, record_id: u64) -> Option<MedicalRecord> {
        env.storage().persistent().get(&record_id)
    }

    /// Get all records for a patient
    pub fn get_patient_records(env: Env, patient_id: String) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&patient_id)
            .unwrap_or(Vec::new(&env))
    }

    /// Update a medical record (only by authorized doctor)
    pub fn update_record(
        env: Env,
        caller: Address,
        record_id: u64,
        diagnosis: String,
        treatment: String,
    ) -> bool {
        caller.require_auth();

        if let Some(mut record) = env.storage().persistent().get(&record_id) {
            let record: MedicalRecord = record;
            
            // Verify caller is the original doctor
            if caller.to_string() != record.doctor_id {
                return false;
            }

            let updated_record = MedicalRecord {
                patient_id: record.patient_id,
                doctor_id: record.doctor_id,
                timestamp: env.ledger().timestamp(),
                diagnosis,
                treatment,
                is_confidential: record.is_confidential,
            };

            env.storage()
                .persistent()
                .set(&record_id, &updated_record);
            
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_add_and_get_record() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let doctor = Address::generate(&env);
        let patient_id = String::from_str(&env, "patient_001");
        let doctor_id = String::from_str(&env, "doctor_001");
        let diagnosis = String::from_str(&env, "Common cold");
        let treatment = String::from_str(&env, "Rest and fluids");

        // Add a record
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
            .add_record(&doctor, &patient_id, &doctor_id, &diagnosis, &treatment, &false);

        // Get the record
        let retrieved_record = client.get_record(&record_id);
        
        assert!(retrieved_record.is_some());
        let record = retrieved_record.unwrap();
        assert_eq!(record.patient_id, patient_id);
        assert_eq!(record.diagnosis, diagnosis);
        assert_eq!(record.treatment, treatment);
        assert_eq!(record.is_confidential, false);
    }

    #[test]
    fn test_get_patient_records() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let doctor = Address::generate(&env);
        let patient_id = String::from_str(&env, "patient_001");
        let doctor_id = String::from_str(&env, "doctor_001");

        // Add multiple records for the same patient
        let _record_id1 = client
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
                &String::from_str(&env, "Diagnosis 1"),
                &String::from_str(&env, "Treatment 1"),
                &false,
            );

        let _record_id2 = client
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
                &String::from_str(&env, "Diagnosis 2"),
                &String::from_str(&env, "Treatment 2"),
                &true,
            );

        // Get patient records
        let patient_records = client.get_patient_records(&patient_id);
        assert_eq!(patient_records.len(), 2);
    }
}