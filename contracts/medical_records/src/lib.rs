#![no_std]
#![allow(clippy::needless_borrow)]

mod events;
mod rate_limiting;
mod validation;
use rate_limiting::{enforce_rate_limit, RateLimitError, UserRole as RLRole};

use soroban_sdk::symbol_short;
#[allow(unused_imports)]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
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
    pub credential_used: BytesN<32>,
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
    pub authorization_credential: BytesN<32>,
}

#[derive(Clone)]
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
    pub feature_importance: Vec<(String, u32)>,
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
pub enum DataKey {
    RecordCount,
    IdentityRegistry,
    AuthLevel,
    AccessLog(u64),
    AccessLogCount,
    EmergencyAccess(Address, Address),
    PatientEmergencyGrants(Address),
    AIConfig,
    PatientRisk(Address),
    RecordAnomaly(Address, u64),
}

const USERS: Symbol = symbol_short!("USERS");
const RECORDS: Symbol = symbol_short!("RECORDS");
const PATIENT_RECORDS: Symbol = symbol_short!("PATIENT_R");
const PAUSED: Symbol = symbol_short!("PAUSED");
const PROPOSALS: Symbol = symbol_short!("PROPOSALS");
const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400;

const BRIDGE_CONTRACT: Symbol = symbol_short!("BRIDGE");
const IDENTITY_CONTRACT: Symbol = symbol_short!("IDENTITY");
const ACCESS_CONTRACT: Symbol = symbol_short!("ACCESS");
const CROSS_CHAIN_REFS: Symbol = symbol_short!("CC_REFS");
const CROSS_CHAIN_ENABLED: Symbol = symbol_short!("CC_ON");

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
    RateLimitExceeded = 100,
}

impl From<RateLimitError> for Error {
    fn from(_: RateLimitError) -> Self {
        Error::RateLimitExceeded
    }
}

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();

        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        if !users.is_empty() {
            panic!("Contract already initialized");
        }

        let admin_profile = UserProfile {
            role: Role::Admin,
            active: true,
            did_reference: None,
        };
        let mut users_map = Map::new(&env);
        users_map.set(admin.clone(), admin_profile);
        env.storage().persistent().set(&USERS, &users_map);

        env.storage().persistent().set(&PAUSED, &false);

        Ok(true)
    }

    fn has_role(env: &Env, address: &Address, role: &Role) -> bool {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
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

    fn get_rate_limit_role(env: &Env, user: &Address) -> RLRole {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));

        match users.get(user.clone()) {
            Some(profile) => match profile.role {
                Role::Admin => RLRole::Admin,
                Role::Doctor => RLRole::Doctor,
                Role::Patient => RLRole::RegularUser,
                Role::None => RLRole::RegularUser,
            },
            None => RLRole::RegularUser,
        }
    }

    fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

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

    fn validate_data_ref(data_ref: &String) -> Result<(), Error> {
        if data_ref.is_empty() {
            return Err(Error::EmptyDataRef);
        }
        let len = data_ref.len();
        if !(1..=200).contains(&len) {
            return Err(Error::InvalidDataRefLength);
        }
        Ok(())
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
            // Check if this is a registered user (has any role in the system)
            let users: Map<Address, UserProfile> = env
                .storage()
                .persistent()
                .get(&USERS)
                .unwrap_or(Map::new(env));

            if let Some(profile) = users.get(caller.clone()) {
                if profile.active {
                    // This is a registered, active user trying to use AI functions
                    // They have some level of authorization, but not for AI operations
                    // Return NotAuthorized to indicate authorization mismatch
                    return Err(Error::NotAuthorized);
                }
            }

            // For unregistered/unknown users, return the specific error
            // indicating they need the AI coordinator role
            return Err(Error::NotAICoordinator);
        }
        Ok(config)
    }

    fn validate_ai_score(score_bps: u32) -> Result<(), Error> {
        if score_bps > 10_000 {
            return Err(Error::InvalidAIScore);
        }
        Ok(())
    }

    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin, "pause:admin")?;
        env.storage().persistent().set(&PAUSED, &true);
        events::emit_contract_paused(&env, caller);
        Ok(true)
    }

    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin, "unpause:admin")?;
        env.storage().persistent().set(&PAUSED, &false);
        events::emit_contract_unpaused(&env, caller);
        Ok(true)
    }

    pub fn manage_user(
        env: Env,
        caller: Address,
        user: Address,
        role: Role,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }
        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));

        let existing_did = users.get(user.clone()).and_then(|p| p.did_reference);
        let profile = UserProfile {
            role,
            active: true,
            did_reference: existing_did,
        };
        users.set(user, profile);
        env.storage().persistent().set(&USERS, &users);
        Ok(true)
    }

    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();

        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can deactivate users");
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

    pub fn get_user_role(env: Env, user: Address) -> Role {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        match users.get(user) {
            Some(profile) => profile.role,
            None => Role::None,
        }
    }

    #[allow(clippy::too_many_arguments)]
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

        if !Self::has_role(&env, &caller, &Role::Doctor) {
            return Err(Error::NotAuthorized);
        }

        let rl_role = Self::get_rate_limit_role(&env, &caller);
        enforce_rate_limit(&env, &caller, rl_role)?;

        Self::validate_data_ref(&data_ref)?;

        if diagnosis.is_empty() {
            return Err(Error::EmptyDiagnosis);
        }

        // FIXED: Enabled empty treatment check
        if treatment.is_empty() {
            return Err(Error::EmptyTreatment);
        }

        let allowed_categories = vec![
            &env,
            String::from_str(&env, "Modern"),
            String::from_str(&env, "Traditional"),
            String::from_str(&env, "Herbal"),
            String::from_str(&env, "Spiritual"),
        ];
        if !allowed_categories.contains(&category) {
            return Err(Self::log_error(
                &env,
                Error::InvalidCategory,
                "add_record:category",
                Some(caller.clone()),
            ));
        }

        if treatment_type.is_empty() {
            return Err(Error::EmptyTreatment);
        }

        for tag in tags.iter() {
            if tag.len() == 0 {
                return Err(Self::log_error(
                    &env,
                    Error::EmptyTag,
                    "add_record:tag",
                    Some(caller.clone()),
                ));
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
            tags: tags.clone(),
            category: category.clone(),
            treatment_type,
            data_ref,
            doctor_did: None,
            // Use zero bytes for none
            authorization_credential: BytesN::from_array(&env, &[0; 32]),
        };

        let mut records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        let mut patient_records: Map<Address, Vec<u64>> = env
            .storage()
            .persistent()
            .get(&PATIENT_RECORDS)
            .unwrap_or(Map::new(&env));
        let mut ids = patient_records
            .get(patient.clone())
            .unwrap_or(Vec::new(&env));
        ids.push_back(record_id);
        patient_records.set(patient.clone(), ids);
        env.storage()
            .persistent()
            .set(&PATIENT_RECORDS, &patient_records);

        Self::log_access(
            &env,
            &caller,
            &patient,
            record_id,
            String::from_str(&env, "CREATE_RECORD"),
            true,
            credential_id,
        );

        env.events().publish(
            (Symbol::new(&env, "RecordAdded"),),
            (patient.clone(), record_id, is_confidential),
        );
        Self::trigger_ai_analysis(&env, record_id, patient);

        Ok(record_id)
    }

    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Option<MedicalRecord> {
        caller.require_auth();
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        if let Some(record) = records.get(record_id) {
            if Self::has_role(&env, &caller, &Role::Admin)
                || caller == record.patient_id
                || caller == record.doctor_id
                || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential)
            {
                Ok(record)
            } else {
                Err(Self::log_error(
                    &env,
                    Error::NotAuthorized,
                    "get_record:not_authorized",
                    Some(caller),
                ))
            }
        } else {
            Err(Self::log_error(
                &env,
                Error::RecordNotFound,
                "get_record:not_found",
                Some(caller),
            ))
        }
    }

    pub fn get_history(
        env: Env,
        caller: Address,
        patient: Address,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<(u64, MedicalRecord)>, Error> {
        caller.require_auth();

        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }
        let patient_records: Map<Address, Vec<u64>> = env
            .storage()
            .persistent()
            .get(&PATIENT_RECORDS)
            .unwrap_or(Map::new(&env));
        let ids = patient_records.get(patient).unwrap_or(Vec::new(&env));

        let start = page * page_size;
        let end = ((page + 1) * page_size).min(ids.len() as u32) as usize;
        if start >= ids.len() as u32 {
            return Vec::new(&env);
        }

        let max_fetch = 100u32.min(page_size * 2);
        let actual_end = ((start + max_fetch as u32) as usize).min(end);
        let mut history = Vec::new(&env);
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        for i in start as usize..actual_end {
            let record_id = ids.get(i as u32).unwrap();
            if let Some(record) = records.get(record_id) {
                if Self::has_role(&env, &caller, &Role::Admin)
                    || caller == record.patient_id
                    || caller == record.doctor_id
                    || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential)
                {
                    let tuple = (record_id, record);
                    history.push_back(tuple);
                }
            }
        }

        Ok(history)
    }

    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();

        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can deactivate users");
        }
        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        if let Some(mut profile) = users.get(user.clone()) {
            profile.active = false;
            users.set(user, profile);
            env.storage().persistent().set(&USERS, &users);
            Ok(true)
        } else {
            Err(Self::log_error(
                &env,
                Error::UserNotFound,
                "deactivate_user:not_found",
                Some(caller),
            ))
        }
    }

    pub fn get_user_role(env: Env, user: Address) -> Role {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        match users.get(user) {
            Some(profile) => profile.role,
            None => Role::None,
        }
    }

    pub fn set_cross_chain_contracts(
        env: Env,
        caller: Address,
        bridge_contract: Address,
        identity_contract: Address,
        access_contract: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }
        env.storage()
            .persistent()
            .set(&BRIDGE_CONTRACT, &bridge_contract);
        env.storage()
            .persistent()
            .set(&IDENTITY_CONTRACT, &identity_contract);
        env.storage()
            .persistent()
            .set(&ACCESS_CONTRACT, &access_contract);
        env.storage().persistent().set(&CROSS_CHAIN_ENABLED, &true);
        env.events().publish(
            (Symbol::new(&env, "CrossChainConfigured"),),
            (bridge_contract, identity_contract, access_contract),
        );
        Ok(true)
    }

    pub fn set_cross_chain_enabled(
        env: Env,
        caller: Address,
        enabled: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }
        env.storage()
            .persistent()
            .set(&CROSS_CHAIN_ENABLED, &enabled);
        env.events()
            .publish((Symbol::new(&env, "CrossChainEnabledChanged"),), (enabled,));
        Ok(true)
    }

    pub fn is_cross_chain_enabled(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&CROSS_CHAIN_ENABLED)
            .unwrap_or(false)
    }

    pub fn get_cross_chain_contracts(env: Env) -> Option<(Address, Address, Address)> {
        let bridge: Option<Address> = env.storage().persistent().get(&BRIDGE_CONTRACT);
        let identity: Option<Address> = env.storage().persistent().get(&IDENTITY_CONTRACT);
        let access: Option<Address> = env.storage().persistent().get(&ACCESS_CONTRACT);
        match (bridge, identity, access) {
            (Some(b), Some(i), Some(a)) => Some((b, i, a)),
            _ => None,
        }
    }

    pub fn get_record_metadata(env: Env, record_id: u64) -> Result<RecordMetadata, Error> {
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let record = records.get(record_id).ok_or(Error::RecordNotFound)?;

        let record_hash = Self::compute_record_hash(&env, &record);
        Ok(RecordMetadata {
            record_id,
            patient_id: record.patient_id,
            timestamp: record.timestamp,
            category: record.category,
            is_confidential: record.is_confidential,
            record_hash,
        })
    }

    pub fn register_cross_chain_ref(
        env: Env,
        caller: Address,
        local_record_id: u64,
        external_chain: ChainId,
        external_record_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        if !Self::is_cross_chain_enabled(env.clone()) {
            return Err(Error::CrossChainNotEnabled);
        }

        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let record = records.get(local_record_id).ok_or(Error::RecordNotFound)?;

        if !Self::has_role(&env, &caller, &Role::Admin)
            && caller != record.patient_id
            && caller != record.doctor_id
        {
            return Err(Error::NotAuthorized);
        }
        let ref_entry = CrossChainRecordRef {
            local_record_id,
            external_chain: external_chain.clone(),
            external_record_hash,
            sync_timestamp: env.ledger().timestamp(),
            is_synced: true,
        };

        let ref_key = Self::cross_chain_ref_key(&env, local_record_id, &external_chain);
        let mut refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&CROSS_CHAIN_REFS)
            .unwrap_or(Map::new(&env));
        refs.set(ref_key, ref_entry);
        env.storage().persistent().set(&CROSS_CHAIN_REFS, &refs);
        env.events().publish(
            (Symbol::new(&env, "CrossChainRefRegistered"),),
            (local_record_id, external_chain),
        );
        Ok(true)
    }

    pub fn get_cross_chain_ref(
        env: Env,
        local_record_id: u64,
        external_chain: ChainId,
    ) -> Option<CrossChainRecordRef> {
        let ref_key = Self::cross_chain_ref_key(&env, local_record_id, &external_chain);
        let refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&CROSS_CHAIN_REFS)
            .unwrap_or(Map::new(&env));
        refs.get(ref_key)
    }

    pub fn get_record_cross_chain(
        env: Env,
        bridge_caller: Address,
        record_id: u64,
        accessor_chain: ChainId,
        accessor_address: String,
    ) -> Result<MedicalRecord, Error> {
        bridge_caller.require_auth();

        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        if !Self::is_cross_chain_enabled(env.clone()) {
            return Err(Error::CrossChainNotEnabled);
        }

        let bridge: Address = env
            .storage()
            .persistent()
            .get(&BRIDGE_CONTRACT)
            .ok_or(Error::CrossChainContractsNotSet)?;
        if bridge_caller != bridge {
            return Err(Error::NotAuthorized);
        }

        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let record = records.get(record_id).ok_or(Error::RecordNotFound)?;

        env.events().publish(
            (Symbol::new(&env, "CrossChainRecordAccess"),),
            (record_id, accessor_chain, accessor_address),
        );
        Ok(record)
    }

    pub fn update_cross_chain_sync(
        env: Env,
        caller: Address,
        local_record_id: u64,
        external_chain: ChainId,
        new_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        if !Self::is_cross_chain_enabled(env.clone()) {
            return Err(Error::CrossChainNotEnabled);
        }

        let is_admin = Self::has_role(&env, &caller, &Role::Admin);
        let bridge: Option<Address> = env.storage().persistent().get(&BRIDGE_CONTRACT);
        let is_bridge = bridge.map_or(false, |b| b == caller);
        if !is_admin && !is_bridge {
            return Err(Error::NotAuthorized);
        }
        let ref_key = Self::cross_chain_ref_key(&env, local_record_id, &external_chain);
        let mut refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&CROSS_CHAIN_REFS)
            .unwrap_or(Map::new(&env));
        if let Some(mut ref_entry) = refs.get(ref_key.clone()) {
            ref_entry.external_record_hash = new_hash;
            ref_entry.sync_timestamp = env.ledger().timestamp();
            ref_entry.is_synced = true;
            refs.set(ref_key, ref_entry);
            env.storage().persistent().set(&CROSS_CHAIN_REFS, &refs);
            env.events().publish(
                (Symbol::new(&env, "CrossChainSyncUpdated"),),
                (local_record_id, external_chain),
            );
            Ok(true)
        } else {
            Err(Error::RecordNotFound)
        }
    }

    pub fn get_all_cross_chain_refs(env: Env, local_record_id: u64) -> Vec<CrossChainRecordRef> {
        let refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&CROSS_CHAIN_REFS)
            .unwrap_or(Map::new(&env));
        let mut result = Vec::new(&env);

        let chains = vec![
            &env,
            ChainId::Ethereum,
            ChainId::Polygon,
            ChainId::Avalanche,
            ChainId::BinanceSmartChain,
            ChainId::Arbitrum,
            ChainId::Optimism,
        ];
        for chain in chains.iter() {
            let ref_key = Self::cross_chain_ref_key(&env, local_record_id, &chain);
            if let Some(ref_entry) = refs.get(ref_key) {
                result.push_back(ref_entry);
            }
        }
        result
    }

    fn cross_chain_ref_key(env: &Env, _record_id: u64, _chain: &ChainId) -> Symbol {
        Symbol::new(env, "cc_ref")
    }

    fn compute_record_hash(env: &Env, record: &MedicalRecord) -> BytesN<32> {
        BytesN::from_array(
            env,
            &[
                (record.timestamp % 256) as u8,
                (record.timestamp / 256 % 256) as u8,
                (record.timestamp / 65536 % 256) as u8,
                (record.timestamp / 16777216 % 256) as u8,
                if record.is_confidential { 1 } else { 0 },
                record.category.len() as u8,
                record.diagnosis.len() as u8,
                record.treatment.len() as u8,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        )
    }

    pub fn propose_recovery(
        env: Env,
        caller: Address,
        token_contract: Address,
        to: Address,
        amount: i128,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin, "propose_recovery:admin")?;

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
        proposal.approvals.push_back(caller.clone());
        let mut proposals: Map<u64, RecoveryProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        Ok(proposal_id)
    }

    pub fn approve_recovery(env: Env, caller: Address, proposal_id: u64) -> bool {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin, "approve_recovery:admin")?;

        let mut proposals: Map<u64, RecoveryProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));
        let mut proposal = proposals.get(proposal_id).ok_or_else(|| {
            Self::log_error(
                &env,
                Error::ProposalNotFound,
                "approve_recovery:not_found",
                Some(caller.clone()),
            )
        })?;
        if proposal.executed {
            panic!("Proposal already executed");
        }
        if proposal.approvals.iter().any(|a| a == caller) {
            return Err(Self::log_error(
                &env,
                Error::DuplicateApproval,
                "approve_recovery:duplicate",
                Some(caller),
            ));
        }
        proposal.approvals.push_back(caller.clone());
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        Ok(true)
    }

    pub fn execute_recovery(env: Env, caller: Address, proposal_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_role(&env, &caller, Role::Admin, "execute_recovery:admin")?;

        let mut proposals: Map<u64, RecoveryProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));
        let mut proposal = proposals.get(proposal_id).ok_or_else(|| {
            Self::log_error(
                &env,
                Error::ProposalNotFound,
                "execute_recovery:not_found",
                Some(caller.clone()),
            )
        })?;
        if proposal.executed {
            return Err(Self::log_error(
                &env,
                Error::ProposalAlreadyExecuted,
                "execute_recovery:executed",
                Some(caller.clone()),
            ));
        }

        let now = env.ledger().timestamp();
        if now < proposal.created_at + TIMELOCK_SECS {
            return Err(Self::log_error(
                &env,
                Error::TimelockNotElasped,
                "execute_recovery:timelock",
                Some(caller.clone()),
            ));
        }

        let distinct_approvals = proposal.approvals.len();
        if (distinct_approvals as u32) < APPROVAL_THRESHOLD {
            return Err(Error::NotEnoughApproval);
        }

        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        Ok(true)
    }

    pub fn set_identity_registry(
        env: Env,
        caller: Address,
        registry_address: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();

        Self::ensure_role(&env, &caller, Role::Admin, "set_identity_registry:admin")?;

        env.storage()
            .persistent()
            .set(&DataKey::IdentityRegistry, &registry_address);
        env.events().publish(
            (Symbol::new(&env, "IdentityRegistrySet"),),
            registry_address,
        );
        Ok(true)
    }

    pub fn set_did_auth_level(
        env: Env,
        caller: Address,
        level: DIDAuthLevel,
    ) -> Result<bool, Error> {
        caller.require_auth();

        Self::ensure_role(&env, &caller, Role::Admin, "set_did_auth_level:admin")?;

        env.storage().persistent().set(&DataKey::AuthLevel, &level);

        env.events()
            .publish((Symbol::new(&env, "AuthLevelSet"),), level);

        Ok(true)
    }

    pub fn get_did_auth_level(env: Env) -> DIDAuthLevel {
        env.storage()
            .persistent()
            .get(&DataKey::AuthLevel)
            .unwrap_or(DIDAuthLevel::None)
    }

    pub fn get_identity_registry(env: Env) -> Option<Address> {
        env.storage().persistent().get(&DataKey::IdentityRegistry)
    }

    pub fn link_did_to_user(
        env: Env,
        caller: Address,
        user: Address,
        did_reference: String,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if caller != user && !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Self::log_error(
                &env,
                Error::NotAuthorized,
                "link_did_to_user:not_authorized",
                Some(caller.clone()),
            ));
        }
        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        if let Some(mut profile) = users.get(user.clone()) {
            profile.did_reference = Some(did_reference.clone());
            users.set(user.clone(), profile);
            env.storage().persistent().set(&USERS, &users);

            env.events()
                .publish((Symbol::new(&env, "DIDLinked"),), (user, did_reference));

            Ok(true)
        } else {
            Err(Self::log_error(
                &env,
                Error::UserNotFound,
                "link_did_to_user:not_found",
                Some(caller),
            ))
        }
    }

    pub fn get_user_did(env: Env, user: Address) -> Option<String> {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        users.get(user).and_then(|p| p.did_reference)
    }

    pub fn grant_emergency_access(
        env: Env,
        patient: Address,
        grantee: Address,
        duration_secs: u64,
        record_scope: Vec<u64>,
    ) -> Result<bool, Error> {
        patient.require_auth();

        if !Self::has_role(&env, &patient, &Role::Patient) {
            return Err(Error::NotAuthorized);
        }

        let rl_role = Self::get_rate_limit_role(&env, &patient);
        enforce_rate_limit(&env, &patient, rl_role)?;

        let now = env.ledger().timestamp();
        let expires_at = now + duration_secs;
        let emergency_access = EmergencyAccess {
            grantee: grantee.clone(),
            patient: patient.clone(),
            expires_at,
            record_scope: record_scope.clone(),
            is_active: true,
        };

        env.storage().persistent().set(
            &DataKey::EmergencyAccess(grantee.clone(), patient.clone()),
            &emergency_access,
        );

        let mut patient_grants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientEmergencyGrants(patient.clone()))
            .unwrap_or(Vec::new(&env));
        if !patient_grants.iter().any(|a| a == grantee) {
            patient_grants.push_back(grantee.clone());
            env.storage().persistent().set(
                &DataKey::PatientEmergencyGrants(patient.clone()),
                &patient_grants,
            );
        }
        env.events().publish(
            (Symbol::new(&env, "EmergencyAccessGranted"),),
            (patient, grantee, expires_at),
        );
        Ok(true)
    }

    pub fn revoke_emergency_access(
        env: Env,
        patient: Address,
        grantee: Address,
    ) -> Result<bool, Error> {
        patient.require_auth();

        let rl_role = Self::get_rate_limit_role(&env, &patient);
        enforce_rate_limit(&env, &patient, rl_role)?;

        let key = DataKey::EmergencyAccess(grantee.clone(), patient.clone());
        if let Some(mut access) = env
            .storage()
            .persistent()
            .get::<DataKey, EmergencyAccess>(&key)
        {
            if !access.is_active {
                return Err(Error::EmergencyAccessNotFound);
            }

            access.is_active = false;
            env.storage().persistent().set(&key, &access);
            env.events().publish(
                (Symbol::new(&env, "EmergencyAccessRevoked"),),
                (patient, grantee),
            );
            Ok(true)
        } else {
            Err(Self::log_error(
                &env,
                Error::EmergencyAccessNotFound,
                "revoke_emergency_access:not_found",
                Some(patient),
            ))
        }
    }

    pub fn has_emergency_access(
        env: Env,
        grantee: Address,
        patient: Address,
        record_id: u64,
    ) -> bool {
        let key = DataKey::EmergencyAccess(grantee, patient);

        if let Some(mut access) = env
            .storage()
            .persistent()
            .get::<DataKey, EmergencyAccess>(&key)
        {
            if !access.is_active {
                return false;
            }
            let now = env.ledger().timestamp();
            if now > access.expires_at {
                access.is_active = false;
                env.storage().persistent().set(&key, &access);
                return false;
            }

            if access.record_scope.is_empty() {
                return true;
            }

            access.record_scope.iter().any(|id| id == record_id)
        } else {
            false
        }
    }

    pub fn get_patient_emergency_grants(env: Env, patient: Address) -> Vec<EmergencyAccess> {
        let grant_addresses: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientEmergencyGrants(patient.clone()))
            .unwrap_or(Vec::new(&env));
        let mut active_grants = Vec::new(&env);
        let now = env.ledger().timestamp();
        for grantee in grant_addresses.iter() {
            let key = DataKey::EmergencyAccess(grantee, patient.clone());
            if let Some(access) = env
                .storage()
                .persistent()
                .get::<DataKey, EmergencyAccess>(&key)
            {
                if access.is_active && now <= access.expires_at {
                    active_grants.push_back(access);
                }
            }
        }
        active_grants
    }

    fn log_access(
        env: &Env,
        requester: &Address,
        patient: &Address,
        record_id: u64,
        purpose: String,
        granted: bool,
        credential_used: BytesN<32>,
    ) {
        let log_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AccessLogCount)
            .unwrap_or(0);
        let access_request = AccessRequest {
            requester: requester.clone(),
            patient: patient.clone(),
            record_id,
            purpose,
            timestamp: env.ledger().timestamp(),
            granted,
            credential_used: credential_used.unwrap_or(BytesN::from_array(env, &[0; 32])),
        };
        let new_count = log_count + 1;
        env.storage()
            .persistent()
            .set(&DataKey::AccessLog(new_count), &access_request);
        env.storage()
            .persistent()
            .set(&DataKey::AccessLogCount, &new_count);
        env.events().publish(
            (Symbol::new(env, "AccessLogged"),),
            (requester.clone(), patient.clone(), record_id, granted),
        );
    }

    pub fn get_access_logs(env: Env, page: u32, page_size: u32) -> Vec<AccessRequest> {
        let total_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AccessLogCount)
            .unwrap_or(0);
        let start = (page * page_size) as u64 + 1;
        let end = ((page + 1) * page_size) as u64;
        let actual_end = end.min(total_count);
        let mut logs = Vec::new(&env);
        for i in start..=actual_end {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AccessRequest>(&DataKey::AccessLog(i))
            {
                logs.push_back(log);
            }
        }
        logs
    }

    pub fn get_patient_access_logs(
        env: Env,
        caller: Address,
        patient: Address,
        page: u32,
        page_size: u32,
    ) -> Vec<AccessRequest> {
        caller.require_auth();

        if caller != patient
            && !Self::has_role(&env, &caller, &Role::Admin)
            && !Self::has_emergency_access(env.clone(), caller.clone(), patient.clone(), 0)
        {
            return Vec::new(&env);
        }
        let total_count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AccessLogCount)
            .unwrap_or(0);
        let mut patient_logs = Vec::new(&env);
        let mut collected = 0u32;
        let skip = page * page_size;
        for i in 1..=total_count {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AccessRequest>(&DataKey::AccessLog(i))
            {
                if log.patient == patient {
                    if collected >= skip && collected < skip + page_size {
                        patient_logs.push_back(log);
                    }
                    collected += 1;
                }
            }
        }
        patient_logs
    }

    pub fn add_record_with_did(
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
        credential_id: BytesN<32>,
    ) -> Result<u64, Error> {
        caller.require_auth();

        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        if !Self::has_role(&env, &caller, &Role::Doctor) {
            return Err(Error::NotAuthorized);
        }

        let rl_role = Self::get_rate_limit_role(&env, &caller);
        enforce_rate_limit(&env, &caller, rl_role)?;

        Self::validate_data_ref(&data_ref)?;

        let allowed_categories = vec![
            &env,
            String::from_str(&env, "Modern"),
            String::from_str(&env, "Traditional"),
            String::from_str(&env, "Herbal"),
            String::from_str(&env, "Spiritual"),
        ];
        if !allowed_categories.contains(&category) {
            return Err(Self::log_error(
                &env,
                Error::InvalidCategory,
                "add_record_with_did:category",
                Some(caller.clone()),
            ));
        }

        if treatment_type.len() == 0 {
            return Err(Error::EmptyTreatment);
        }

        for tag in tags.iter() {
            if tag.len() == 0 {
                return Err(Self::log_error(
                    &env,
                    Error::EmptyTag,
                    "add_record_with_did:tag",
                    Some(caller.clone()),
                ));
            }
        }

        let doctor_did = Self::get_user_did(env.clone(), caller.clone());
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
            data_ref,
            doctor_did,
            // FIXED: Handle Option<BytesN> conversion
            authorization_credential: credential_id.clone().unwrap_or(BytesN::from_array(&env, &[0; 32])),
        };

        let mut records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        let mut patient_records: Map<Address, Vec<u64>> = env
            .storage()
            .persistent()
            .get(&PATIENT_RECORDS)
            .unwrap_or(Map::new(&env));
        let mut ids = patient_records
            .get(patient.clone())
            .unwrap_or(Vec::new(&env));
        ids.push_back(record_id);
        patient_records.set(patient.clone(), ids);
        env.storage()
            .persistent()
            .set(&PATIENT_RECORDS, &patient_records);

        Self::log_access(
            &env,
            &caller,
            &patient,
            record_id,
            String::from_str(&env, "CREATE_RECORD"),
            true,
        );

        env.events().publish(
            (Symbol::new(&env, "RecordAdded"),),
            (patient.clone(), record_id, is_confidential),
        );
        Self::trigger_ai_analysis(&env, record_id, patient);

        Ok(record_id)
    }

    pub fn get_record_with_did(
        env: Env,
        caller: Address,
        record_id: u64,
        access_purpose: String,
    ) -> Result<MedicalRecord, Error> {
        caller.require_auth();
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        if let Some(record) = records.get(record_id) {
            let patient = record.patient_id.clone();

            let has_access = Self::has_role(&env, &caller, &Role::Admin)
                || caller == record.patient_id
                || caller == record.doctor_id
                || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential)
                || Self::has_emergency_access(
                    env.clone(),
                    caller.clone(),
                    patient.clone(),
                    record_id,
                );

            Self::log_access(
                &env,
                &caller,
                &patient,
                record_id,
                access_purpose,
                has_access,
                BytesN::from_array(&env, &[0u8; 32]),
            );
            if has_access {
                Ok(record)
            } else {
                Err(Self::log_error(
                    &env,
                    Error::NotAuthorized,
                    "get_record_with_did:not_authorized",
                    Some(caller),
                ))
            }
        } else {
            Err(Error::NotAuthorized)
        }
    }

    fn trigger_ai_analysis(env: &Env, record_id: u64, patient: Address) {
        if let Ok(_config) = Self::load_ai_config(env) {
            env.events().publish(
                (Symbol::new(env, "AIAnalysisTriggered"),),
                (patient, record_id),
            );
        }
    }

    pub fn verify_professional_credential(env: Env, professional: Address) -> bool {
        let _registry: Option<Address> = env.storage().persistent().get(&DataKey::IdentityRegistry);

        Self::has_role(&env, &professional, &Role::Doctor)
    }

    pub fn set_ai_config(
        env: Env,
        caller: Address,
        ai_coordinator: Address,
        dp_epsilon: u32,
        min_participants: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }
        if min_participants == 0 {
            panic!("min_participants must be > 0");
        }
        let config = AIConfig {
            ai_coordinator,
            dp_epsilon,
            min_participants,
        };
        env.storage()
            .persistent()
            .set(&DataKey::AIConfig, &config);
        env.events().publish(
            (Symbol::new(&env, "AIConfigSet"),),
            (config.dp_epsilon, config.min_participants),
        );
        Ok(true)
    }

    pub fn get_ai_config(env: Env) -> Option<AIConfig> {
        env.storage().persistent().get(&DataKey::AIConfig)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn submit_anomaly_score(
        env: Env,
        caller: Address,
        record_id: u64,
        model_id: BytesN<32>,
        score_bps: u32,
        explanation_ref: String,
        explanation_summary: String,
        model_version: String,
        feature_importance: Vec<(String, u32)>,
    ) -> Result<bool, Error> {
        caller.require_auth();

        let _config = Self::ensure_ai_coordinator(&env, &caller)?;

        Self::validate_ai_score(score_bps)?;

        for i in 0..feature_importance.len() {
            if let Some((_, importance_bps)) = feature_importance.get(i) {
                if importance_bps > 10_000 {
                    return Err(Error::InvalidAIScore);
                }
            }
        }

        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let record = records
            .get(record_id)
            .unwrap_or_else(|| panic!("Record not found for anomaly score"));
        let patient = record.patient_id.clone();
        if explanation_ref.len() == 0 {
            panic!("explanation_ref cannot be empty");
        }
        let insight = AIInsight {
            patient: patient.clone(),
            record_id,
            model_id,
            insight_type: AIInsightType::AnomalyScore,
            score_bps,
            explanation_ref,
            explanation_summary,
            created_at: env.ledger().timestamp(),
            model_version,
            feature_importance,
        };
        env.storage().persistent().set(
            &DataKey::RecordAnomaly(patient.clone(), record_id),
            &insight,
        );
        env.events().publish(
            (Symbol::new(&env, "AIAnomalyRecorded"),),
            (patient, record_id, score_bps),
        );
        Ok(true)
    }

    pub fn get_anomaly_score(env: Env, caller: Address, record_id: u64) -> Option<AIInsight> {
        caller.require_auth();
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        let record = match records.get(record_id) {
            Some(r) => r,
            None => {
                return None;
            }
        };
        let patient = record.patient_id.clone();
        let has_access = Self::has_role(&env, &caller, &Role::Admin)
            || caller == record.patient_id
            || caller == record.doctor_id
            || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential)
            || Self::has_emergency_access(env.clone(), caller.clone(), patient.clone(), record_id);
        if !has_access {
            panic!("Unauthorized access to AI anomaly insights");
        }
        env
            .storage()
            .persistent()
            .get(&DataKey::RecordAnomaly(patient, record_id))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn submit_risk_score(
        env: Env,
        caller: Address,
        patient: Address,
        model_id: BytesN<32>,
        risk_score_bps: u32,
        explanation_ref: String,
        explanation_summary: String,
        model_version: String,
        feature_importance: Vec<(String, u32)>,
    ) -> Result<bool, Error> {
        caller.require_auth();

        let _config = Self::ensure_ai_coordinator(&env, &caller)?;

        Self::validate_ai_score(risk_score_bps)?;

        for i in 0..feature_importance.len() {
            if let Some((_, importance_bps)) = feature_importance.get(i) {
                if importance_bps > 10_000 {
                    return Err(Error::InvalidAIScore);
                }
            }
        }
        if explanation_ref.len() == 0 {
            panic!("explanation_ref cannot be empty");
        }
        let insight = AIInsight {
            patient: patient.clone(),
            record_id: 0,
            model_id,
            insight_type: AIInsightType::RiskScore,
            score_bps: risk_score_bps,
            explanation_ref,
            explanation_summary,
            created_at: env.ledger().timestamp(),
            model_version,
            feature_importance,
        };
        env.storage()
            .persistent()
            .set(&DataKey::PatientRisk(patient.clone()), &insight);
        env.events().publish(
            (Symbol::new(&env, "AIRiskRecorded"),),
            (patient, risk_score_bps),
        );
        Ok(true)
    }

    pub fn get_latest_risk_score(env: Env, caller: Address, patient: Address) -> Option<AIInsight> {
        caller.require_auth();
        if caller != patient
            && !Self::has_role(&env, &caller, &Role::Admin)
            && !Self::has_emergency_access(env.clone(), caller.clone(), patient.clone(), 0)
        {
            panic!("Unauthorized access to AI risk insights");
        }
        env.storage().persistent().get(&DataKey::PatientRisk(patient))
    }
}