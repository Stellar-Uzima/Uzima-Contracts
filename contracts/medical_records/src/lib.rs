#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, vec, Address, BytesN,
    Env, Map, String, Symbol, Vec,
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

// ==================== Core Types & Roles ====================

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
    pub credential_used: BytesN<32>, // Resolved: Fixed type from main
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
    pub authorization_credential: BytesN<32>, // Resolved: Fixed type from main
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
const BRIDGE_CONTRACT: Symbol = symbol_short!("BRIDGE");
const IDENTITY_CONTRACT: Symbol = symbol_short!("IDENTITY");
const ACCESS_CONTRACT: Symbol = symbol_short!("ACCESS");
const CROSS_CHAIN_REFS: Symbol = symbol_short!("CC_REFS");
const CROSS_CHAIN_ENABLED: Symbol = symbol_short!("CC_ON");

const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400;

// === Error Definitions ===
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
        let mut users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(&env));
        if !users.is_empty() { panic!("Contract already initialized"); }

        users.set(admin, UserProfile { role: Role::Admin, active: true, did_reference: None });
        env.storage().persistent().set(&USERS, &users);
        env.storage().persistent().set(&PAUSED, &false);
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

        // Validation Helpers from Main
        validate_address(&caller)?;
        validate_address(&patient)?;
        validate_string(&diagnosis)?;
        validate_string(&treatment)?;
        validate_string(&category)?;
        validate_string(&treatment_type)?;
        validate_string(&data_ref)?;
        for tag in tags.iter() { validate_string(&tag)?; }

        if Self::is_paused(&env) { return Err(Error::ContractPaused); }
        if !Self::has_role(&env, &caller, &Role::Doctor) { return Err(Error::NotAuthorized); }
        Self::validate_data_ref(&data_ref)?;

        let record_id = Self::get_and_increment_record_count(&env);
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
            authorization_credential: BytesN::from_array(&env, &[0u8; 32]), // Resolved merge
        };

        let mut records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        let mut patient_records: Map<Address, Vec<u64>> = env.storage().persistent().get(&PATIENT_RECORDS).unwrap_or(Map::new(&env));
        let mut ids = patient_records.get(patient.clone()).unwrap_or(Vec::new(&env));
        ids.push_back(record_id);
        patient_records.set(patient.clone(), ids);
        env.storage().persistent().set(&PATIENT_RECORDS, &patient_records);

        env.events().publish((symbol_short!("RecordAdd"),), (patient.clone(), record_id, is_confidential));
        Self::trigger_ai_analysis(&env, record_id, patient.clone());

        // MONITORING: Standardized Metric Emission
        env.events().publish((symbol_short!("METRICS"), symbol_short!("add_rec")), 1u32);

        Ok(record_id)
    }

    pub fn health_check(env: Env) -> (Symbol, u32, u64) {
        let status = if Self::is_paused(&env) { symbol_short!("PAUSED") } else { symbol_short!("OK") };
        (status, 1, env.ledger().timestamp())
    }

    // --- Accessors & Role Helpers ---
    fn has_role(env: &Env, address: &Address, role: &Role) -> bool {
        let users: Map<Address, UserProfile> = env.storage().persistent().get(&USERS).unwrap_or(Map::new(env));
        users.get(address.clone()).map_or(false, |p| {
            matches!((&p.role, role), (Role::Admin, Role::Admin) | (Role::Doctor, Role::Doctor) | (Role::Patient, Role::Patient)) && p.active
        })
    }

    fn is_paused(env: &Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

    fn get_and_increment_record_count(env: &Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&DataKey::RecordCount).unwrap_or(0);
        let next = count + 1;
        env.storage().persistent().set(&DataKey::RecordCount, &next);
        next
    }

    fn validate_data_ref(data_ref: &String) -> Result<(), Error> {
        if data_ref.len() < 10 || data_ref.len() > 200 { return Err(Error::InvalidDataRefLength); }
        Ok(())
    }

    fn trigger_ai_analysis(env: &Env, record_id: u64, patient: Address) {
        if env.storage().persistent().has(&DataKey::AIConfig) {
            env.events().publish((symbol_short!("AITrigger"),), (patient, record_id));
        }
    }
}

// ==================== Validation Helpers ====================

const MAX_STRING_LEN: u32 = 256;

fn validate_string(input: &String) -> Result<(), Error> {
    if input.len() == 0 { return Err(Error::EmptyTreatment); }
    if input.len() > MAX_STRING_LEN { return Err(Error::NumberOutOfBounds); }
    Ok(())
}

fn validate_address(addr: &Address) -> Result<(), Error> {
    addr.require_auth();
    Ok(())
}