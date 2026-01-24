#![no_std]

#[cfg(test)]
mod test;

mod events;
mod validation;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, vec, Address, Bytes, BytesN,
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
    /// Verifiable credential used (if any)
    pub credential_used: Option<Bytes>,
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
    /// Verifiable credential ID used for authorization (if any)
    pub authorization_credential: Option<Bytes>,
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

        users.set(admin.clone(), UserProfile { role: Role::Admin, active: true, did_reference: None });
        env.storage().persistent().set(&USERS, &users);
        env.storage().persistent().set(&PAUSED, &false);

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

        // Verify caller is a doctor
        if !Self::has_role(&env, &caller, &Role::Doctor) {
            return Err(Error::NotAuthorized);
        }

        // === Comprehensive Input Validation ===

        // Validate addresses
        validation::validate_address(&env, &patient)?;
        validation::validate_address(&env, &caller)?;
        validation::validate_addresses_different(&patient, &caller)?;

        // Validate string inputs
        validation::validate_diagnosis(&diagnosis)?;
        validation::validate_treatment(&treatment)?;
        validation::validate_category(&category, &env)?;
        validation::validate_treatment_type(&treatment_type)?;
        validation::validate_data_ref(&env, &data_ref)?;
        validation::validate_tags(&tags)?;

        let record_id = Self::get_and_increment_record_count(&env);
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
            authorization_credential: None,
        };

        let mut records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        let mut patient_records: Map<Address, Vec<u64>> = env.storage().persistent().get(&PATIENT_RECORDS).unwrap_or(Map::new(&env));
        let mut ids = patient_records.get(patient.clone()).unwrap_or(Vec::new(&env));
        ids.push_back(record_id);
        patient_records.set(patient.clone(), ids);
        env.storage().persistent().set(&PATIENT_RECORDS, &patient_records);

        // Emit structured event for record creation
        events::emit_record_created(&env, caller.clone(), record_id, patient.clone(), is_confidential, category.clone(), tags.clone());

        // Trigger AI analysis for this new record
        Self::trigger_ai_analysis(&env, record_id, patient.clone());

        // Emit metric update
        events::emit_metric_update(&env, "record_created", 1);

        Ok(record_id)
    }

    pub fn health_check(env: Env) -> (Symbol, u32, u64) {
        let status = if Self::is_paused(&env) { symbol_short!("PAUSED") } else { symbol_short!("OK") };
        // let gas_used = env.budget().cpu_instruction_cost();
        let gas_used = 0;
        let status_str = String::from_str(&env, "OK");
        events::emit_health_check(&env, status_str, gas_used);
        (status, 1, env.ledger().timestamp())
    }

    fn validate_data_ref(data_ref: &String) -> Result<(), Error> {
        if data_ref.len() < 10 || data_ref.len() > 200 { return Err(Error::InvalidDataRefLength); }
        Ok(())
    }

    fn trigger_ai_analysis(env: &Env, record_id: u64, patient: Address) {
        if env.storage().persistent().has(&DataKey::AIConfig) {
            events::emit_ai_analysis_triggered(env, record_id, patient);
        }
    }

    pub fn propose_recovery(env: Env, caller: Address, token_contract: Address, recipient: Address, amount: i128) -> u64 {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { panic_with_error!(&env, Error::NotAuthorized); }

        validation::validate_amount(amount).unwrap_or_else(|e| panic_with_error!(&env, e));

        let proposal_id = Self::get_and_increment_record_count(&env);
        let proposal = RecoveryProposal {
            proposal_id,
            token_contract: token_contract.clone(),
            to: recipient.clone(),
            amount,
            created_at: env.ledger().timestamp(),
            executed: false,
            approvals: vec![&env, caller.clone()],
        };

        let mut proposals: Map<u64, RecoveryProposal> = env.storage().persistent().get(&PROPOSALS).unwrap_or(Map::new(&env));
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        events::emit_recovery_proposed(&env, caller, proposal_id, token_contract, recipient, amount);
        proposal_id
    }

    pub fn approve_recovery(env: Env, caller: Address, proposal_id: u64) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { return false; }

        let mut proposals: Map<u64, RecoveryProposal> = env.storage().persistent().get(&PROPOSALS).unwrap_or(Map::new(&env));
        if let Some(mut proposal) = proposals.get(proposal_id) {
            if proposal.executed { return false; }

            // Check if already approved
            for approver in proposal.approvals.iter() {
                if approver == caller { return false; }
            }

            proposal.approvals.push_back(caller.clone());
            proposals.set(proposal_id, proposal);
            env.storage().persistent().set(&PROPOSALS, &proposals);

            events::emit_recovery_approved(&env, caller, proposal_id);
            true
        } else {
            false
        }
    }

    pub fn execute_recovery(env: Env, caller: Address, proposal_id: u64) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { return false; }

        let mut proposals: Map<u64, RecoveryProposal> = env.storage().persistent().get(&PROPOSALS).unwrap_or(Map::new(&env));
        if let Some(mut proposal) = proposals.get(proposal_id) {
            if proposal.executed { return false; }

            // Check timelock
            let current_time = env.ledger().timestamp();
            if current_time < proposal.created_at + TIMELOCK_SECS { panic_with_error!(&env, Error::TimelockNotElasped); }

            // Check approvals
            if proposal.approvals.len() < APPROVAL_THRESHOLD { panic_with_error!(&env, Error::NotEnoughApproval); }

            proposal.executed = true;
            proposals.set(proposal_id, proposal.clone());
            env.storage().persistent().set(&PROPOSALS, &proposals);

            events::emit_recovery_executed(&env, caller, proposal_id, proposal.token_contract, proposal.to, proposal.amount);
            true
        } else {
            false
        }
    }

    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Option<MedicalRecord> {
        caller.require_auth();

        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        if let Some(record) = records.get(record_id) {
            // Check access permissions
            let can_access = if caller == record.patient_id {
                true // Patient owns the record
            } else if Self::has_role(&env, &caller, &Role::Admin) {
                true // Admin can access all records
            } else if caller == record.doctor_id {
                true // Doctor who created the record
            } else {
                false // No access
            };

            if can_access {
                events::emit_record_accessed(&env, caller, record_id, record.patient_id.clone());
                Some(record)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_records_batch(env: Env, caller: Address, patient: Address, offset: u32, limit: u32) -> Vec<(u64, MedicalRecord)> {
        caller.require_auth();

        let patient_records: Map<Address, Vec<u64>> = env.storage().persistent().get(&PATIENT_RECORDS).unwrap_or(Map::new(&env));
        let record_ids = patient_records.get(patient.clone()).unwrap_or(Vec::new(&env));

        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);
        let start = offset as usize;
        let end = (start + limit as usize).min(record_ids.len() as usize);

        for i in start..end {
            if let Some(record_id) = record_ids.get(i as u32) {
                if let Some(record) = records.get(record_id) {
                    // Check access permissions
                    let can_access = if caller == record.patient_id {
                        true // Patient owns the record
                    } else if Self::has_role(&env, &caller, &Role::Admin) {
                        true // Admin can access all records
                    } else if caller == record.doctor_id {
                        true // Doctor who created the record
                    } else if !record.is_confidential && Self::has_role(&env, &caller, &Role::Doctor) {
                        true // Non-confidential records accessible to doctors
                    } else {
                        false
                    };

                    if can_access {
                        result.push_back((record_id, record.clone()));
                        events::emit_record_accessed(&env, caller.clone(), record_id, record.patient_id.clone());
                    }
                }
            }
        }

        result
    }

    pub fn get_history(env: Env, caller: Address, patient: Address, page: u32, page_size: u32) -> Vec<(u64, MedicalRecord)> {
        Self::get_records_batch(env, caller, patient, page * page_size, page_size)
    }

    pub fn add_records_batch(
        env: Env,
        caller: Address,
        records: Vec<(Address, String, String, bool, Vec<String>, String, String, String)>
    ) -> BatchResult {
        caller.require_auth();
        if Self::is_paused(&env) { panic_with_error!(&env, Error::ContractPaused); }
        if !Self::has_role(&env, &caller, &Role::Doctor) { panic_with_error!(&env, Error::NotAuthorized); }

        // Implementation of batching would go here
        // For now returning empty result to satisfy signature
        BatchResult {
            successes: Vec::new(&env),
            failures: Vec::new(&env),
        }
    }

    // ========================================================================
    // DID INTEGRATION FUNCTIONS
    // ========================================================================

    /// Set the identity registry contract address for DID verification
    /// Only admins can configure this
    pub fn set_identity_registry(
        env: Env,
        caller: Address,
        registry_address: Address,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }

        env.storage()
            .persistent()
            .set(&DataKey::IdentityRegistry, &registry_address);

        Ok(true)
    }

    /// Retrieve the identity registry address
    pub fn get_identity_registry(env: Env) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::IdentityRegistry)
    }

    /// Helper to get a user's DID using the registry
    fn get_user_did(env: Env, user: Address) -> Option<String> {
        if let Some(_registry) = Self::get_identity_registry(env.clone()) {
            // Logic to call the registry contract would go here
            // For now, we return None as placeholder
            None
        } else {
            // Check local profile
            let users: Map<Address, UserProfile> = env
                .storage()
                .persistent()
                .get(&USERS)
                .unwrap_or(Map::new(&env));
            
            users.get(user).and_then(|p| p.did_reference)
        }
    }

    // ========================================================================
    // ACCESS AUDIT LOGGING
    // ========================================================================

    /// Log an access request for audit purposes
    fn log_access(
        env: &Env,
        requester: &Address,
        patient: &Address,
        record_id: u64,
        purpose: String,
        granted: bool,
        credential_used: Option<Bytes>,
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
            credential_used,
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

    /// Get access log entries (paginated)
    pub fn get_access_logs(
        env: Env,
        page: u32,
        page_size: u32,
    ) -> Vec<AccessRequest> {
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

    /// Get access logs for a specific patient
    pub fn get_patient_access_logs(
        env: Env,
        caller: Address,
        patient: Address,
        page: u32,
        page_size: u32,
    ) -> Vec<AccessRequest> {
        caller.require_auth();

        // Only patient, admin, or with emergency access can view logs
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

    // ========================================================================
    // DID-ENHANCED RECORD ACCESS
    // ========================================================================

    /// Add a medical record with DID verification
    /// This enhanced version includes DID and credential tracking
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
        credential_id: Option<Bytes>,
    ) -> Result<u64, Error> {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        // Verify caller is a doctor
        if !Self::has_role(&env, &caller, &Role::Doctor) {
            return Err(Error::NotAuthorized);
        }

        // Validate data_ref
        validation::validate_data_ref(&env, &data_ref)?;

        // Validate category
        validation::validate_category(&category, &env)?;

        // Validate treatment_type (non-empty)
        if treatment_type.len() == 0 {
            return Err(Error::EmptyTreatment);
        }

        // Validate tags (all non-empty)
        for tag in tags.iter() {
            if tag.len() == 0 {
                return Err(Error::EmptyTag);
            }
        }

        // Get doctor's DID if available
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
            authorization_credential: credential_id.clone(),
        };

        // Store the record
        let mut records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));
        records.set(record_id, record);
        env.storage().persistent().set(&RECORDS, &records);

        // Track record ID per patient
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

        // Log the access
        Self::log_access(
            &env,
            &caller,
            &patient,
            record_id,
            String::from_str(&env, "CREATE_RECORD"),
            true,
            credential_id,
        );

        // Emit RecordAdded event
        env.events().publish(
            (Symbol::new(&env, "RecordAdded"),),
            (patient.clone(), record_id, is_confidential),
        );

        // Trigger AI analysis for this new record
        Self::trigger_ai_analysis(&env, record_id, patient.clone());

        Ok(record_id)
    }

    /// Get a medical record with DID-based access control and logging
    pub fn get_record_with_did(
        env: Env,
        caller: Address,
        record_id: u64,
        access_purpose: String,
    ) -> Result<MedicalRecord, Error> {
        caller.require_auth();
        
        // Validate purpose
        validation::validate_purpose(&access_purpose)?;

        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));

        if let Some(record) = records.get(record_id) {
            let patient = record.patient_id.clone();

            // Check access rights
            let has_access = Self::has_role(&env, &caller, &Role::Admin)
                || caller == record.patient_id
                || caller == record.doctor_id
                || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential)
                || Self::has_emergency_access(env.clone(), caller.clone(), patient.clone(), record_id);

            // Log the access attempt
            Self::log_access(
                &env,
                &caller,
                &patient,
                record_id,
                access_purpose,
                has_access,
                None,
            );

            if has_access {
                Ok(record)
            } else {
                Err(Error::NotAuthorized)
            }
        } else {
            Err(Error::NotAuthorized)
        }
    }

    /// Verify a medical professional's credentials
    /// This would typically call the identity registry contract
    pub fn verify_professional_credential(
        env: Env,
        professional: Address,
    ) -> bool {
        // Check if identity registry is set
        let _registry: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::IdentityRegistry);

        // In a full implementation, this would:
        // 1. Call the identity registry contract
        // 2. Verify the professional has a valid DID
        // 3. Check for valid MedicalLicense or SpecialistCertification credential
        // 4. Return the verification result

        // For now, check if they have a doctor role and are active
        Self::has_role(&env, &professional, &Role::Doctor)
    }

    // ========================================================================
    // AI / ML INTEGRATION POINTS
    // ========================================================================

    /// Configure the AI coordinator and privacy parameters
    /// Only admins can call this
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

        // Validate AI configuration parameters
        validation::validate_address(&env, &ai_coordinator)?;
        validation::validate_dp_epsilon(dp_epsilon)?;
        validation::validate_min_participants(min_participants)?;

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

    /// Public view of the current AI configuration
    pub fn get_ai_config(env: Env) -> Option<AIConfig> {
        env.storage().persistent().get(&DataKey::AIConfig)
    }

    /// Record an anomaly score for a specific medical record.
    /// This is called by the AI coordinator after running off-chain models.
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

        // Ensure caller is the configured AI coordinator
        let _config = Self::ensure_ai_coordinator(&env, &caller)?;

        // Validate inputs using validation module
        validation::validate_record_id(record_id)?;
        validation::validate_score_bps(score_bps)?;
        validation::validate_data_ref(&env, &explanation_ref)?;
        validation::validate_ai_explanation(&explanation_summary, &model_version)?;
        validation::validate_feature_importance(&feature_importance)?;

        // Load the referenced medical record to derive the patient
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

    /// Retrieve the latest anomaly score for a record.
    /// Access is restricted to the same roles that can view the underlying record.
    pub fn get_anomaly_score(
        env: Env,
        caller: Address,
        record_id: u64,
    ) -> Option<AIInsight> {
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

    /// Record a predictive risk score for a patient.
    /// This represents AI-powered predictive analytics for health outcomes.
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

        // Ensure caller is the configured AI coordinator
        let _config = Self::ensure_ai_coordinator(&env, &caller)?;

        // Validate inputs using validation module
        validation::validate_address(&env, &patient)?;
        validation::validate_score_bps(risk_score_bps)?;
        validation::validate_data_ref(&env, &explanation_ref)?;
        validation::validate_ai_explanation(&explanation_summary, &model_version)?;
        validation::validate_feature_importance(&feature_importance)?;

        let insight = AIInsight {
            patient: patient.clone(),
            // 0 denotes a patient-level risk insight not tied to a single record
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

    /// Retrieve the latest risk score for a patient
    pub fn get_latest_risk_score(
        env: Env,
        caller: Address,
        patient: Address,
    ) -> Option<AIInsight> {
        caller.require_auth();

        let has_access = Self::has_role(&env, &caller, &Role::Admin)
            || caller == patient
            // Doctors can access patient risk scores
            || Self::has_role(&env, &caller, &Role::Doctor)
            || Self::has_emergency_access(env.clone(), caller.clone(), patient.clone(), 0);

        if !has_access {
            panic!("Unauthorized access to AI risk insights");
        }

        env.storage()
            .persistent()
            .get(&DataKey::PatientRisk(patient))
    }

    // ========================================================================
    // FEDERATED LEARNING SUPPORT
    // ========================================================================

    /// Register a local model update for federated learning
    /// In a real system, this would accept gradients or weights
    pub fn update_model_weights(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        round_id: u32,
        update_ref: String, // IPFS/Storage reference to the weights
    ) -> Result<bool, Error> {
        caller.require_auth();

        // Only doctors or specialized nodes can contribute updates
        if !Self::has_role(&env, &caller, &Role::Doctor) {
            return Err(Error::NotAuthorized);
        }

        validation::validate_data_ref(&env, &update_ref)?;

        // Emit an event that an update is available for aggregation
        env.events().publish(
            (Symbol::new(&env, "ModelUpdateSubmitted"),),
            (model_id, round_id, update_ref),
        );

        Ok(true)
    }

    /// Get the latest aggregated model version for client download
    pub fn get_latest_update(env: Env, _model_id: BytesN<32>) -> Option<String> {
        // In a real implementation, this would look up the latest aggregated model ref
        // For now, return a strict placeholder
        Some(String::from_str(&env, "QmPlaceholderModelHash"))
    }

    // ========================================================================
    // EMERGENCY ACCESS
    // ========================================================================

    /// Grant emergency access to a specific doctor or entity
    pub fn grant_emergency_access(
        env: Env,
        caller: Address,
        grantee: Address,
        duration_seconds: u64,
        record_ids: Vec<u64>,
    ) -> Result<bool, Error> {
        caller.require_auth();

        // Only patient can grant access primarily, or existing emergency contact?
        // For simplicity, only patient can grant
        if caller == grantee {
            return Err(Error::NotAuthorized);
        }

        // Validate duration using module
        validation::validate_duration(duration_seconds)?;

        // Validate record IDs
        validation::validate_record_ids(&record_ids)?;

        let access = EmergencyAccess {
            grantee: grantee.clone(),
            patient: caller.clone(),
            expires_at: env.ledger().timestamp() + duration_seconds,
            record_scope: record_ids,
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::EmergencyAccess(caller.clone(), grantee.clone()), &access);

        env.events().publish(
            (Symbol::new(&env, "EmergencyAccessGranted"),),
            (caller, grantee, duration_seconds),
        );

        Ok(true)
    }

    /// Revoke emergency access
    pub fn revoke_emergency_access(env: Env, caller: Address, grantee: Address) -> bool {
        caller.require_auth();

        let key = DataKey::EmergencyAccess(caller.clone(), grantee.clone());
        if env.storage().persistent().has(&key) {
            env.storage().persistent().remove(&key);
            env.events().publish(
                (Symbol::new(&env, "EmergencyAccessRevoked"),),
                (caller, grantee),
            );
            true
        } else {
            false
        }
    }

    /// Internal check for emergency access
    fn has_emergency_access(
        env: Env,
        caller: Address,
        patient: Address,
        record_id: u64,
    ) -> bool {
        let key = DataKey::EmergencyAccess(patient.clone(), caller.clone());
        
        if let Some(access) = env
            .storage()
            .persistent()
            .get::<DataKey, EmergencyAccess>(&key) 
        {
            if !access.is_active {
                return false;
            }

            if env.ledger().timestamp() > access.expires_at {
                return false;
            }

            // If record_scope is empty, it means full access
            if access.record_scope.is_empty() {
                return true;
            }

            // Check if record_id is in scope
            if record_id == 0 {
                // accessing general patient info
                return true;
            }

            for id in access.record_scope.iter() {
                if id == record_id {
                    return true;
                }
            }
        }
        
        false
    }
}