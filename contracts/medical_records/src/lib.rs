#![no_std]

#[cfg(test)]
mod test;

mod events;

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

        users.set(admin.clone(), UserProfile { role: Role::Admin, active: true, did_reference: None });
        env.storage().persistent().set(&USERS, &users);
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

        // Emit structured event for record creation
        events::emit_record_created(&env, caller.clone(), record_id, patient.clone(), is_confidential, category.clone(), tags.clone());

        Self::trigger_ai_analysis(&env, record_id, patient.clone());

        // Emit metric update
        events::emit_metric_update(&env, "record_created", 1);

        Ok(record_id)
    }

    pub fn health_check(env: Env) -> (Symbol, u32, u64) {
        let status = if Self::is_paused(&env) { symbol_short!("PAUSED") } else { symbol_short!("OK") };
        let gas_used = env.budget().cpu_instruction_cost();
        events::emit_health_check(&env, &status.to_string(), gas_used);
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
            events::emit_ai_analysis_triggered(env, record_id, patient);
        }
    }

    pub fn propose_recovery(env: Env, caller: Address, token_contract: Address, recipient: Address, amount: i128) -> u64 {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { panic_with_error!(&env, Error::NotAuthorized); }

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
                events::emit_record_accessed(&env, caller, record_id, record.patient_id);
                Some(record)
            } else {
                None
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

    pub fn get_records_batch(env: Env, caller: Address, patient: Address, offset: u32, limit: u32) -> Vec<(u64, MedicalRecord)> {
        caller.require_auth();

        let patient_records: Map<Address, Vec<u64>> = env.storage().persistent().get(&PATIENT_RECORDS).unwrap_or(Map::new(&env));
        let record_ids = patient_records.get(patient.clone()).unwrap_or(Vec::new(&env));

        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);
        let start = offset as usize;
        let end = (start + limit as usize).min(record_ids.len());

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
                        result.push_back((record_id, record));
                        events::emit_record_accessed(&env, caller.clone(), record_id, record.patient_id);
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

        let mut successes = Vec::new(&env);
        let mut failures = Vec::new(&env);

        for (index, (patient, diagnosis, treatment, is_confidential, tags, category, treatment_type, data_ref)) in records.iter().enumerate() {
            // Validation
            if let Err(error) = validate_address(&patient) {
                failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                continue;
            }
            if let Err(error) = validate_string(&diagnosis) {
                failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                continue;
            }
            if let Err(error) = validate_string(&treatment) {
                failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                continue;
            }
            if let Err(error) = validate_string(&category) {
                failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                continue;
            }
            if let Err(error) = validate_string(&treatment_type) {
                failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                continue;
            }
            if let Err(error) = validate_string(&data_ref) {
                failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                continue;
            }
            for tag in tags.iter() {
                if let Err(error) = validate_string(&tag) {
                    failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                    continue;
                }
            }
            if let Err(error) = Self::validate_data_ref(&data_ref) {
                failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 });
                continue;
            }

            // Add record
            match Self::add_record(env.clone(), caller.clone(), patient.clone(), diagnosis.clone(), treatment.clone(), is_confidential, tags.clone(), category.clone(), treatment_type.clone(), data_ref.clone()) {
                Ok(record_id) => successes.push_back(record_id),
                Err(error) => failures.push_back(FailureInfo { index: index as u32, error_code: error as u32 }),
            }
        }

        BatchResult { successes, failures }
    }

    pub fn set_ai_config(env: Env, caller: Address, ai_coordinator: Address, dp_epsilon: u32, min_participants: u32) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) { return false; }

        let config = AIConfig {
            ai_coordinator: ai_coordinator.clone(),
            dp_epsilon,
            min_participants,
        };

        env.storage().persistent().set(&DataKey::AIConfig, &config);
        events::emit_ai_config_updated(&env, caller, ai_coordinator);
        true
    }

    pub fn get_ai_config(env: Env) -> Option<AIConfig> {
        env.storage().persistent().get(&DataKey::AIConfig)
    }

    pub fn submit_anomaly_score(
        env: Env,
        caller: Address,
        record_id: u64,
        model_id: BytesN<32>,
        score_bps: u32,
        explanation_ref: String,
        explanation_summary: String,
        model_version: String,
        feature_importance: Vec<(String, u32)>
    ) -> bool {
        caller.require_auth();

        // Check if AI coordinator
        if let Some(config) = env.storage().persistent().get(&DataKey::AIConfig) {
            if caller != config.ai_coordinator { return false; }
        } else {
            return false;
        }

        if score_bps > 10_000 { panic_with_error!(&env, Error::InvalidAIScore); }

        // Get patient from record
        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        if let Some(record) = records.get(record_id) {
            let insight = AIInsight {
                patient: record.patient_id.clone(),
                record_id,
                model_id: model_id.clone(),
                insight_type: AIInsightType::AnomalyScore,
                score_bps,
                explanation_ref: explanation_ref.clone(),
                explanation_summary: explanation_summary.clone(),
                created_at: env.ledger().timestamp(),
                model_version: model_version.clone(),
                feature_importance,
            };

            let key = DataKey::RecordAnomaly(record.patient_id.clone(), record_id);
            env.storage().persistent().set(&key, &insight);

            events::emit_anomaly_score_submitted(&env, caller, record_id, record.patient_id, model_id, score_bps, model_version);
            true
        } else {
            false
        }
    }

    pub fn get_anomaly_score(env: Env, caller: Address, record_id: u64) -> Option<AIInsight> {
        caller.require_auth();

        // Get patient from record to construct key
        let records: Map<u64, MedicalRecord> = env.storage().persistent().get(&RECORDS).unwrap_or(Map::new(&env));
        if let Some(record) = records.get(record_id) {
            // Check access (patient or admin only)
            if caller != record.patient_id && !Self::has_role(&env, &caller, &Role::Admin) {
                return None;
            }

            let key = DataKey::RecordAnomaly(record.patient_id, record_id);
            env.storage().persistent().get(&key)
        } else {
            None
        }
    }

    pub fn submit_risk_score(
        env: Env,
        caller: Address,
        patient: Address,
        model_id: BytesN<32>,
        score_bps: u32,
        explanation_ref: String,
        explanation_summary: String,
        model_version: String,
        feature_importance: Vec<(String, u32)>
    ) -> bool {
        caller.require_auth();

        // Check if AI coordinator
        if let Some(config) = env.storage().persistent().get(&DataKey::AIConfig) {
            if caller != config.ai_coordinator { return false; }
        } else {
            return false;
        }

        if score_bps > 10_000 { panic_with_error!(&env, Error::InvalidAIScore); }

        let insight = AIInsight {
            patient: patient.clone(),
            record_id: 0, // No specific record for patient-level risk
            model_id: model_id.clone(),
            insight_type: AIInsightType::RiskScore,
            score_bps,
            explanation_ref: explanation_ref.clone(),
            explanation_summary: explanation_summary.clone(),
            created_at: env.ledger().timestamp(),
            model_version: model_version.clone(),
            feature_importance,
        };

        let key = DataKey::PatientRisk(patient.clone());
        env.storage().persistent().set(&key, &insight);

        events::emit_risk_score_submitted(&env, caller, patient, model_id, score_bps, model_version);
        true
    }

    pub fn get_latest_risk_score(env: Env, caller: Address, patient: Address) -> Option<AIInsight> {
        caller.require_auth();

        // Check access (patient or admin only)
        if caller != patient && !Self::has_role(&env, &caller, &Role::Admin) {
            return None;
        }

        let key = DataKey::PatientRisk(patient);
        env.storage().persistent().get(&key)
    }

    // ==================== Event System Functions ====================

    pub fn query_events(env: Env, caller: Address, filter: events::EventFilter) -> events::EventQueryResult {
        caller.require_auth();
        // For now, return empty result as we don't have persistent event storage
        // In a real implementation, events would be stored and retrieved from storage
        events::EventQueryResult {
            events: Vec::new(&env),
            total_count: 0,
            has_more: false,
        }
    }

    pub fn get_event_stats(env: Env, caller: Address) -> events::EventStats {
        caller.require_auth();
        // For now, return empty stats as we don't have persistent event storage
        // In a real implementation, this would aggregate stored events
        events::EventStats {
            total_events: 0,
            events_by_type: Map::new(&env),
            events_by_category: Map::new(&env),
            events_by_user: Map::new(&env),
            time_range: (0, 0),
        }
    }

    pub fn get_monitoring_dashboard(env: Env, caller: Address, recent_limit: u32) -> events::MonitoringDashboard {
        caller.require_auth();
        // For now, return empty dashboard as we don't have persistent event storage
        // In a real implementation, this would create a dashboard from stored events
        events::MonitoringDashboard {
            stats: events::EventStats {
                total_events: 0,
                events_by_type: Map::new(&env),
                events_by_category: Map::new(&env),
                events_by_user: Map::new(&env),
                time_range: (0, 0),
            },
            recent_events: Vec::new(&env),
            alerts: Vec::new(&env),
            health_status: String::from_str(&env, "unknown"),
        }
    }

    pub fn replay_events(
        env: Env,
        caller: Address,
        start_time: u64,
        end_time: u64,
        event_types: Option<Vec<events::EventType>>
    ) -> Vec<events::BaseEvent> {
        caller.require_auth();
        // For now, return empty result as we don't have persistent event storage
        // In a real implementation, this would replay events from stored history
        Vec::new(&env)
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