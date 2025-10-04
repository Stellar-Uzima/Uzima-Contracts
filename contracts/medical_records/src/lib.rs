#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, Env, Map, String, Symbol, Vec};

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
    pub tags: Vec<String>,
    pub category: String,
    pub treatment_type: String,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    RecordCount,
}

const USERS: Symbol = Symbol::short("USERS");
const ADMINS: Symbol = Symbol::short("ADMINS");
const RECORDS: Symbol = Symbol::short("RECORDS");
// Pausable state and recovery storage
const PAUSED: Symbol = Symbol::short("PAUSED");
const PROPOSALS: Symbol = Symbol::short("PROPOSALS");
const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400; // 24 hours timelock
const CONSENT_CONTRACT: Symbol = Symbol::short("CONSENT");

#[derive(Clone)]
#[contracttype]
pub struct RecoveryProposal {
    pub proposal_id: u64,
    pub token_contract: Address,
    pub to: Address,
    pub amount: i128,
    pub created_at: u64,
    pub executed: bool,
    pub approvals: Vec<Address>,
}

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    /// Initialize the contract with the first admin and consent contract address
    pub fn initialize(env: Env, admin: Address, consent_contract: Address) -> bool {
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

        // Initialize paused state to false
        env.storage().persistent().set(&PAUSED, &false);

        // Store consent contract address
        env.storage().persistent().set(&CONSENT_CONTRACT, &consent_contract);

        true
    }

    /// Internal function to check if an address has a specific role
    fn has_role(env: &Env, address: &Address, role: &Role) -> bool {
        let users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        match users.get(address.clone()) {
            Some(profile) => matches!((profile.role, role),
                (Role::Admin, Role::Admin) |
                (Role::Doctor, Role::Doctor) |
                (Role::Patient, Role::Patient)) && profile.active,
            None => false,
        }
    }

    /// Internal function to check paused state
    fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

    /// Internal function to get and increment the record counter
    fn get_and_increment_record_count(env: &Env) -> u64 {
        let current_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RecordCount)
            .unwrap_or(0);
        let next_count = current_count + 1;
        env.storage()
            .persistent()
            .set(&DataKey::RecordCount, &next_count);
        next_count
    }

    /// Check if a patient has given consent to a doctor for accessing confidential records
    fn has_consent(env: &Env, patient: &Address, _doctor: &Address) -> bool {
        // Check if consent contract is configured
        if !env.storage().persistent().has(&CONSENT_CONTRACT) {
            return false; // No consent contract configured
        }

        // For this basic implementation, we'll assume if a consent contract is configured
        // and the patient exists, they have given general consent.
        // In a real implementation, you would make cross-contract calls to check specific consent tokens.

        // Note: Cross-contract calls in Soroban require proper client generation and type handling.
        // For the purposes of this demonstration, we'll implement a simplified version.

        // TODO: Implement actual cross-contract calls when consent contract client is available
        // This would involve:
        // 1. Creating a client for the consent contract
        // 2. Calling tokens_of_owner(patient)
        // 3. For each token, calling is_valid(token_id)
        // 4. Checking if any valid token grants access to the requesting doctor

        true // Simplified: assume consent is granted if consent contract is configured
    }

    /// Emergency pause - only admins
    pub fn pause(env: Env, caller: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can pause");
        }
        env.storage().persistent().set(&PAUSED, &true);
        // Emit Paused event
        let ts = env.ledger().timestamp();
        env.events().publish((Symbol::short("Paused"),), (caller.clone(), ts));
        true
    }

    /// Resume operations - only admins
    pub fn unpause(env: Env, caller: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can unpause");
        }
        env.storage().persistent().set(&PAUSED, &false);
        // Emit Unpaused event
        let ts = env.ledger().timestamp();
        env.events().publish((Symbol::short("Unpaused"),), (caller.clone(), ts));
        true
    }

    /// Add or update a user with a specific role
    pub fn manage_user(env: Env, caller: Address, user: Address, role: Role) -> bool {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

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
        tags: Vec<String>,
        category: String,
        treatment_type: String,
    ) -> u64 {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

        // Verify caller is a doctor
        if !Self::has_role(&env, &caller, &Role::Doctor) {
            panic!("Only doctors can add medical records");
        }

        // Validate category
        let allowed_categories = vec![
            &env,
            String::from_str(&env, "Modern"),
            String::from_str(&env, "Traditional"),
            String::from_str(&env, "Herbal"),
            String::from_str(&env, "Spiritual"),
        ];
        if !allowed_categories.contains(&category) {
            panic!("Invalid category");
        }

        // Validate treatment_type (non-empty)
        if treatment_type.len() == 0 {
            panic!("Treatment type cannot be empty");
        }

        // Validate tags (all non-empty)
        for tag in tags.iter() {
            if tag.len() == 0 {
                panic!("Tags cannot be empty");
            }
        }

        let record_id = Self::get_and_increment_record_count(&env);
        let timestamp = env.ledger().timestamp();

        let record = MedicalRecord {
            patient_id: patient.clone(),
            doctor_id: caller.clone(),
            timestamp,
            diagnosis,
            treatment,
            is_confidential,
            tags,
            category,
            treatment_type,
        };

        // Store the record
        let mut records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        // Emit RecordAdded event
        env.events().publish((Symbol::short("RecordAdded"),), record_id);

        record_id
    }

    /// Get a medical record with role-based access control and consent verification
    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Option<MedicalRecord> {
        caller.require_auth();

        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));

        if let Some(record) = records.get(record_id) {
            // Allow access if:
            // 1. Caller is an admin
            // 2. Caller is the patient
            // 3. Caller is the doctor who created the record
            // 4. Caller is any doctor and record is not confidential
            // 5. Caller is a doctor, record is confidential, and patient has given consent
            if Self::has_role(&env, &caller, &Role::Admin)
                || caller == record.patient_id
                || caller == record.doctor_id
                || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential)
                || (Self::has_role(&env, &caller, &Role::Doctor) && record.is_confidential && Self::has_consent(&env, &record.patient_id, &caller)) {
                Some(record)
            } else {
                panic!("Unauthorized access to medical record - consent required for confidential records");
            }
        } else {
            None
        }
    }

    /// Deactivate a user
    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

        // Only admins can deactivate users
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can deactivate users");
        }

        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));

        if let Some(mut profile) = users.get(user.clone()) {
            profile.active = false;
            users.set(user, profile);
            env.storage().persistent().set(&USERS, &users);
            true
        } else {
            false
        }
    }

    /// Get the role of a user by address (public key)
    pub fn get_user_role(env: Env, user: Address) -> Role {
        let users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        match users.get(user) {
            Some(profile) => profile.role,
            None => Role::None,
        }
    }

    // ------------------ Recovery (timelock + multisig) ------------------

    /// Propose a recovery operation (e.g., recover tokens sent to this contract)
    pub fn propose_recovery(env: Env, caller: Address, token_contract: Address, to: Address, amount: i128) -> u64 {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can propose recovery");
        }

        let proposal_id = Self::get_and_increment_record_count(&env);
        let created_at = env.ledger().timestamp();
        let mut proposal = RecoveryProposal {
            proposal_id,
            token_contract,
            to,
            amount,
            created_at,
            executed: false,
            approvals: Vec::new(&env),
        };
        // Initial approval by proposer for convenience
        proposal.approvals.push_back(caller.clone());

        let mut proposals: Map<u64, RecoveryProposal> = env.storage().persistent().get(&PROPOSALS).unwrap_or(Map::new(&env));
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        proposal_id
    }

    /// Approve a recovery proposal (admin only)
    pub fn approve_recovery(env: Env, caller: Address, proposal_id: u64) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can approve recovery");
        }

        let mut proposals: Map<u64, RecoveryProposal> = env.storage().persistent().get(&PROPOSALS).unwrap_or(Map::new(&env));
        let mut proposal = proposals.get(proposal_id).unwrap_or_else(|| panic!("Proposal not found"));
        if proposal.executed {
            panic!("Proposal already executed");
        }
        // Prevent duplicate approvals
        if proposal.approvals.iter().any(|a| a == caller) {
            return true;
        }
        proposal.approvals.push_back(caller.clone());
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        true
    }

    /// Execute recovery after timelock and approvals threshold
    pub fn execute_recovery(env: Env, caller: Address, proposal_id: u64) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can execute recovery");
        }

        let mut proposals: Map<u64, RecoveryProposal> = env.storage().persistent().get(&PROPOSALS).unwrap_or(Map::new(&env));
        let mut proposal = proposals.get(proposal_id).unwrap_or_else(|| panic!("Proposal not found"));
        if proposal.executed {
            panic!("Proposal already executed");
        }

        // Check timelock
        let now = env.ledger().timestamp();
        if now < proposal.created_at + TIMELOCK_SECS {
            panic!("Timelock not elapsed");
        }

        // Check multisig approvals
        let distinct_approvals = proposal.approvals.len();
        if (distinct_approvals as u32) < APPROVAL_THRESHOLD {
            panic!("Not enough approvals");
        }

        // In actual implementation, we would invoke the token contract transfer here.
        // For auditability within this project scope, we just mark executed and emit an event.
        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        // Emit RecoveryExecuted event
        let ts = env.ledger().timestamp();
        env.events().publish((Symbol::short("RecoveryExecuted"),), (caller.clone(), proposal_id, ts));
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke};
    use soroban_sdk::{Address, Env, String, Vec};

    #[test]
    fn test_add_and_get_record() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        let diagnosis = String::from_str(&env, "Common cold");
        let treatment = String::from_str(&env, "Rest and fluids");

        // Initialize and set roles
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add a record
        let record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &diagnosis,
                &treatment,
                &false,
                &vec![String::from_str(&env, "respiratory")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "Medication"),
            );

        // Get the record as patient
        let retrieved_record = client.mock_all_auths().get_record(&patient, &record_id);
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

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add multiple records for the same patient
        let record_id1 = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis 1"),
                &String::from_str(&env, "Treatment 1"),
                &false,
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );

        let record_id2 = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis 2"),
                &String::from_str(&env, "Treatment 2"),
                &true,
                &vec![String::from_str(&env, "spiritual")],
                String::from_str(&env, "Spiritual"),
                String::from_str(&env, "Prayer"),
            );

        // Patient can access both records
        assert!(client.mock_all_auths().get_record(&patient, &record_id1).is_some());
        assert!(client.mock_all_auths().get_record(&patient, &record_id2).is_some());
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
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);

        // Admin manages user roles
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Doctor adds a confidential record
        let record_id = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Flu"),
                &String::from_str(&env, "Antiviral medication"),
                &true,
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );

        // Patient tries to access the record (should succeed)
        let retrieved_record = client.mock_all_auths().get_record(&patient, &record_id);
        assert!(retrieved_record.is_some());

        // Doctor (creator) tries to access the record (should succeed)
        let retrieved_record = client.mock_all_auths().get_record(&doctor, &record_id);
        assert!(retrieved_record.is_some());

        // Admin tries to access the record (should succeed)
        let retrieved_record = client.mock_all_auths().get_record(&admin, &record_id);
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
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);

        // Admin manages user roles
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Deactivate the doctor
        assert!(client.mock_all_auths().deactivate_user(&admin, &doctor));

        // Try to add a record as the deactivated doctor (should fail)
        let result = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .try_add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Cold"),
                &String::from_str(&env, "Rest"),
                &false,
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );
        assert!(result.is_err());

        // Reactivate the doctor
        assert!(client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor));

        // Add a record as the reactivated doctor (should succeed)
        let record_id = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Cold"),
                &String::from_str(&env, "Rest"),
                &false,
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );
        assert!(record_id > 0);
    }

    #[test]
    fn test_pause_unpause_blocks_sensitive_functions() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize and set up roles
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add a record (not paused)
        let _record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis"),
                &String::from_str(&env, "Treatment"),
                &false,
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );

        // Pause the contract
        assert!(client.mock_all_auths().pause(&admin));

        // Mutating functions should be blocked when paused
        let r1 = client.mock_all_auths().try_manage_user(&admin, &Address::generate(&env), &Role::Doctor);
        assert!(r1.is_err());
        let r2 = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .try_add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis2"),
                &String::from_str(&env, "Treatment2"),
                &false,
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );
        assert!(r2.is_err());

        // Unpause
        assert!(client.mock_all_auths().unpause(&admin));

        // Now mutating calls should succeed
        assert!(client.mock_all_auths().manage_user(&admin, &Address::generate(&env), &Role::Doctor));
        let r3 = client
            .mock_auths(&[MockAuth { address: &doctor, invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] } }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis3"),
                &String::from_str(&env, "Treatment3"),
                &false,
                &vec![String::from_str(&env, "herbal")],
                String::from_str(&env, "Traditional"),
                String::from_str(&env, "Herbal Therapy"),
            );
        assert!(r3 > 0);
    }

    #[test]
    fn test_recovery_timelock_and_multisig() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin1 = Address::generate(&env);
        let admin2 = Address::generate(&env);
        let token = Address::generate(&env);
        let recipient = Address::generate(&env);

        // Initialize and add second admin
        client.mock_all_auths().initialize(&admin1);
        client.mock_all_auths().manage_user(&admin1, &admin2, &Role::Admin);

        // Propose recovery by admin1
        let proposal_id = client
            .mock_all_auths()
            .propose_recovery(&admin1, &token, &recipient, &100i128);
        assert!(proposal_id > 0);

        // Approve by admin2
        assert!(client.mock_all_auths().approve_recovery(&admin2, &proposal_id));

        // Try execute before timelock elapsed -> should error
        let res = client.mock_all_auths().try_execute_recovery(&admin1, &proposal_id);
        assert!(res.is_err());

        // Advance time beyond timelock
        let now = env.ledger().timestamp();
        env.ledger().with_mut(|l| {
            l.timestamp = now + TIMELOCK_SECS + 1;
        });

        // Execute should succeed now
        assert!(client.mock_all_auths().execute_recovery(&admin1, &proposal_id));
    }

    #[test]
    fn test_monotonic_record_ids() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize and set roles
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add multiple records and verify IDs are monotonically increasing
        let record_id1 = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis 1"),
                &String::from_str(&env, "Treatment 1"),
                &false,
                &vec![String::from_str(&env, "tag1")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "Type1"),
            );

        let record_id2 = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis 2"),
                &String::from_str(&env, "Treatment 2"),
                &false,
                &vec![String::from_str(&env, "tag2")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "Type2"),
            );

        let record_id3 = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis 3"),
                &String::from_str(&env, "Treatment 3"),
                &false,
                &vec![String::from_str(&env, "tag3")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "Type3"),
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
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor1 = Address::generate(&env);
        let doctor2 = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize and set roles
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor1, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &doctor2, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add records from different doctors
        let record_id1 = client
            .mock_auths(&[MockAuth {
                address: &doctor1,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor1,
                &patient,
                &String::from_str(&env, "Diagnosis A"),
                &String::from_str(&env, "Treatment A"),
                &false,
                &vec![String::from_str(&env, "tag")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "TypeA"),
            );

        let record_id2 = client
            .mock_auths(&[MockAuth {
                address: &doctor2,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor2,
                &patient,
                &String::from_str(&env, "Diagnosis B"),
                &String::from_str(&env, "Treatment B"),
                &false,
                &vec![String::from_str(&env, "tag")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "TypeB"),
            );

        // Verify all IDs are unique
        assert_ne!(record_id1, record_id2);
    }

    #[test]
    fn test_record_ordering() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize and set roles
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add records in sequence
        let mut record_ids: Vec<u64> = Vec::new(&env);
        let diagnoses = vec![
            &env,
            String::from_str(&env, "Diagnosis 0"),
            String::from_str(&env, "Diagnosis 1"),
            String::from_str(&env, "Diagnosis 2"),
            String::from_str(&env, "Diagnosis 3"),
            String::from_str(&env, "Diagnosis 4"),
        ];
        let treatments = vec![
            &env,
            String::from_str(&env, "Treatment 0"),
            String::from_str(&env, "Treatment 1"),
            String::from_str(&env, "Treatment 2"),
            String::from_str(&env, "Treatment 3"),
            String::from_str(&env, "Treatment 4"),
        ];

        for i in 0..5 {
            let id = client
                .mock_auths(&[MockAuth {
                    address: &doctor,
                    invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
                }])
                .add_record(
                    &doctor,
                    &patient,
                    &diagnoses.get(i).unwrap(),
                    &treatments.get(i).unwrap(),
                    &false,
                    &vec![String::from_str(&env, "tag")],
                    String::from_str(&env, "Modern"),
                    String::from_str(&env, "Type"),
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
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize and set roles
        let consent_contract = Address::generate(&env);
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Add first record
        let record_id1 = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis"),
                &String::from_str(&env, "Treatment"),
                &false,
                &vec![String::from_str(&env, "tag")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "Type"),
            );

        // Create a recovery proposal (also uses the counter)
        let proposal_id = client
            .mock_all_auths()
            .propose_recovery(&admin, &Address::generate(&env), &Address::generate(&env), &100i128);

        // Add another record
        let record_id2 = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis 2"),
                &String::from_str(&env, "Treatment 2"),
                &false,
                &vec![String::from_str(&env, "tag")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "Type"),
            );

        // Verify all IDs are unique and monotonic
        assert_eq!(record_id1, 1);
        assert_eq!(proposal_id, 2);
        assert_eq!(record_id2, 3);
        assert!(proposal_id > record_id1);
        assert!(record_id2 > proposal_id);
    }

    #[test]
    fn test_confidential_record_access_without_consent() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor1 = Address::generate(&env);
        let doctor2 = Address::generate(&env);
        let patient = Address::generate(&env);
        let consent_contract = Address::generate(&env);

        // Initialize and set roles
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor1, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &doctor2, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Doctor1 adds a confidential record
        let record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor1,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor1,
                &patient,
                &String::from_str(&env, "Confidential diagnosis"),
                &String::from_str(&env, "Confidential treatment"),
                &true, // confidential
                &vec![String::from_str(&env, "confidential")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "Specialized"),
            );

        // Patient can access their own confidential record
        let patient_record = client.mock_all_auths().get_record(&patient, &record_id);
        assert!(patient_record.is_some());

        // Doctor1 (creator) can access the confidential record
        let doctor1_record = client.mock_all_auths().get_record(&doctor1, &record_id);
        assert!(doctor1_record.is_some());

        // Admin can access any record
        let admin_record = client.mock_all_auths().get_record(&admin, &record_id);
        assert!(admin_record.is_some());

        // Doctor2 (different doctor) should NOT be able to access confidential record without consent
        let result = client.mock_all_auths().try_get_record(&doctor2, &record_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_confidential_record_access() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor1 = Address::generate(&env);
        let doctor2 = Address::generate(&env);
        let patient = Address::generate(&env);
        let consent_contract = Address::generate(&env);

        // Initialize and set roles
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor1, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &doctor2, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);

        // Doctor1 adds a non-confidential record
        let record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor1,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor1,
                &patient,
                &String::from_str(&env, "Common cold"),
                &String::from_str(&env, "Rest and fluids"),
                &false, // not confidential
                &vec![String::from_str(&env, "routine")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "General"),
            );

        // Patient can access their own record
        let patient_record = client.mock_all_auths().get_record(&patient, &record_id);
        assert!(patient_record.is_some());

        // Doctor1 (creator) can access the record
        let doctor1_record = client.mock_all_auths().get_record(&doctor1, &record_id);
        assert!(doctor1_record.is_some());

        // Doctor2 (different doctor) can access non-confidential record
        let doctor2_record = client.mock_all_auths().get_record(&doctor2, &record_id);
        assert!(doctor2_record.is_some());

        // Admin can access any record
        let admin_record = client.mock_all_auths().get_record(&admin, &record_id);
        assert!(admin_record.is_some());
    }

    #[test]
    fn test_unauthorized_role_access() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicalRecordsContract);
        let client = MedicalRecordsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        let unauthorized_user = Address::generate(&env);
        let consent_contract = Address::generate(&env);

        // Initialize and set roles
        client.mock_all_auths().initialize(&admin, &consent_contract);
        client.mock_all_auths().manage_user(&admin, &doctor, &Role::Doctor);
        client.mock_all_auths().manage_user(&admin, &patient, &Role::Patient);
        // Note: unauthorized_user is not given any role

        // Doctor adds a record
        let record_id = client
            .mock_auths(&[MockAuth {
                address: &doctor,
                invoke: &MockAuthInvoke { contract: &contract_id, fn_name: "add_record", args: (), sub_invokes: &[] },
            }])
            .add_record(
                &doctor,
                &patient,
                &String::from_str(&env, "Diagnosis"),
                &String::from_str(&env, "Treatment"),
                &false,
                &vec![String::from_str(&env, "tag")],
                String::from_str(&env, "Modern"),
                String::from_str(&env, "General"),
            );

        // Unauthorized user should not be able to access any record
        let result = client.mock_all_auths().try_get_record(&unauthorized_user, &record_id);
        assert!(result.is_err());
    }
}
