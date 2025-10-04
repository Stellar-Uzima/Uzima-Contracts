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
    pub tags: Vec<String>,
    pub category: String,
    pub treatment_type: String,
    pub data_ref: String, // <-- added off-chain data reference
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    RecordCount,
}

const USERS: Symbol = Symbol::short("USERS");
const ADMINS: Symbol = Symbol::short("ADMINS");
const RECORDS: Symbol = Symbol::short("RECORDS");
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
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();
        let users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        if !users.is_empty() { panic!("Contract already initialized"); }

        let admin_profile = UserProfile { role: Role::Admin, active: true };
        let mut users_map = Map::new(&env);
        users_map.set(admin, admin_profile);
        env.storage().persistent().set(&USERS, &users_map);
        env.storage().persistent().set(&PAUSED, &false);
        true
    }

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

    fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get::<bool>(&PAUSED).unwrap_or(false)
    }

    fn get_and_increment_record_count(env: &Env) -> u64 {
        let current_count: u64 = env.storage().persistent().get(&DataKey::RecordCount).unwrap_or(0);
        let next_count = current_count + 1;
        env.storage().persistent().set(&DataKey::RecordCount, &next_count);
        next_count
    }

    fn validate_data_ref(_env: &Env, data_ref: &String) {
        if data_ref.len() == 0 { panic!("data_ref cannot be empty"); }
        if data_ref.len() < 46 || data_ref.len() > 100 { panic!("data_ref length must be between 46 and 100 characters"); }
        for c in data_ref.chars() {
            if !(c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                panic!("data_ref contains invalid characters");
            }
        }
    }

    pub fn pause(env: Env, caller: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { panic!("Only admins can pause"); }
        env.storage().persistent().set(&PAUSED, &true);
        env.events().publish((Symbol::short("Paused"),), (caller.clone(), env.ledger().timestamp()));
        true
    }

    pub fn unpause(env: Env, caller: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { panic!("Only admins can unpause"); }
        env.storage().persistent().set(&PAUSED, &false);
        env.events().publish((Symbol::short("Unpaused"),), (caller.clone(), env.ledger().timestamp()));
        true
    }

    pub fn manage_user(env: Env, caller: Address, user: Address, role: Role) -> bool {
        caller.require_auth();
        if Self::is_paused(&env) { panic!("Contract is paused"); }
        if !Self::has_role(&env, &caller, &Role::Admin) { panic!("Only admins can manage users"); }

        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        let profile = UserProfile { role, active: true };
        users.set(user, profile);
        env.storage().persistent().set(&USERS, &users);
        true
    }

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
        data_ref: String, // <-- new parameter
    ) -> u64 {
        caller.require_auth();
        if Self::is_paused(&env) { panic!("Contract is paused"); }
        if !Self::has_role(&env, &caller, &Role::Doctor) { panic!("Only doctors can add medical records"); }

        Self::validate_data_ref(&env, &data_ref);

        let allowed_categories = vec![
            String::from_str(&env, "Modern"),
            String::from_str(&env, "Traditional"),
            String::from_str(&env, "Herbal"),
            String::from_str(&env, "Spiritual"),
        ];
        if !allowed_categories.contains(&category) { panic!("Invalid category"); }
        if treatment_type.len() == 0 { panic!("Treatment type cannot be empty"); }
        for tag in tags.iter() { if tag.len() == 0 { panic!("Tags cannot be empty"); } }

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
            data_ref, // <-- store off-chain reference
        };

        let mut records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        env.events().publish((Symbol::short("RecordAdded"),), record_id);
        record_id
    }

    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Option<MedicalRecord> {
        caller.require_auth();
        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        if let Some(record) = records.get(record_id) {
            if Self::has_role(&env, &caller, &Role::Admin)
                || caller == record.patient_id
                || caller == record.doctor_id
                || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential) {
                Some(record)
            } else { panic!("Unauthorized access to medical record"); }
        } else { None }
    }

    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();
        if Self::is_paused(&env) { panic!("Contract is paused"); }
        if !Self::has_role(&env, &caller, &Role::Admin) { panic!("Only admins can deactivate users"); }

        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        if let Some(mut profile) = users.get(user) {
            profile.active = false;
            users.set(user, profile);
            env.storage().persistent().set(&USERS, &users);
            true
        } else { false }
    }

    pub fn get_user_role(env: Env, user: Address) -> Role {
        let users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        match users.get(user) { Some(profile) => profile.role, None => Role::None }
    }

    // Recovery functions remain unchanged...
}
