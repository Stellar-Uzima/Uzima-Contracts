#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Symbol, Vec};

#[derive(Clone)]
#[contracttype]
pub enum Role {
    Admin,
    Doctor,
    Patient,
    None,
}

#[derive(Clone)]
#[contracttype]
pub struct UserProfile {
    pub role: Role,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicalRecord {
    pub patient_id: Address,
    pub doctor_id: Address,
    pub timestamp: u64,
    pub diagnosis: String,
    pub treatment: String,
    pub is_confidential: bool,
}

const USERS: Symbol = Symbol::short("USERS");
const ADMINS: Symbol = Symbol::short("ADMINS");
const RECORDS: Symbol = Symbol::short("RECORDS");

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    /// Initialize the contract with the first admin
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();
        
        // Ensure contract hasn't been initialized
        let users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        if !users.is_empty() {
            panic!("Contract already initialized");
        }

        // Set up initial admin
        let admin_profile = UserProfile {
            role: Role::Admin,
            active: true,
        };
        
        let mut users_map = Map::new(&env);
        users_map.set(admin, admin_profile);
        env.storage().persistent().set(&USERS, &users_map);
        
        true
    }

    /// Internal function to check if an address has a specific role
    fn has_role(env: &Env, address: &Address, role: &Role) -> bool {
        let users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        match users.get(address) {
            Some(profile) => matches!((profile.role, role), 
                (Role::Admin, Role::Admin) |
                (Role::Doctor, Role::Doctor) |
                (Role::Patient, Role::Patient)) && profile.active,
            None => false,
        }
    }

    /// Add or update a user with a specific role
    pub fn manage_user(env: Env, caller: Address, user: Address, role: Role) -> bool {
        caller.require_auth();
        
        // Only admins can manage users
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can manage users");
        }

        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        let profile = UserProfile {
            role,
            active: true,
        };
        
        users.set(user, profile);
        env.storage().persistent().set(&USERS, &users);
        
        true
    }

    /// Add a new medical record with role-based access control
    pub fn add_record(
        env: Env,
        caller: Address,
        patient: Address,
        diagnosis: String,
        treatment: String,
        is_confidential: bool,
    ) -> u64 {
        caller.require_auth();
        
        // Verify caller is a doctor
        if !Self::has_role(&env, &caller, &Role::Doctor) {
            panic!("Only doctors can add medical records");
        }

        let record_id = env.ledger().sequence();
        let timestamp = env.ledger().timestamp();

        let record = MedicalRecord {
            patient_id: patient.clone(),
            doctor_id: caller.clone(),
            timestamp,
            diagnosis,
            treatment,
            is_confidential,
        };

        // Store the record
        let mut records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        record_id
    }

    /// Get a medical record with role-based access control
    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Option<MedicalRecord> {
        caller.require_auth();

        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        
        if let Some(record) = records.get(record_id) {
            // Allow access if:
            // 1. Caller is an admin
            // 2. Caller is the patient
            // 3. Caller is the doctor who created the record
            // 4. Caller is any doctor and record is not confidential
            if Self::has_role(&env, &caller, &Role::Admin) 
                || caller == record.patient_id
                || caller == record.doctor_id
                || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential) {
                Some(record)
            } else {
                panic!("Unauthorized access to medical record");
            }
        } else {
            None
        }
    }

    /// Deactivate a user
    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();
        
        // Only admins can deactivate users
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can deactivate users");
        }

        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        
        if let Some(mut profile) = users.get(user) {
            profile.active = false;
            users.set(user, profile);
            env.storage().persistent().set(&USERS, &users);
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
        let patient = Address::generate(&env);
        let diagnosis = String::from_str(&env, "Common cold");
        let treatment = String::from_str(&env, "Rest and fluids");

        // Initialize the contract with the doctor as admin
        client.initialize(&doctor);

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
            .add_record(&doctor, &patient, &diagnosis, &treatment, &false);

        // Get the record
        let retrieved_record = client.get_record(&record_id);
        
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
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize the contract with the doctor as admin
        client.initialize(&doctor);

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
                &patient,
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
                &patient,
                &String::from_str(&env, "Diagnosis 2"),
                &String::from_str(&env, "Treatment 2"),
                &true,
            );

        // Get patient records
        let patient_records = client.get_patient_records(&patient);
        assert_eq!(patient_records.len(), 2);
    }

    #[test]
    fn test_role_based_access() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize the contract with the admin
        client.initialize(&admin);

        // Admin manages user roles
        client.manage_user(&admin, &doctor, Role::Doctor);
        client.manage_user(&admin, &patient, Role::Patient);

        // Doctor adds a record
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
                &patient,
                &String::from_str(&env, "Flu"),
                &String::from_str(&env, "Antiviral medication"),
                &true,
            );

        // Patient tries to access the record (should succeed)
        let retrieved_record = client.get_record(&record_id, &patient);
        assert!(retrieved_record.is_some());

        // Doctor tries to access the record (should succeed)
        let retrieved_record = client.get_record(&record_id, &doctor);
        assert!(retrieved_record.is_some());

        // Admin tries to access the record (should succeed)
        let retrieved_record = client.get_record(&record_id, &admin);
        assert!(retrieved_record.is_some());
    }

    #[test]
    fn test_deactivate_user() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize the contract with the admin
        client.initialize(&admin);

        // Admin manages user roles
        client.manage_user(&admin, &doctor, Role::Doctor);
        client.manage_user(&admin, &patient, Role::Patient);

        // Deactivate the doctor
        client.deactivate_user(&admin, &doctor);

        // Try to add a record as the deactivated doctor (should fail)
        let result = client.add_record(&doctor, &patient, &String::from_str(&env, "Cold"), &String::from_str(&env, "Rest"), &false);
        assert!(result.is_err());

        // Reactivate the doctor
        client.manage_user(&admin, &doctor, Role::Doctor);

        // Add a record as the reactivated doctor (should succeed)
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
                &patient,
                &String::from_str(&env, "Cold"),
                &String::from_str(&env, "Rest"),
                &false,
            );

        assert!(record_id.is_ok());
    }
}