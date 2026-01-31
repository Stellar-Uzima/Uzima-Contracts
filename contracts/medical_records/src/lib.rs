#![no_std]
#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::enum_variant_names)]

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_permissions;

mod events;
mod validation;

use upgradeability::storage::{ADMIN as UPGRADE_ADMIN, VERSION};
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
        if env.storage().instance().has(&UPGRADE_ADMIN) {
            return false;
        }
        env.storage().instance().set(&UPGRADE_ADMIN, &admin);
        env.storage().instance().set(&VERSION, &1u32);
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

        events::emit_user_created(&env, admin.clone(), admin, "Admin", None);
        true
    }

    fn has_role(env: &Env, address: &Address, role: &Role) -> Result<(), Error> {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(env));
        match users.get(address.clone()) {
            Some(profile) if profile.role == *role && profile.active => Ok(()),
            _ => {
                env.events().publish(
                    (symbol_short!("ERR_LOG"), symbol_short!("AUTH_FL")),
                    (address.clone(), Error::NotAuthorized as u32),
                );
                Err(Error::NotAuthorized)
            }
        }
    }

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

    fn load_ai_config(env: &Env) -> Result<AIConfig, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::AIConfig)
            .ok_or(Error::AIConfigNotSet)
    }

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
            Error::NotAuthorized
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

    fn check_permission(env: &Env, user: &Address, permission: Permission) -> Result<(), Error> {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(env));

        if let Some(profile) = users.get(user.clone()) {
            if !profile.active {
                env.events().publish(
                    (symbol_short!("ERR_LOG"), symbol_short!("DEACTIVE")),
                    (user.clone(), Error::NotAuthorized as u32),
                );
                return Err(Error::NotAuthorized);
            }
            match profile.role {
                Role::Admin => return Ok(()),
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
                _ => {}
            }
        }

        let key = DataKey::UserPermissions(user.clone());
        let grants: Vec<PermissionGrant> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        let now = env.ledger().timestamp();

        for grant in grants.iter() {
            if grant.permission == permission {
                if grant.expires_at == 0 || grant.expires_at > now {
                    return Ok(());
                } else {
                    env.events().publish(
                        (symbol_short!("ERR_LOG"), symbol_short!("EXPIRED")),
                        (user.clone(), permission as u32),
                    );
                    return Err(Error::NotAuthorized);
                }
            }
        }

        env.events().publish(
            (symbol_short!("ERR_LOG"), symbol_short!("PERM_DENY")),
            (user.clone(), permission as u32, Error::NotAuthorized as u32),
        );
        Err(Error::NotAuthorized)
    }

    pub fn grant_permission(
        env: Env,
        granter: Address,
        grantee: Address,
        permission: Permission,
        expiration: u64,
        is_delegatable: bool,
    ) -> Result<bool, Error> {
        granter.require_auth();
        if Self::check_permission(&env, &granter, Permission::DelegatePermission).is_err() {
            Self::has_role(&env, &granter, &Role::Admin)?;
        }

        let key = DataKey::UserPermissions(grantee.clone());
        let grants: Vec<PermissionGrant> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        let mut found = false;
        let mut new_grants = Vec::new(&env);
        for grant in grants.iter() {
            if grant.permission == permission {
                new_grants.push_back(PermissionGrant {
                    permission,
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
        events::emit_permission_granted(&env, granter, grantee, permission as u32, expiration, is_delegatable);
        Ok(true)
    }

    pub fn revoke_permission(
        env: Env,
        revoker: Address,
        grantee: Address,
        permission: Permission,
    ) -> Result<bool, Error> {
        revoker.require_auth();
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
        Self::is_paused(&env)?;
        Self::check_permission(&env, &caller, Permission::CreateRecord)?;

        validation::validate_diagnosis(&diagnosis)?;
        validation::validate_treatment(&treatment)?;
        validation::validate_category(&category, &env)?;
        validation::validate_treatment_type(&treatment_type)?;
        validation::validate_data_ref(&env, &data_ref)?;
        validation::validate_tags(&tags)?;

        if patient == caller {
            env.events().publish(
                (symbol_short!("ERR_LOG"), symbol_short!("SELF_REC")),
                (caller.clone(), Error::SameAddress as u32),
            );
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
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        events::emit_record_created(&env, caller, record_id, patient, is_confidential, category, tags);
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
        record.diagnosis = new_diagnosis;
        record.treatment = new_treatment;
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);
        Ok(true)
    }

    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Result<MedicalRecord, Error> {
        caller.require_auth();
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));

        let record = records.get(record_id).ok_or_else(|| {
            env.events().publish(
                (symbol_short!("ERR_LOG"), symbol_short!("REC_404")),
                (record_id, Error::RecordNotFound as u32),
            );
            Error::RecordNotFound
        })?;

        if record.patient_id == caller || record.doctor_id == caller {
            events::emit_record_accessed(&env, caller, record_id, record.patient_id.clone());
            return Ok(record);
        }

        Self::check_permission(&env, &caller, Permission::ReadRecord)?;
        if record.is_confidential {
            if let Err(e) = Self::check_permission(&env, &caller, Permission::ReadConfidential) {
                env.events().publish(
                    (symbol_short!("ERR_LOG"), symbol_short!("CONF_DEN")),
                    (caller.clone(), record_id, Error::NotAuthorized as u32),
                );
                return Err(e);
            }
        }

        events::emit_record_accessed(&env, caller, record_id, record.patient_id.clone());
        Ok(record)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&UPGRADE_ADMIN)
            .ok_or(Error::NotAuthorized)?;
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    pub fn version(env: Env) -> u32 {
        env.storage().instance().get(&VERSION).unwrap_or(0)
    }
}