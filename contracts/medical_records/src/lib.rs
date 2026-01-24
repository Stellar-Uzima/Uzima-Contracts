#![no_std]

#[cfg(test)]
mod test;

mod validation;

use soroban_sdk::symbol_short;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, vec, Address, BytesN, Env, Map, String,
    Symbol, Vec,
};

// ==================== Cross-Chain Types ====================

/// Supported blockchain networks for cross-chain interoperability
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

/// Cross-chain record reference linking local records to external chains
#[derive(Clone)]
#[contracttype]
pub struct CrossChainRecordRef {
    pub local_record_id: u64,
    pub external_chain: ChainId,
    pub external_record_hash: BytesN<32>,
    pub sync_timestamp: u64,
    pub is_synced: bool,
}

/// Record metadata for cross-chain queries (non-sensitive data)
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

// ============================================================================
// MEDICAL RECORDS CONTRACT WITH DID INTEGRATION
// ============================================================================
// Enhanced with W3C DID-based authorization and verifiable credentials support
// ============================================================================

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
    /// Optional DID reference for this user
    pub did_reference: Option<String>,
}

/// DID-based authorization level
#[derive(Clone)]
#[contracttype]
pub enum DIDAuthLevel {
    /// No DID verification required (legacy mode)
    None,
    /// Basic DID verification - user must have active DID
    Basic,
    /// Credential verification - user must have valid medical credential
    CredentialRequired,
    /// Full verification - DID + credential + specific permission
    Full,
}

/// Access request for DID-based authorization
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AccessRequest {
    /// Requester's address
    pub requester: Address,
    /// Patient whose records are being accessed
    pub patient: Address,
    /// Specific record ID (0 for general access)
    pub record_id: u64,
    /// Purpose of access (for audit)
    pub purpose: String,
    /// Timestamp of request
    pub timestamp: u64,
    /// Whether access was granted
    pub granted: bool,
    /// Verifiable credential used (if any)
    pub credential_used: Option<BytesN<32>>,
}

/// Emergency access grant
#[derive(Clone)]
#[contracttype]
pub struct EmergencyAccess {
    /// Address granted emergency access
    pub grantee: Address,
    /// Patient who granted access
    pub patient: Address,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Scope of access (specific record IDs, or empty for all)
    pub record_scope: Vec<u64>,
    /// Whether this grant is active
    pub is_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
    /// DID of the doctor who created this record (for interoperability)
    pub doctor_did: Option<String>,
    /// Verifiable credential ID used for authorization (if any)
    pub authorization_credential: Option<BytesN<32>>,
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
    /// Score in basis points (0-10_000)
    pub score_bps: u32,
    /// Off-chain reference for detailed explanation (e.g. IPFS CID)
    pub explanation_ref: String,
    /// Human-readable explanation summary stored on-chain
    pub explanation_summary: String,
    pub created_at: u64,
    /// AI model version for reproducibility
    pub model_version: String,
    /// Feature importance data for explainable AI
    pub feature_importance: Vec<(String, u32)>, // (feature_name, importance_bps)
}

#[derive(Clone)]
#[contracttype]
pub struct AIConfig {
    /// Address of the off-chain or on-chain AI coordinator allowed to write insights
    pub ai_coordinator: Address,
    /// Differential privacy budget in epsilon units
    pub dp_epsilon: u32,
    /// Minimum number of participants required for federated rounds
    pub min_participants: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    RecordCount,
    /// Identity registry contract address for DID verification
    IdentityRegistry,
    /// DID authentication level requirement
    AuthLevel,
    /// Access audit log
    AccessLog(u64),
    /// Access log counter
    AccessLogCount,
    /// Emergency access grants
    EmergencyAccess(Address, Address), // (grantee, patient)
    /// Patient's emergency access list
    PatientEmergencyGrants(Address),
    /// Global AI configuration for medical analytics
    AIConfig,
    /// Latest risk score per patient
    PatientRisk(Address),
    /// Latest anomaly detection insight per record (patient, record_id)
    RecordAnomaly(Address, u64),
}

const USERS: Symbol = symbol_short!("USERS");
const RECORDS: Symbol = symbol_short!("RECORDS");
const PATIENT_RECORDS: Symbol = symbol_short!("PATIENT_R");
// Pausable state and recovery storage
const PAUSED: Symbol = symbol_short!("PAUSED");
const PROPOSALS: Symbol = symbol_short!("PROPOSALS");
const APPROVAL_THRESHOLD: u32 = 2;
const TIMELOCK_SECS: u64 = 86_400; // 24 hours timelock

// Cross-chain storage keys
const BRIDGE_CONTRACT: Symbol = symbol_short!("BRIDGE");
const IDENTITY_CONTRACT: Symbol = symbol_short!("IDENTITY");
const ACCESS_CONTRACT: Symbol = symbol_short!("ACCESS");
const CROSS_CHAIN_REFS: Symbol = symbol_short!("CC_REFS");
const CROSS_CHAIN_ENABLED: Symbol = symbol_short!("CC_ON");

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
    // Cross-chain errors
    CrossChainNotEnabled = 12,
    CrossChainContractsNotSet = 13,
    RecordNotFound = 14,
    CrossChainAccessDenied = 15,
    RecordAlreadySynced = 16,
    InvalidChain = 17,
    // DID-related errors
    DIDNotFound = 18,
    DIDNotActive = 19,
    InvalidCredential = 20,
    CredentialExpired = 21,
    CredentialRevoked = 22,
    MissingRequiredCredential = 23,
    EmergencyAccessExpired = 24,
    EmergencyAccessNotFound = 25,
    IdentityRegistryNotSet = 26,
    // AI-related errors
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
}

#[contract]
pub struct MedicalRecordsContract;

#[contractimpl]
impl MedicalRecordsContract {
    /// Initialize the contract with the first admin
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();

        // Ensure contract hasn't been initialized
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));
        if !users.is_empty() {
            panic!("Contract already initialized");
        }

        // Set up initial admin
        let admin_profile = UserProfile {
            role: Role::Admin,
            active: true,
            did_reference: None,
        };

        let mut users_map = Map::new(&env);
        users_map.set(admin, admin_profile);
        env.storage().persistent().set(&USERS, &users_map);

        // Initialize paused state to false
        env.storage().persistent().set(&PAUSED, &false);

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
        env
            .storage()
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


    /// Emergency pause - only admins
    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&PAUSED, &true);
        // Emit Paused event
        let ts = env.ledger().timestamp();
        env.events()
            .publish((symbol_short!("Paused"),), (caller.clone(), ts));
        Ok(true)
    }

    /// Resume operations - only admins
    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&PAUSED, &false);
        // Emit Unpaused event
        let ts = env.ledger().timestamp();
        env.events()
            .publish((symbol_short!("Unpaused"),), (caller.clone(), ts));
        Ok(true)
    }

    /// Add or update a user with a specific role
    pub fn manage_user(
        env: Env,
        caller: Address,
        user: Address,
        role: Role,
    ) -> Result<bool, Error> {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        // Only admins can manage users
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }

        let mut users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));

        // Preserve existing DID reference if user already exists
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

    /// Add a new medical record with role-based access control
    ///
    /// # Arguments
    /// * `env` - The environment
    /// * `caller` - Check who is calling the function, must be a `Doctor`
    /// * `patient` - The patient associated with the record
    /// * `diagnosis` - Minimum 1 char, max 512 chars
    /// * `treatment` - Minimum 1 char, max 512 chars
    /// * `tags` - Each tag min 1 char, max 50 chars, max 20 tags
    /// * `category` - Must be one of: Modern, Traditional, Herbal, Spiritual
    /// * `treatment_type` - Min 1 char, max 100 chars
    /// * `data_ref` - IPFS CID or similar, 10-200 chars
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

        // Block when paused
        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

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
            // DID fields - None for legacy function
            doctor_did: None,
            authorization_credential: None,
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

        // Emit RecordAdded event
        env.events().publish(
            (Symbol::new(&env, "RecordAdded"),),
            (patient.clone(), record_id, is_confidential),
        );

        // Trigger AI analysis for this new record
        Self::trigger_ai_analysis(&env, record_id, patient);

        Ok(record_id)
    }

    /// Get a medical record with role-based access control
    pub fn get_record(env: Env, caller: Address, record_id: u64) -> Option<MedicalRecord> {
        caller.require_auth();

        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));

        if let Some(record) = records.get(record_id) {
            // Allow access if:
            // 1. Caller is an admin
            // 2. Caller is the patient
            // 3. Caller is the doctor who created the record
            // 4. Caller is any doctor and record is not confidential
            if Self::has_role(&env, &caller, &Role::Admin)
                || caller == record.patient_id
                || caller == record.doctor_id
                || (Self::has_role(&env, &caller, &Role::Doctor) && !record.is_confidential)
            {
                Some(record)
            } else {
                panic!("Unauthorized access to medical record");
            }
        } else {
            None
        }
    }

    /// Retrieve paginated history of records for a patient with access control
    pub fn get_history(
        env: Env,
        caller: Address,
        patient: Address,
        page: u32,
        page_size: u32,
    ) -> Vec<(u64, MedicalRecord)> {
        caller.require_auth();

        // Validate pagination parameters
        if validation::validate_pagination(page, page_size).is_err() {
            panic!("Invalid pagination parameters");
        }

        // Block when paused (optional; reads are generally allowed during pause)
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

        let patient_records: Map<Address, Vec<u64>> = env
            .storage()
            .persistent()
            .get(&PATIENT_RECORDS)
            .unwrap_or(Map::new(&env));
        let ids = patient_records.get(patient).unwrap_or(Vec::new(&env));

        // Pagination: calculate start and end indices
        let start = page * page_size;
        let end = ((page + 1) * page_size).min(ids.len() as u32) as usize;

        if start >= ids.len() as u32 {
            return Vec::new(&env); // Empty page
        }

        // Gas bounding: limit total records fetched (e.g., max 100 per call)
        let max_fetch = 100u32.min(page_size * 2); // Conservative bound
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

        history
    }

    /// Deactivate a user
    pub fn deactivate_user(env: Env, caller: Address, user: Address) -> bool {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }

        // Only admins can deactivate users
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
            true
        } else {
            false
        }
    }

    /// Get the role of a user by address (public key)
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

    // ==================== Cross-Chain Functions ====================

    /// Configure cross-chain contract references (admin only)
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

    /// Enable or disable cross-chain functionality (admin only)
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

    /// Check if cross-chain functionality is enabled
    pub fn is_cross_chain_enabled(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&CROSS_CHAIN_ENABLED)
            .unwrap_or(false)
    }

    /// Get cross-chain contract addresses
    pub fn get_cross_chain_contracts(env: Env) -> Option<(Address, Address, Address)> {
        let bridge: Option<Address> = env.storage().persistent().get(&BRIDGE_CONTRACT);
        let identity: Option<Address> = env.storage().persistent().get(&IDENTITY_CONTRACT);
        let access: Option<Address> = env.storage().persistent().get(&ACCESS_CONTRACT);

        match (bridge, identity, access) {
            (Some(b), Some(i), Some(a)) => Some((b, i, a)),
            _ => None,
        }
    }

    /// Get record metadata for cross-chain queries (non-sensitive data only)
    /// This allows external chains to query basic record information
    pub fn get_record_metadata(env: Env, record_id: u64) -> Result<RecordMetadata, Error> {
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));

        let record = records.get(record_id).ok_or(Error::RecordNotFound)?;

        // Generate a hash of the record content for verification
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

    /// Register a cross-chain record reference (links local record to external chain)
    pub fn register_cross_chain_ref(
        env: Env,
        caller: Address,
        local_record_id: u64,
        external_chain: ChainId,
        external_record_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        // Check cross-chain is enabled
        if !Self::is_cross_chain_enabled(env.clone()) {
            return Err(Error::CrossChainNotEnabled);
        }

        // Verify record exists
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));

        let record = records.get(local_record_id).ok_or(Error::RecordNotFound)?;

        // Only patient, doctor who created the record, or admin can register cross-chain refs
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

        // Store cross-chain reference
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

    /// Get cross-chain record reference
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

    /// Get record for cross-chain access (called via bridge after access verification)
    /// This is the main entry point for cross-chain record retrieval
    pub fn get_record_cross_chain(
        env: Env,
        bridge_caller: Address,
        record_id: u64,
        accessor_chain: ChainId,
        accessor_address: String,
    ) -> Result<MedicalRecord, Error> {
        bridge_caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        // Check cross-chain is enabled
        if !Self::is_cross_chain_enabled(env.clone()) {
            return Err(Error::CrossChainNotEnabled);
        }

        // Verify caller is the bridge contract
        let bridge: Address = env
            .storage()
            .persistent()
            .get(&BRIDGE_CONTRACT)
            .ok_or(Error::CrossChainContractsNotSet)?;

        if bridge_caller != bridge {
            return Err(Error::NotAuthorized);
        }

        // Get the record
        let records: Map<u64, MedicalRecord> = env
            .storage()
            .persistent()
            .get(&RECORDS)
            .unwrap_or(Map::new(&env));

        let record = records.get(record_id).ok_or(Error::RecordNotFound)?;

        // Emit cross-chain access event for audit
        env.events().publish(
            (Symbol::new(&env, "CrossChainRecordAccess"),),
            (record_id, accessor_chain, accessor_address),
        );

        Ok(record)
    }

    /// Update cross-chain sync status for a record
    pub fn update_cross_chain_sync(
        env: Env,
        caller: Address,
        local_record_id: u64,
        external_chain: ChainId,
        new_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        caller.require_auth();

        // Block when paused
        if Self::is_paused(&env) {
            return Err(Error::ContractPaused);
        }

        // Check cross-chain is enabled
        if !Self::is_cross_chain_enabled(env.clone()) {
            return Err(Error::CrossChainNotEnabled);
        }

        // Only admin or bridge can update sync status
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

    /// Get all cross-chain references for a record
    pub fn get_all_cross_chain_refs(env: Env, local_record_id: u64) -> Vec<CrossChainRecordRef> {
        let refs: Map<Symbol, CrossChainRecordRef> = env
            .storage()
            .persistent()
            .get(&CROSS_CHAIN_REFS)
            .unwrap_or(Map::new(&env));

        let mut result = Vec::new(&env);

        // Check common chains
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

    // ==================== Internal Cross-Chain Helpers ====================

    fn cross_chain_ref_key(env: &Env, _record_id: u64, _chain: &ChainId) -> Symbol {
        // Create a unique key combining record ID and chain
        // Using a simplified key due to Symbol limitations
        Symbol::new(env, "cc_ref")
    }

    fn compute_record_hash(env: &Env, record: &MedicalRecord) -> BytesN<32> {
        // Compute a hash of the record content for integrity verification
        // In a production environment, this would use proper cryptographic hashing
        // For now, we use a placeholder that combines key fields
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
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        )
    }

    // ------------------ Recovery (timelock + multisig) ------------------

    /// Propose a recovery operation (e.g., recover tokens sent to this contract)
    pub fn propose_recovery(
        env: Env,
        caller: Address,
        token_contract: Address,
        to: Address,
        amount: i128,
    ) -> u64 {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can propose recovery");
        }
        
        // Validate inputs
        if validation::validate_address(&env, &token_contract).is_err() || 
           validation::validate_address(&env, &to).is_err() {
            panic!("Invalid address");
        }
        
        if validation::validate_amount(amount).is_err() {
            panic!("Invalid amount");
        }

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
        // Initial approval by proposer for convenience
        proposal.approvals.push_back(caller.clone());

        let mut proposals: Map<u64, RecoveryProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        proposal_id
    }

    /// Approve a recovery proposal (admin only)
    pub fn approve_recovery(env: Env, caller: Address, proposal_id: u64) -> bool {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            panic!("Only admins can approve recovery");
        }

        let mut proposals: Map<u64, RecoveryProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));
        let mut proposal = proposals
            .get(proposal_id)
            .unwrap_or_else(|| panic!("Proposal not found"));
        if proposal.executed {
            panic!("Proposal already executed");
        }
        // Prevent duplicate approvals
        if proposal.approvals.iter().any(|a| a == caller) {
            return true;
        }
        proposal.approvals.push_back(caller.clone());
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);
        true
    }

    /// Execute recovery after timelock and approvals threshold
    pub fn execute_recovery(env: Env, caller: Address, proposal_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }

        let mut proposals: Map<u64, RecoveryProposal> = env
            .storage()
            .persistent()
            .get(&PROPOSALS)
            .unwrap_or(Map::new(&env));
        let mut proposal = proposals
            .get(proposal_id)
            .unwrap_or_else(|| panic!("Proposal not found"));
        if proposal.executed {
            return Err(Error::ProposalAlreadyExecuted);
        }

        // Check timelock
        let now = env.ledger().timestamp();
        if now < proposal.created_at + TIMELOCK_SECS {
            return Err(Error::TimelockNotElasped);
        }

        // Check multisig approvals
        let distinct_approvals = proposal.approvals.len();
        if (distinct_approvals as u32) < APPROVAL_THRESHOLD {
            return Err(Error::NotEnoughApproval);
        }

        // In actual implementation, we would invoke the token contract transfer here.
        // For auditability within this project scope, we just mark executed and emit an event.
        proposal.executed = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&PROPOSALS, &proposals);

        // Emit RecoveryExecuted event
        let _ts = env.ledger().timestamp();
        // env.events().publish((symbol_short!("RecoveryExecuted"),), (caller.clone(), proposal_id, _ts));
        Ok(true)
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

        env.events().publish(
            (Symbol::new(&env, "IdentityRegistrySet"),),
            registry_address,
        );

        Ok(true)
    }

    /// Set the DID authentication level required for operations
    pub fn set_did_auth_level(
        env: Env,
        caller: Address,
        level: DIDAuthLevel,
    ) -> Result<bool, Error> {
        caller.require_auth();

        if !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
        }

        env.storage().persistent().set(&DataKey::AuthLevel, &level);

        env.events().publish(
            (Symbol::new(&env, "AuthLevelSet"),),
            level,
        );

        Ok(true)
    }

    /// Get the current DID authentication level
    pub fn get_did_auth_level(env: Env) -> DIDAuthLevel {
        env.storage()
            .persistent()
            .get(&DataKey::AuthLevel)
            .unwrap_or(DIDAuthLevel::None)
    }

    /// Get the identity registry address
    pub fn get_identity_registry(env: Env) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::IdentityRegistry)
    }

    /// Link a DID to a user profile
    pub fn link_did_to_user(
        env: Env,
        caller: Address,
        user: Address,
        did_reference: String,
    ) -> Result<bool, Error> {
        caller.require_auth();

        // Validate DID reference
        validation::validate_did_reference(&did_reference)?;

        // Only admins can link DIDs, or users can link their own
        if caller != user && !Self::has_role(&env, &caller, &Role::Admin) {
            return Err(Error::NotAuthorized);
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

            env.events().publish(
                (Symbol::new(&env, "DIDLinked"),),
                (user, did_reference),
            );

            Ok(true)
        } else {
            Err(Error::NotAuthorized)
        }
    }

    /// Get a user's linked DID
    pub fn get_user_did(env: Env, user: Address) -> Option<String> {
        let users: Map<Address, UserProfile> = env
            .storage()
            .persistent()
            .get(&USERS)
            .unwrap_or(Map::new(&env));

        users.get(user).and_then(|p| p.did_reference)
    }

    // ========================================================================
    // EMERGENCY ACCESS MANAGEMENT
    // ========================================================================

    /// Grant emergency access to a healthcare provider
    /// Only patients can grant emergency access to their records
    ///
    /// # Arguments
    /// * `duration_secs` - Must be positive and max 1 year (31,536,000 seconds)
    pub fn grant_emergency_access(
        env: Env,
        patient: Address,
        grantee: Address,
        duration_secs: u64,
        record_scope: Vec<u64>,
    ) -> Result<bool, Error> {
        patient.require_auth();

        // Validate addresses
        validation::validate_address(&env, &grantee)?;
        validation::validate_addresses_different(&patient, &grantee)?;

        // Verify patient role
        if !Self::has_role(&env, &patient, &Role::Patient) {
            return Err(Error::NotAuthorized);
        }

        // Validate duration (must be positive and reasonable)
        if duration_secs == 0 || duration_secs > 31_536_000 {
            // Max 1 year
            return Err(Error::NotAuthorized);
        }

        let now = env.ledger().timestamp();
        let expires_at = now + duration_secs;

        let emergency_access = EmergencyAccess {
            grantee: grantee.clone(),
            patient: patient.clone(),
            expires_at,
            record_scope: record_scope.clone(),
            is_active: true,
        };

        // Store the emergency access grant
        env.storage().persistent().set(
            &DataKey::EmergencyAccess(grantee.clone(), patient.clone()),
            &emergency_access,
        );

        // Add to patient's grant list
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

    /// Revoke emergency access
    pub fn revoke_emergency_access(
        env: Env,
        patient: Address,
        grantee: Address,
    ) -> Result<bool, Error> {
        patient.require_auth();

        let key = DataKey::EmergencyAccess(grantee.clone(), patient.clone());

        if let Some(mut access) = env
            .storage()
            .persistent()
            .get::<DataKey, EmergencyAccess>(&key)
        {
            access.is_active = false;
            env.storage().persistent().set(&key, &access);

            env.events().publish(
                (Symbol::new(&env, "EmergencyAccessRevoked"),),
                (patient, grantee),
            );

            Ok(true)
        } else {
            Err(Error::EmergencyAccessNotFound)
        }
    }

    /// Check if someone has valid emergency access
    pub fn has_emergency_access(
        env: Env,
        grantee: Address,
        patient: Address,
        record_id: u64,
    ) -> bool {
        let key = DataKey::EmergencyAccess(grantee, patient);

        if let Some(access) = env
            .storage()
            .persistent()
            .get::<DataKey, EmergencyAccess>(&key)
        {
            if !access.is_active {
                return false;
            }

            let now = env.ledger().timestamp();
            if now > access.expires_at {
                return false;
            }

            // Check scope - empty means all records
            if access.record_scope.is_empty() {
                return true;
            }

            // Check if record_id is in scope
            access.record_scope.iter().any(|id| id == record_id)
        } else {
            false
        }
    }

    /// Get all active emergency access grants for a patient
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
        credential_used: Option<BytesN<32>>,
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
        credential_id: Option<BytesN<32>>,
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
        let allowed_categories = vec![
            &env,
            String::from_str(&env, "Modern"),
            String::from_str(&env, "Traditional"),
            String::from_str(&env, "Herbal"),
            String::from_str(&env, "Spiritual"),
        ];
        if !allowed_categories.contains(&category) {
            return Err(Error::InvalidCategory);
        }

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
            (patient, record_id, is_confidential),
        );

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

    /// Internal function to trigger AI analysis when a new record is added
    fn trigger_ai_analysis(env: &Env, record_id: u64, patient: Address) {
        // Check if AI analysis is enabled/configured
        if let Ok(config) = Self::load_ai_config(env) {
            // In a real implementation, this would trigger an asynchronous
            // analysis job with the AI coordinator
            // For now, we just emit an event to signal that analysis should be triggered
            env.events().publish(
                (Symbol::new(env, "AIAnalysisTriggered"),),
                (patient, record_id),
            );
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

    /// Retrieve the latest patient-level AI risk score.
    /// Only the patient, admins, or emergency grantees can view this insight.
    pub fn get_latest_risk_score(
        env: Env,
        caller: Address,
        patient: Address,
    ) -> Option<AIInsight> {
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
