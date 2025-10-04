#![no_std]

#[cfg(test)]
mod test;

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
const PATIENT_RECORDS: Symbol = Symbol::short("PATIENT_R");
// Pausable state and recovery storage
const PAUSED: Symbol = Symbol::short("PAUSED");
const PROPOSALS: Symbol = Symbol::short("PROPOSALS");
const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400; // 24 hours timelock

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

        // Initialize paused state to false
        env.storage().persistent().set(&PAUSED, &false);

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

        // Track record ID per patient
        let mut patient_records: Map<Address, Vec<u64>> = env.storage().persistent().get(&PATIENT_RECORDS).unwrap_or(Map::new(&env));
        let mut ids = patient_records.get(patient.clone()).unwrap_or(Vec::new(&env));
        ids.push_back(record_id);
        patient_records.set(patient, ids);
        env.storage().persistent().set(&PATIENT_RECORDS, &patient_records);

        // Emit RecordAdded event
        env.events().publish(
            (
                Symbol::new(&env, "RecordAdded"),
            ),
            record_id,
        );

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

    /// Retrieve paginated history of records for a patient with access control
    pub fn get_history(
        env: Env,
        caller: Address,
        patient: Address,
        page: u32,
        page_size: u32,
    ) -> Vec<(u64, MedicalRecord)> {
        caller.require_auth();

        // Block when paused (optional; reads are generally allowed during pause)
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

        let patient_records: Map<Address, Vec<u64>> = env.storage().persistent().get(&PATIENT_RECORDS).unwrap_or(Map::new(&env));
        let mut ids = patient_records.get(patient).unwrap_or(Vec::new(&env));

        // Pagination: calculate start and end indices
        let start = page * page_size;
        let end = ((page + 1) * page_size).min(ids.len() as u32) as usize;

        if start >= ids.len() as u32 {
            return Vec::new(&env); // Empty page
        }

        // Gas bounding: limit total records fetched (e.g., max 100 per call)
        let max_fetch = 100u32.min(page_size * 2); // Conservative bound
        let actual_end = ((start + max_fetch as u32) as usize).min(end);

        let mut history = Vec::new(&env);
        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));

        for i in start as usize..actual_end {
            let record_id = ids.get(i as u32).unwrap();
            if let Some(record) = records.get(record_id) {
                if Self::has_role(&env, &caller, &Role::Admin)
                    || caller == record.patient_id
                    || caller == record.doctor_id
                    || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential) {
                    let tuple = (record_id, record);
                    history.push_back(tuple);
                }
            }
        }

        history
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
        // env.events().publish((Symbol::short("RecoveryExecuted"),), (caller.clone(), proposal_id, ts));
        true
    }
}
