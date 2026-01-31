#![no_std]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::enum_variant_names)]

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_permissions;

mod errors;
mod events;
mod validation;

use crate::errors::Error;
use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Symbol, Vec,
};

// ... (Types remain the same until AccessRequest) ...

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
    fn has_role(env: &Env, address: &Address, role: &Role) -> Result<(), Error> {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(env));
        match users.get(address.clone()) {
            Some(profile) if profile.role == *role && profile.active => Ok(()),
            _ => {
                // AC: Context information (parameters involved: the address)
                env.events().publish(
                    (symbol_short!("ERR_LOG"), symbol_short!("AUTH_FL")),
                    (address.clone(), Error::NotAuthorized as u32),
                );
                Err(Error::NotAuthorized)
            }
        }
    }

    /// Internal function to check paused state
    fn is_paused(env: &Env) -> Result<(), Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            env.events().publish(
                (symbol_short!("ERR_LOG"), symbol_short!("PAUSED")),
                Error::ContractPaused as u32,
            );
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    /// Internal function to get and increment the record counter
    fn get_and_increment_record_count(env: &Env) -> Result<u64, Error> {
        let current: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RecordCount)
            .unwrap_or(0);
        let next = current.checked_add(1).ok_or_else(|| {
            env.events().publish(
                (symbol_short!("ERR_LOG"), symbol_short!("OVERFLOW")),
                Error::Overflow as u32,
            );
            Error::Overflow
        })?;
        env.storage().persistent().set(&DataKey::RecordCount, &next);
        Ok(next)
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

    pub fn manage_user(
        env: Env,
        caller: Address,
        user: Address,
        role: Role,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::has_role(&env, &caller, &Role::Admin)?;

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
            // Update existing user: Capture context for detailed feedback
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

            // AC: Provide specific feedback via events on what changed
            events::emit_user_role_updated(&env, caller, user, role_str, Some(previous_role_str));
        } else {
            // Create new user: Log context for monitoring
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

        // Return Ok(true) to signify successful execution with full context logged
        Ok(true)
    }

    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::has_role(&env, &caller, &Role::Admin)?;

        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        let mut profile = users.get(user.clone()).ok_or_else(|| {
            env.events().publish(
                (symbol_short!("ERR_LOG"), symbol_short!("USR_404")),
                user.clone(),
            );
            Error::NotAuthorized // Better for security: don't reveal existence, just deny
        })?;

        profile.active = false;
        users.set(user.clone(), profile);
        env.storage().persistent().set(&USERS, &users);
        events::emit_user_deactivated(&env, caller, user);
        Ok(true)
    }

    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::has_role(&env, &caller, &Role::Admin)?;

        env.storage().persistent().set(&PAUSED, &true);
        events::emit_contract_paused(&env, caller);
        Ok(true)
    }

    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::has_role(&env, &caller, &Role::Admin)?;

        env.storage().persistent().set(&PAUSED, &false);
        events::emit_contract_unpaused(&env, caller);
        Ok(true)
    }

    /// Internal function to check granular permissions
    fn check_permission(env: &Env, user: &Address, permission: Permission) -> Result<(), Error> {
        // 1. Role-based default permissions (Imply logic)
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(env));

        if let Some(profile) = users.get(user.clone()) {
            if !profile.active {
                // Log that a deactivated user tried to access the system
                env.events().publish(
                    (symbol_short!("ERR_LOG"), symbol_short!("DEACTIVE")),
                    (user.clone(), Error::NotAuthorized as u32),
                );
                return Err(Error::NotAuthorized);
            }
            match profile.role {
                Role::Admin => return Ok(()), // Admin has all permissions
                Role::Doctor => {
                    if matches!(
                        permission,
                        Permission::CreateRecord
                            | Permission::ReadRecord
                            | Permission::UpdateRecord
                            | Permission::ReadConfidential
                            | Permission::DelegatePermission
                    ) {
                        return Ok(());
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
                    return Ok(());
                } else {
                    // Specific log for expired permissions
                    env.events().publish(
                        (symbol_short!("ERR_LOG"), symbol_short!("EXPIRED")),
                        (user.clone(), permission as u32),
                    );
                    return Err(Error::NotAuthorized);
                }
            }
        }

        // --- THE CRITICAL UPDATE: Detailed Error Feedback ---
        // If we reach this point, no permission was found.
        // AC: Detailed context for debugging (User + Permission tried)
        env.events().publish(
            (symbol_short!("ERR_LOG"), symbol_short!("PERM_DENY")),
            (user.clone(), permission as u32, Error::NotAuthorized as u32),
        );

        Err(Error::NotAuthorized)
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
        // 1. Try the first permission.
        // If it fails (is_err), we try the second check.
        if Self::check_permission(&env, &granter, Permission::DelegatePermission).is_err() {
            // 2. If the user isn't a delegate, check if they are an Admin.
            // We use '?' here because if they aren't an Admin either,
            // we want to return the Error::NotAuthorized immediately.
            Self::has_role(&env, &granter, &Role::Admin)?;
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

        if Self::check_permission(&env, &revoker, Permission::DelegatePermission).is_err() {
            Self::has_role(&env, &revoker, &Role::Admin)?;
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

        // 1. System & Permission Checks (Using ? for automatic detailed logging)
        Self::is_paused(&env)?;
        Self::check_permission(&env, &caller, Permission::CreateRecord)?;

        // 2. Comprehensive Validation
        // These now return specific Error variants (e.g., EmptyDiagnosis)
        validation::validate_diagnosis(&diagnosis)?;
        validation::validate_treatment(&treatment)?;
        validation::validate_category(&category, &env)?;
        validation::validate_treatment_type(&treatment_type)?;
        validation::validate_data_ref(&env, &data_ref)?;
        validation::validate_tags(&tags)?;

        // 3. Logic Check: Prevent self-diagnosis for integrity
        if patient == caller {
            // AC: Add context information (the address involved)
            env.events().publish(
                (symbol_short!("ERR_LOG"), symbol_short!("SELF_REC")),
                (caller.clone(), Error::SameAddress as u32),
            );
            return Err(Error::SameAddress);
        }

        // 4. State Management
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

        // 5. Success Feedback
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

        Self::is_paused(&env)?;
        Self::check_permission(&env, &caller, Permission::UpdateRecord)?;

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

        // AC: Specific error for missing records with event logging for debugging
        let record = records.get(record_id).ok_or_else(|| {
            env.events().publish(
                (symbol_short!("ERR_LOG"), symbol_short!("REC_404")),
                (record_id, Error::RecordNotFound as u32),
            );
            Error::RecordNotFound
        })?;

        // --- ENHANCED PERMISSION LOGIC ---

        // 1. Direct Ownership (Patient or Original Doctor) - Always allowed
        if record.patient_id == caller || record.doctor_id == caller {
            events::emit_record_accessed(&env, caller, record_id, record.patient_id.clone());
            return Ok(record);
        }

        // 2. Third-Party Access (Other Doctors/Authorities)
        // Check general read permission first
        Self::check_permission(&env, &caller, Permission::ReadRecord)?;

        // 3. Confidentiality Check
        // If the record is marked confidential, verify specific 'ReadConfidential' grant
        if record.is_confidential {
            if let Err(e) = Self::check_permission(&env, &caller, Permission::ReadConfidential) {
                // AC: Detailed context - Log that general read was OK but confidential was denied
                env.events().publish(
                    (symbol_short!("ERR_LOG"), symbol_short!("CONF_DEN")),
                    (caller.clone(), record_id, Error::NotAuthorized as u32),
                );
                return Err(e);
            }
        }

        // 4. Success Feedback
        events::emit_record_accessed(&env, caller, record_id, record.patient_id.clone());
        Ok(record)
    }
}
