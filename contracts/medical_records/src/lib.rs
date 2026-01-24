#![no_std]

#[cfg(test)]
mod test;

mod events;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, vec, Address, BytesN,
    Env, Map, String, Symbol, Vec,
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

#[derive(Clone)]
#[contracttype]
pub enum AIInsightType {
    AnomalyScore,
    RiskScore,
}

#[derive(Clone)]
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
}

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
    BatchTooLarge = 30,
    InvalidBatch = 31,
    InvalidInput = 32,
    NumberOutOfBounds = 33,
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
impl MedicalRecordsContract {
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();
        env.storage().persistent().set(&PAUSED, &false);
        // Emit user creation event
        events::emit_user_created(&env, admin, admin, "Admin", None);
        true
    }

    pub fn manage_user(env: Env, caller: Address, user: Address, role: Role) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { return false; }

        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
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
            users.set(user.clone(), UserProfile { role: role.clone(), active: true, did_reference: profile.did_reference });
            events::emit_user_role_updated(&env, caller, user, role_str, Some(previous_role_str));
        } else {
            // Create new user
            users.set(user.clone(), UserProfile { role: role.clone(), active: true, did_reference: None });
            events::emit_user_created(&env, caller, user, role_str, None);
        }

        env.storage().persistent().set(&USERS, &users);
        true
    }

    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { return false; }

        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
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
        if !Self::has_role(&env, &caller, &Role::Admin) { return false; }

        env.storage().persistent().set(&PAUSED, &true);
        events::emit_contract_paused(&env, caller);
        true
    }

    pub fn unpause(env: Env, caller: Address) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { return false; }

        env.storage().persistent().set(&PAUSED, &false);
        events::emit_contract_unpaused(&env, caller);
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
        data_ref: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        let record = MedicalRecord {
            patient_id: patient.clone(),
            doctor_id: caller.clone(),
            timestamp: env.ledger().timestamp(),
            diagnosis,
            treatment,
            is_confidential,
            tags,
            category,
            treatment_type,
            data_ref,
            doctor_did: None,
}