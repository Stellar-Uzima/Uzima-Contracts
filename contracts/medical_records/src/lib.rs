#![no_std]

//#[cfg(test)]
//mod test;

// Keep your new migration test active
#[cfg(test)]
mod test_migration;

mod events;
mod validation;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, BytesN, Env,
    Map, String, Symbol, Vec,
};

// ==================== Constants ====================

// NEW: Tracks the current version of the contract logic.
// Increment this number (2, 3, etc.) whenever you merge a PR that changes data structures.
const CONTRACT_VERSION: u32 = 1;

const USERS: Symbol = symbol_short!("USERS");
const RECORDS: Symbol = symbol_short!("RECORDS");
const PATIENT_RECORDS: Symbol = symbol_short!("PATIENT_R");
const PAUSED: Symbol = symbol_short!("PAUSED");
const PROPOSALS: Symbol = symbol_short!("PROPOSALS");
const BRIDGE_CONTRACT: Symbol = symbol_short!("BRIDGE");
const IDENTITY_CONTRACT: Symbol = symbol_short!("IDENTITY");
const ACCESS_CONTRACT: Symbol = symbol_short!("ACCESS");
const CROSS_CHAIN_REFS: Symbol = symbol_short!("CC_REFS");
const CROSS_CHAIN_ENABLED: Symbol = symbol_short!("CC_ON");

const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400;

// ==================== Cross-Chain Types ====================

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum ChainId {
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    BinanceSmartChain,
    Arbitrum,
    Optimism,
    Custom(u32),
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainRecordRef {
    pub local_record_id: u64,
    pub external_chain: ChainId,
    pub external_record_hash: BytesN<32>,
    pub sync_timestamp: u64,
    pub is_synced: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct RecordMetadata {
    pub record_id: u64,
    pub patient_id: Address,
    pub timestamp: u64,
    pub category: String,
    pub is_confidential: bool,
    pub record_hash: BytesN<32>,
}

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
    pub did_reference: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub enum DIDAuthLevel {
    None,
    Basic,
    CredentialRequired,
    Full,
}

#[derive(Clone)]
#[contracttype]
pub struct AccessRequest {
    pub requester: Address,
    pub patient: Address,
    pub record_id: u64,
    pub purpose: String,
    pub timestamp: u64,
    pub granted: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct EmergencyAccess {
    pub grantee: Address,
    pub patient: Address,
    pub expires_at: u64,
    pub record_scope: Vec<u64>,
    pub is_active: bool,
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
    pub data_ref: String,
    pub doctor_did: Option<String>,
}

// ==================== AI & Recovery Types ====================

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AIInsightType {
    AnomalyScore,
    RiskScore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AIInsight {
    pub patient: Address,
    pub record_id: u64,
    pub model_id: BytesN<32>,
    pub insight_type: AIInsightType,
    pub score_bps: u32,
    pub explanation_ref: String,
    pub explanation_summary: String,
    pub created_at: u64,
    pub model_version: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AIConfig {
    pub ai_coordinator: Address,
    pub dp_epsilon: u32,
    pub min_participants: u32,
}

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

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    RecordCount,
    IdentityRegistry,
    AuthLevel,
    AccessLog(u64),
    AccessLogCount,
    PatientEmergencyGrants(Address),
    AIConfig,
    PatientRisk(Address),
    RecordAnomaly(Address, u64),
    // NEW: Tracks the current data version to prevent schema mismatches
    ProtocolVersion,
}

// ==================== Errors ====================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ContractPaused = 1,
    NotAuthorized = 2,
    InvalidCategory = 3,
    EmptyTreatment = 4,
    EmptyTag = 5,
    ProposalAlreadyExecuted = 6,
    TimelockNotElasped = 7,
    NotEnoughApproval = 8,
    EmptyDataRef = 9,
    InvalidDataRefLength = 10,
    InvalidDataRefCharset = 11,
    CrossChainNotEnabled = 12,
    CrossChainContractsNotSet = 13,
    RecordNotFound = 14,
    CrossChainAccessDenied = 15,
    RecordAlreadySynced = 16,
    InvalidChain = 17,
    DIDNotFound = 18,
    DIDNotActive = 19,
    InvalidCredential = 20,
    CredentialExpired = 21,
    CredentialRevoked = 22,
    MissingRequiredCredential = 23,
    EmergencyAccessExpired = 24,
    EmergencyAccessNotFound = 25,
    IdentityRegistryNotSet = 26,
    AIConfigNotSet = 27,
    NotAICoordinator = 28,
    InvalidAIScore = 29,
    // Validation Errors
    EmptyDiagnosis = 30,
    InvalidDiagnosisLength = 31,
    InvalidTreatmentLength = 32,
    InvalidPurposeLength = 33,
    InvalidTagLength = 34,
    InvalidScore = 35,
    InvalidDPEpsilon = 36,
    InvalidParticipantCount = 37,
    InvalidModelVersionLength = 38,
    InvalidExplanationLength = 39,
    InvalidAddress = 40,
    SameAddress = 41,
    InvalidTreatmentTypeLength = 42,
    BatchTooLarge = 43,
    InvalidBatch = 44,
    InvalidInput = 45,
    NumberOutOfBounds = 46,
}

// NEW: Specific errors for the migration system
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MigrationError {
    AlreadyOnLatestVersion = 100,
    MigrationFailed = 101,
}

#[derive(Clone)]
#[contracttype]
pub struct FailureInfo {
    pub index: u32,
    pub error_code: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct BatchResult {
    pub successes: Vec<u64>,
    pub failures: Vec<FailureInfo>,
}

// ==================== Contract Implementation ====================

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    // --- Initialization & Upgrade System ---

    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();
        env.storage().persistent().set(&PAUSED, &false);

        // Initialize version tracking
        env.storage()
            .instance()
            .set(&DataKey::ProtocolVersion, &CONTRACT_VERSION);

        // Emit user creation event
        events::emit_user_created(&env, admin.clone(), admin, "Admin", None);
        true
    }

    /// Upgrades the contract WASM code and migrates data atomically.
    /// This is the entry point for all future updates.
    pub fn upgrade(env: Env, caller: Address, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        caller.require_auth();

        // 1. Security: Only Admin can upgrade
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }

        // 2. Code Upgrade: Update the WASM blob
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        // 3. Data Migration: execute any necessary data transformations
        // If this fails, the entire upgrade rolls back
        Self::migrate_data(&env);

        Ok(())
    }

    /// Internal function to handle version-specific migrations
    fn migrate_data(env: &Env) {
        // Get the current stored version (default to 0 if not set)
        let current_version = env
            .storage()
            .instance()
            .get(&DataKey::ProtocolVersion)
            .unwrap_or(0u32);

        // Safety check: Prevent downgrades or re-running the same migration
        if current_version >= CONTRACT_VERSION {
            // We use panic here to ensure the transaction aborts and no gas is wasted on a no-op
            panic_with_error!(env, MigrationError::AlreadyOnLatestVersion);
        }

        // Migration Router: Apply changes sequentially
        // e.g. If current is 0 and target is 2, it runs 0->1, then 1->2
        let mut ver = current_version;
        while ver < CONTRACT_VERSION {
            match ver {
                0 => {
                    // Migration from V0 to V1
                    Self::migrate_v0_to_v1(env);
                }
                // Future versions will be added here:
                // 1 => Self::migrate_v1_to_v2(env),
                _ => panic_with_error!(env, MigrationError::MigrationFailed),
            }
            ver += 1;
        }

        // Update the stored version to match the code constant
        env.storage()
            .instance()
            .set(&DataKey::ProtocolVersion, &CONTRACT_VERSION);
    }

    /// Actual logic for V0 -> V1 migration
    fn migrate_v0_to_v1(_env: &Env) {
        // Example: If V1 introduced a new global counter, we would initialize it here.
        // For now, this acts as the "Genesis" migration logic.
        // _env.storage().persistent().set(&DataKey::NewFeature, &0);
    }

    // --- Core Logic ---

    /// Internal function to check if an address has a specific role
    fn has_role(env: &Env, address: &Address, role: &Role) -> bool {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(env));
        match users.get(address.clone()) {
            Some(profile) => {
                matches!(
                    (profile.role, role),
                    (Role::Admin, Role::Admin)
                        | (Role::Doctor, Role::Doctor)
                        | (Role::Patient, Role::Patient)
                ) && profile.active
            }
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

    pub fn get_record_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::RecordCount)
            .unwrap_or(0)
    }

    /// Internal helper to load AI configuration
    fn load_ai_config(env: &Env) -> Result<AIConfig, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::AIConfig)
            .ok_or(Error::AIConfigNotSet)
    }

    /// Ensure that the caller is the configured AI coordinator
    fn ensure_ai_coordinator(env: &Env, caller: &Address) -> Result<AIConfig, Error> {
        let config = Self::load_ai_config(env)?;
        if config.ai_coordinator != *caller {
            return Err(Error::NotAICoordinator);
        }
        Ok(config)
    }

    pub fn manage_user(env: Env, caller: Address, user: Address, role: Role) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return false;
        }

        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        let existing_profile = users.get(user.clone());

        let role_str = match role {
            Role::Admin => "Admin",
            Role::Doctor => "Doctor",
            Role::Patient => "Patient",
            Role::None => "None",
        };

        if let Some(profile) = existing_profile {
            // Update existing user
            let previous_role_str = match profile.role {
                Role::Admin => "Admin",
                Role::Doctor => "Doctor",
                Role::Patient => "Patient",
                Role::None => "None",
            };
            users.set(
                user.clone(),
                UserProfile {
                    role: role.clone(),
                    active: true,
                    did_reference: profile.did_reference,
                },
            );
            events::emit_user_role_updated(&env, caller, user, role_str, Some(previous_role_str));
        } else {
            // Create new user
            users.set(
                user.clone(),
                UserProfile {
                    role: role.clone(),
                    active: true,
                    did_reference: None,
                },
            );
            events::emit_user_created(&env, caller, user, role_str, None);
        }

        env.storage().persistent().set(&USERS, &users);
        true
    }

    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return false;
        }

        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        if let Some(mut profile) = users.get(user.clone()) {
            profile.active = false;
            users.set(user.clone(), profile);
            env.storage().persistent().set(&USERS, &users);
            events::emit_user_deactivated(&env, caller, user);
            true
        } else {
            false
        }
    }

    pub fn pause(env: Env, caller: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return false;
        }

        env.storage().persistent().set(&PAUSED, &true);
        events::emit_contract_paused(&env, caller);
        true
    }

    pub fn unpause(env: Env, caller: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return false;
        }

        env.storage().persistent().set(&PAUSED, &false);
        events::emit_contract_unpaused(&env, caller);
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
        data_ref: String,
    ) -> Result<u64, Error> {
        caller.require_auth();

        // 1. Create the Record Struct
        let record = MedicalRecord {
            patient_id: patient.clone(),
            doctor_id: caller.clone(),
            timestamp: env.ledger().timestamp(),
            diagnosis,
            treatment,
            is_confidential,
            tags: tags.clone(),
            category: category.clone(),
            treatment_type,
            data_ref,
            doctor_did: None,
        };

        // 2. Generate a new ID
        let record_id = Self::get_and_increment_record_count(&env);

        // 3. Load the existing Records Map
        let mut records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));

        // 4. Insert the new record
        records.set(record_id, record);

        // 5. Save the Map back to storage
        env.storage().persistent().set(&RECORDS, &records);

        // 6. Return the new ID
        Ok(record_id)
    }
}
