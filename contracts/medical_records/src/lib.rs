#![no_std]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::enum_variant_names)]

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_migration;
#[cfg(test)]
mod test_permissions;

mod events;
mod validation;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Map, String, Symbol,
    Vec,
};

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

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum Role {
    Admin,
    Doctor,
    Patient,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum Permission {
    // Admin / Management
    ManageUsers = 1,
    ManageSystem = 2,

    // Record Access
    CreateRecord = 10,
    ReadRecord = 11,
    UpdateRecord = 12,
    DeleteRecord = 13,

    // Privacy
    ReadConfidential = 20,

    // Advanced
    DelegatePermission = 30,
}

#[derive(Clone)]
#[contracttype]
pub struct PermissionGrant {
    pub permission: Permission,
    pub granter: Address,
    pub expires_at: u64, // 0 means no expiration
    pub is_delegatable: bool,
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
    Admin,
    RecordCount,
    IdentityRegistry,
    AuthLevel,
    AccessLog(u64),
    AccessLogCount,
    PatientEmergencyGrants(Address),
    AIConfig,
    PatientRisk(Address),
    RecordAnomaly(Address, u64),
    UserPermissions(Address),
    ContractVersion, // <--- Added for migration system
}

const USERS: Symbol = symbol_short!("USERS");
const RECORDS: Symbol = symbol_short!("RECORDS");
const _PATIENT_RECORDS: Symbol = symbol_short!("PATIENT_R");
const PAUSED: Symbol = symbol_short!("PAUSED");
const _PROPOSALS: Symbol = symbol_short!("PROPOSALS");
const BRIDGE_CONTRACT: Symbol = symbol_short!("BRIDGE");
const IDENTITY_CONTRACT: Symbol = symbol_short!("IDENTITY");
const ACCESS_CONTRACT: Symbol = symbol_short!("ACCESS");
const CROSS_CHAIN_REFS: Symbol = symbol_short!("CC_REFS");
const CROSS_CHAIN_ENABLED: Symbol = symbol_short!("CC_ON");

const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400;

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
    Overflow = 47,
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

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl MedicalRecordsContract {
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();
        env.storage().persistent().set(&PAUSED, &false);

        let mut users: Map<Address, UserProfile> = Map::new(&env);
        users.set(
            admin.clone(),
            UserProfile {
                role: Role::Admin,
                active: true,
                did_reference: None,
            },
        );
        env.storage().persistent().set(&USERS, &users);

        // Emit user creation event
        events::emit_user_created(&env, admin.clone(), admin, "Admin", None);
        true
    }

    /// Internal function to check if an address has a specific role
    fn has_role(env: &Env, address: &Address, role: &Role) -> bool {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(env));
        match users.get(address.clone()) {
            Some(profile) => profile.role == *role && profile.active,
            None => false,
        }
    }

    /// Internal function to check paused state
    fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

    /// Internal function to get and increment the record counter
    fn get_and_increment_record_count(env: &Env) -> Result<u64, Error> {
        let current_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RecordCount)
            .unwrap_or(0);
        let next_count = current_count.checked_add(1).ok_or(Error::Overflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::RecordCount, &next_count);
        Ok(next_count)
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
        // FIXED: Using ! because has_role returns a bool
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
                    role,
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
                    role,
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

    /// Internal function to check granular permissions
    fn check_permission(env: &Env, user: &Address, permission: Permission) -> bool {
        // 1. Role-based default permissions (Imply logic)
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(env));

        if let Some(profile) = users.get(user.clone()) {
            if !profile.active {
                return false;
            }
            match profile.role {
                Role::Admin => return true, // Admin has all permissions
                Role::Doctor => {
                    if matches!(
                        permission,
                        Permission::CreateRecord
                            | Permission::ReadRecord
                            | Permission::UpdateRecord
                            | Permission::ReadConfidential
                            | Permission::DelegatePermission
                    ) {
                        return true;
                    }
                }
                Role::Patient => {
                    // Patients might have specific permissions contextually, but generally strict.
                }
                _ => {}
            }
        }

        // 2. Explicit Granular permissions
        let key = DataKey::UserPermissions(user.clone());
        let grants: Vec<PermissionGrant> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        let now = env.ledger().timestamp();

        for grant in grants.iter() {
            if grant.permission == permission {
                // Check expiration
                if grant.expires_at == 0 || grant.expires_at > now {
                    return true;
                }
            }
        }

        false
    }

    /// Grant a specific permission to a user
    pub fn grant_permission(
        env: Env,
        granter: Address,
        grantee: Address,
        permission: Permission,
        expiration: u64, // 0 = permanent
        is_delegatable: bool,
    ) -> Result<bool, Error> {
        granter.require_auth();

        // Granter must satisfy one of:
        // 1. Is Admin
        // 2. Has DelegatePermission permission (and if we tracked scope, within scope)
        if !Self::check_permission(&env, &granter, Permission::DelegatePermission)
            && !Self::has_role(&env, &granter, &Role::Admin)
        {
            return Err(Error::NotAuthorized);
        }

        let key = DataKey::UserPermissions(grantee.clone());
        let grants: Vec<PermissionGrant> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        // Check if already exists, update if so
        let mut found = false;
        let mut new_grants = Vec::new(&env);

        for grant in grants.iter() {
            if grant.permission == permission {
                new_grants.push_back(PermissionGrant {
                    permission, // Copy
                    granter: granter.clone(),
                    expires_at: expiration,
                    is_delegatable,
                });
                found = true;
            } else {
                new_grants.push_back(grant);
            }
        }

        if !found {
            new_grants.push_back(PermissionGrant {
                permission,
                granter: granter.clone(),
                expires_at: expiration,
                is_delegatable,
            });
        }

        env.storage().persistent().set(&key, &new_grants);

        // Emit event
        events::emit_permission_granted(
            &env,
            granter,
            grantee,
            permission as u32,
            expiration,
            is_delegatable,
        );

        Ok(true)
    }

    /// Revoke a permission from a user
    pub fn revoke_permission(
        env: Env,
        revoker: Address,
        grantee: Address,
        permission: Permission,
    ) -> Result<bool, Error> {
        revoker.require_auth();

        // Revoker must have authority
        if !Self::check_permission(&env, &revoker, Permission::DelegatePermission)
            && !Self::has_role(&env, &revoker, &Role::Admin)
        {
            return Err(Error::NotAuthorized);
        }

        let key = DataKey::UserPermissions(grantee.clone());
        let grants: Vec<PermissionGrant> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        let mut new_grants = Vec::new(&env);
        let mut removed = false;

        for grant in grants.iter() {
            if grant.permission == permission {
                removed = true;
            } else {
                new_grants.push_back(grant);
            }
        }

        if removed {
            env.storage().persistent().set(&key, &new_grants);
            events::emit_permission_revoked(&env, revoker, grantee, permission as u32);
        }

        Ok(removed)
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

        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        // Granular Permission Check
        if !Self::check_permission(&env, &caller, Permission::CreateRecord) {
            return Err(Error::NotAuthorized);
        }

        // Validation
        validation::validate_diagnosis(&diagnosis)?;
        validation::validate_treatment(&treatment)?;
        validation::validate_category(&category, &env)?;
        validation::validate_treatment_type(&treatment_type)?;
        validation::validate_data_ref(&env, &data_ref)?;
        validation::validate_tags(&tags)?;

        if patient == caller {
            return Err(Error::SameAddress);
        }

        let record_id = Self::get_and_increment_record_count(&env)?;

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

        let mut records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        records.set(record_id, record.clone());
        env.storage().persistent().set(&RECORDS, &records);

        events::emit_record_created(
            &env,
            caller,
            record_id,
            patient,
            is_confidential,
            category,
            tags,
        );

        Ok(record_id)
    }

    pub fn update_record(
        env: Env,
        caller: Address,
        record_id: u64,
        new_diagnosis: String,
        new_treatment: String,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        if !Self::check_permission(&env, &caller, Permission::UpdateRecord) {
            return Err(Error::NotAuthorized);
        }

        let mut records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let mut record = records.get(record_id).ok_or(Error::RecordNotFound)?;

        // Additional check: If confidential, might need ReadConfidential to even know about it?
        // But for update, we assume they know ID.

        record.diagnosis = new_diagnosis;
        record.treatment = new_treatment;
        // record.timestamp = env.ledger().timestamp(); // preserve original timestamp? or update? usually keep created_at

        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        // events::emit_record_updated(&env, caller, record_id); // Assuming this exists or I add it

        Ok(true)
    }

    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Result<MedicalRecord, Error> {
        caller.require_auth();

        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let record = records.get(record_id).ok_or(Error::RecordNotFound)?;

        // Permission Check
        let mut allowed = false;

        // 1. Own record OR Doctor who created it
        if record.patient_id == caller || record.doctor_id == caller {
            allowed = true;
        }
        // 3. Permission
        else if Self::check_permission(&env, &caller, Permission::ReadRecord) {
            allowed = true;
            // If confidential, check ReadConfidential
            if record.is_confidential
                && !Self::check_permission(&env, &caller, Permission::ReadConfidential)
            {
                allowed = false;
            }
        } else if record.is_confidential {
            // If no generic ReadRecord, maybe they have ReadConfidential specific to this record?
            // Granular scope isn't implemented per record in PermissionGrant (it's boolean per user).
            // We stick to simple logic: ReadRecord allows reading public records. ReadConfidential allows confidential.
        }

        if !allowed {
            return Err(Error::NotAuthorized);
        }

        events::emit_record_accessed(&env, caller, record_id, record.patient_id.clone());

        Ok(record)
    }

    // =================================================================
    // MIGRATION & UPGRADE SYSTEM
    // =================================================================

    // Helper to get the current version
    fn get_contract_version(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ContractVersion)
            .unwrap_or(0) // Default to 0 if not set
    }

    // Helper to set the version
    fn set_contract_version(env: &Env, new_version: u32) {
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &new_version);
    }

    // The Main Upgrade Function
    // Updates WASM code and migrates data atomically.
    pub fn upgrade(env: Env, caller: Address, new_wasm_hash: BytesN<32>) {
        // A. Security Check
        caller.require_auth();

        // Use existing Role system instead of missing DataKey::Admin
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Not authorized to upgrade contract");
        }

        // B. Update the Contract Code
        env.deployer().update_current_contract_wasm(new_wasm_hash);

        // C. Run Data Migration
        Self::migrate_data(&env);
    }

    // The Data Migration Logic
    fn migrate_data(env: &Env) {
        // Define the version this code represents
        const CURRENT_CONTRACT_VERSION: u32 = 1;

        let current_version = Self::get_contract_version(env);

        // If the stored data is older than the code's version, run migration
        if current_version < CURRENT_CONTRACT_VERSION {
            // Example: Migration from V0 to V1
            if current_version < 1 {
                // e.g., Self::migrate_v0_to_v1(env);
            }

            // Update the stored version to match the current code
            Self::set_contract_version(env, CURRENT_CONTRACT_VERSION);
        }
    }
}