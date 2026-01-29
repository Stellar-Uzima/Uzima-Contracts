#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

/// Permission levels for medical record access
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum PermissionLevel {
    None,
    Read,             // Can view non-confidential records
    ReadConfidential, // Can view all records
    Write,            // Can create records
    Admin,            // Full access
}

/// Supported blockchain networks
#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ChainId {
    None, // Used when chain is not specified
    Stellar,
    Ethereum,
    Polygon,
    Avalanche,
    BinanceSmartChain,
    Arbitrum,
    Optimism,
    Custom(u32),
}

/// Cross-chain access grant
#[derive(Clone)]
#[contracttype]
pub struct AccessGrant {
    pub grant_id: u64,
    pub grantor: Address,        // Patient who grants access
    pub grantee_chain: ChainId,  // Chain of the grantee
    pub grantee_address: String, // Address on external chain
    pub permission_level: PermissionLevel,
    pub record_scope: AccessScope,
    pub granted_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
    pub conditions: Vec<AccessCondition>,
}

/// Scope of access granted
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AccessScope {
    AllRecords,                    // Access to all patient's records
    SpecificRecords(Vec<u64>), // Access to specific record IDs
    CategoryBased(String),     // Access based on record category
    TimeRanged(u64, u64),      // Access to records in time range
}

/// Conditions for access
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AccessCondition {
    EmergencyOnly,                // Only for emergency access
    RequireConsent,           // Requires explicit consent each time
    AuditRequired,            // All access must be audited
    SingleUse,                // Can only be used once
    TimeRestricted(u64, u64), // Only valid during specific hours (start, end in seconds from midnight)
}

/// Access request from external chain
#[derive(Clone)]
#[contracttype]
pub struct AccessRequest {
    pub request_id: u64,
    pub requester_chain: ChainId,
    pub requester_address: String,
    pub patient: Address,
    pub requested_records: Vec<u64>,
    pub purpose: String,
    pub is_emergency: bool,
    pub created_at: u64,
    pub status: RequestStatus,
    pub decision_by: Option<Address>,
    pub decision_at: Option<u64>,
}

/// Status of access request
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum RequestStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Revoked,
}

/// Audit log entry for access events
#[derive(Clone)]
#[contracttype]
pub struct AuditEntry {
    pub entry_id: u64,
    pub accessor_chain: ChainId,
    pub accessor_address: String,
    pub patient: Address,
    pub record_id: u64,
    pub action: AccessAction,
    pub timestamp: u64,
    pub ip_hash: BytesN<32>, // Hashed IP for privacy
    pub success: bool,
}

/// Types of access actions
#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AccessAction {
    View,
    Download,
    Share,
    Export,
    EmergencyAccess,
}

/// Delegation of access management
#[derive(Clone)]
#[contracttype]
pub struct Delegation {
    pub delegator: Address,       // Patient delegating
    pub delegate: Address,        // Trusted party
    pub delegate_chain: ChainId,  // Use ChainId::None when not specified
    pub delegate_address: String, // Empty string when not specified
    pub can_grant: bool,
    pub can_revoke: bool,
    pub can_manage_emergency: bool,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
}

/// Emergency access configuration
#[derive(Clone)]
#[contracttype]
pub struct EmergencyConfig {
    pub patient: Address,
    pub is_enabled: bool,
    pub auto_approve_duration: u64, // How long emergency access lasts
    pub required_attestations: u32, // Number of validators needed
    pub trusted_providers: Vec<String>, // Pre-approved emergency providers
}

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const BRIDGE: Symbol = symbol_short!("BRIDGE");
const IDENTITY: Symbol = symbol_short!("IDENTITY");
const GRANTS: Symbol = symbol_short!("GRANTS");
const REQUESTS: Symbol = symbol_short!("REQUESTS");
const AUDIT_LOG: Symbol = symbol_short!("AUDIT");
const DELEGATIONS: Symbol = symbol_short!("DELEG");
const EMERGENCY_CONFIG: Symbol = symbol_short!("EMERG");
const PAUSED: Symbol = symbol_short!("PAUSED");
const GRANT_COUNT: Symbol = symbol_short!("GR_CNT");
const REQUEST_COUNT: Symbol = symbol_short!("REQ_CNT");
const AUDIT_COUNT: Symbol = symbol_short!("AUD_CNT");

// Constants
const DEFAULT_GRANT_DURATION: u64 = 2_592_000; // 30 days in seconds
const REQUEST_EXPIRY: u64 = 86_400; // 24 hours

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    AlreadyInitialized = 3,
    GrantNotFound = 4,
    GrantExpired = 5,
    GrantRevoked = 6,
    RequestNotFound = 7,
    RequestExpired = 8,
    RequestAlreadyProcessed = 9,
    DelegationNotFound = 10,
    DelegationExpired = 11,
    InsufficientPermissions = 12,
    EmergencyNotEnabled = 13,
    EmergencyNotAuthorized = 14,
    InvalidScope = 15,
    InvalidCondition = 16,
    AuditRequired = 17,
    SingleUseConsumed = 18,
    TimeRestrictionViolated = 19,
}

#[contract]
pub struct CrossChainAccessContract;

#[contractimpl]
#[allow(clippy::too_many_arguments)] // Suppress warning for functions with > 7 args
impl CrossChainAccessContract {
    /// Initialize the access control contract
    pub fn initialize(
        env: Env,
        admin: Address,
        bridge_contract: Address,
        identity_contract: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&BRIDGE, &bridge_contract);
        env.storage()
            .persistent()
            .set(&IDENTITY, &identity_contract);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().persistent().set(&GRANT_COUNT, &0u64);
        env.storage().persistent().set(&REQUEST_COUNT, &0u64);
        env.storage().persistent().set(&AUDIT_COUNT, &0u64);

        env.events().publish(
            (Symbol::new(&env, "AccessControlInitialized"),),
            (admin.clone(),),
        );

        Ok(true)
    }

    // ==================== Access Grant Functions ====================

    /// Grant cross-chain access to medical records
    pub fn grant_access(
        env: Env,
        grantor: Address,
        grantee_chain: ChainId,
        grantee_address: String,
        permission_level: PermissionLevel,
        record_scope: AccessScope,
        duration: u64,
        conditions: Vec<AccessCondition>,
    ) -> Result<u64, Error> {
        grantor.require_auth();
        Self::require_not_paused(env.clone())?;

        let now = env.ledger().timestamp();
        let grant_id = Self::get_and_increment_grant_count(env.clone());

        let grant = AccessGrant {
            grant_id,
            grantor: grantor.clone(),
            grantee_chain: grantee_chain.clone(),
            grantee_address: grantee_address.clone(),
            permission_level,
            record_scope,
            granted_at: now,
            expires_at: now + duration,
            is_active: true,
            conditions,
        };

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        grants.set(grant_id, grant);
        env.storage().persistent().set(&GRANTS, &grants);

        env.events().publish(
            (Symbol::new(&env, "AccessGranted"),),
            (grantor, grantee_chain, grantee_address, grant_id),
        );

        Ok(grant_id)
    }

    /// Revoke an access grant
    pub fn revoke_access(env: Env, caller: Address, grant_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(env.clone())?;

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        let mut grant = grants.get(grant_id).ok_or(Error::GrantNotFound)?;

        // Check authorization: must be grantor, admin, or authorized delegate
        if !Self::can_revoke_access(env.clone(), &caller, &grant) {
            return Err(Error::NotAuthorized);
        }

        grant.is_active = false;
        grants.set(grant_id, grant.clone());
        env.storage().persistent().set(&GRANTS, &grants);

        env.events()
            .publish((Symbol::new(&env, "AccessRevoked"),), (caller, grant_id));

        Ok(true)
    }

    /// Update access grant conditions
    pub fn update_grant_conditions(
        env: Env,
        caller: Address,
        grant_id: u64,
        new_conditions: Vec<AccessCondition>,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(env.clone())?;

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        let mut grant = grants.get(grant_id).ok_or(Error::GrantNotFound)?;

        // Only grantor can update conditions
        if caller != grant.grantor {
            return Err(Error::NotAuthorized);
        }

        grant.conditions = new_conditions;
        grants.set(grant_id, grant);
        env.storage().persistent().set(&GRANTS, &grants);

        Ok(true)
    }

    /// Extend access grant duration
    pub fn extend_grant(
        env: Env,
        caller: Address,
        grant_id: u64,
        additional_duration: u64,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(env.clone())?;

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        let mut grant = grants.get(grant_id).ok_or(Error::GrantNotFound)?;

        if caller != grant.grantor {
            return Err(Error::NotAuthorized);
        }

        grant.expires_at += additional_duration;
        grants.set(grant_id, grant);
        env.storage().persistent().set(&GRANTS, &grants);

        Ok(true)
    }

    // ==================== Access Request Functions ====================

    /// Request access from external chain (called via bridge)
    pub fn request_access(
        env: Env,
        requester_chain: ChainId,
        requester_address: String,
        patient: Address,
        requested_records: Vec<u64>,
        purpose: String,
        is_emergency: bool,
    ) -> Result<u64, Error> {
        Self::require_not_paused(env.clone())?;

        let now = env.ledger().timestamp();
        let request_id = Self::get_and_increment_request_count(env.clone());

        let request = AccessRequest {
            request_id,
            requester_chain: requester_chain.clone(),
            requester_address: requester_address.clone(),
            patient: patient.clone(),
            requested_records,
            purpose,
            is_emergency,
            created_at: now,
            status: RequestStatus::Pending,
            decision_by: None,
            decision_at: None,
        };

        let mut requests: Map<u64, AccessRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));

        requests.set(request_id, request);
        env.storage().persistent().set(&REQUESTS, &requests);

        // If emergency, check if auto-approve is possible
        if is_emergency {
            Self::handle_emergency_request(env.clone(), request_id, &requester_address, &patient)?;
        }

        env.events().publish(
            (Symbol::new(&env, "AccessRequested"),),
            (
                requester_chain,
                requester_address,
                patient,
                request_id,
                is_emergency,
            ),
        );

        Ok(request_id)
    }

    /// Approve or reject an access request
    pub fn process_request(
        env: Env,
        caller: Address,
        request_id: u64,
        approve: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(env.clone())?;

        let mut requests: Map<u64, AccessRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));

        let mut request = requests.get(request_id).ok_or(Error::RequestNotFound)?;

        // Check status
        if request.status != RequestStatus::Pending {
            return Err(Error::RequestAlreadyProcessed);
        }

        // Check expiry
        let now = env.ledger().timestamp();
        if now > request.created_at + REQUEST_EXPIRY {
            request.status = RequestStatus::Expired;
            requests.set(request_id, request);
            env.storage().persistent().set(&REQUESTS, &requests);
            return Err(Error::RequestExpired);
        }

        // Check authorization: must be patient or authorized delegate
        if !Self::can_process_request(env.clone(), &caller, &request) {
            return Err(Error::NotAuthorized);
        }

        request.status = if approve {
            RequestStatus::Approved
        } else {
            RequestStatus::Rejected
        };
        request.decision_by = Some(caller.clone());
        request.decision_at = Some(now);

        requests.set(request_id, request.clone());
        env.storage().persistent().set(&REQUESTS, &requests);

        // If approved, create temporary access grant
        if approve {
            Self::create_request_grant(env.clone(), &request)?;
        }

        env.events().publish(
            (Symbol::new(&env, "RequestProcessed"),),
            (request_id, approve, caller),
        );

        Ok(true)
    }

    // ==================== Delegation Functions ====================

    /// Delegate access management to a trusted party
    pub fn create_delegation(
        env: Env,
        delegator: Address,
        delegate: Address,
        delegate_chain: ChainId,
        delegate_address: String,
        can_grant: bool,
        can_revoke: bool,
        can_manage_emergency: bool,
        duration: u64,
    ) -> Result<bool, Error> {
        delegator.require_auth();
        Self::require_not_paused(env.clone())?;

        let now = env.ledger().timestamp();

        let delegation = Delegation {
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            delegate_chain,
            delegate_address,
            can_grant,
            can_revoke,
            can_manage_emergency,
            created_at: now,
            expires_at: now + duration,
            is_active: true,
        };

        let deleg_key = Self::delegation_key(env.clone(), &delegator, &delegate);
        let mut delegations: Map<Symbol, Delegation> = env
            .storage()
            .persistent()
            .get(&DELEGATIONS)
            .unwrap_or(Map::new(&env));

        delegations.set(deleg_key, delegation);
        env.storage().persistent().set(&DELEGATIONS, &delegations);

        env.events().publish(
            (Symbol::new(&env, "DelegationCreated"),),
            (delegator, delegate),
        );

        Ok(true)
    }

    /// Revoke a delegation
    pub fn revoke_delegation(
        env: Env,
        delegator: Address,
        delegate: Address,
    ) -> Result<bool, Error> {
        delegator.require_auth();
        Self::require_not_paused(env.clone())?;

        let deleg_key = Self::delegation_key(env.clone(), &delegator, &delegate);
        let mut delegations: Map<Symbol, Delegation> = env
            .storage()
            .persistent()
            .get(&DELEGATIONS)
            .unwrap_or(Map::new(&env));

        if let Some(mut delegation) = delegations.get(deleg_key.clone()) {
            delegation.is_active = false;
            delegations.set(deleg_key, delegation);
            env.storage().persistent().set(&DELEGATIONS, &delegations);

            env.events().publish(
                (Symbol::new(&env, "DelegationRevoked"),),
                (delegator, delegate),
            );

            Ok(true)
        } else {
            Err(Error::DelegationNotFound)
        }
    }

    // ==================== Emergency Access Functions ====================

    /// Configure emergency access settings
    pub fn configure_emergency(
        env: Env,
        patient: Address,
        is_enabled: bool,
        auto_approve_duration: u64,
        required_attestations: u32,
        trusted_providers: Vec<String>,
    ) -> Result<bool, Error> {
        patient.require_auth();
        Self::require_not_paused(env.clone())?;

        let config = EmergencyConfig {
            patient: patient.clone(),
            is_enabled,
            auto_approve_duration,
            required_attestations,
            trusted_providers,
        };

        let config_key = Self::emergency_config_key(env.clone(), &patient);
        let mut configs: Map<Symbol, EmergencyConfig> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_CONFIG)
            .unwrap_or(Map::new(&env));

        configs.set(config_key, config);
        env.storage().persistent().set(&EMERGENCY_CONFIG, &configs);

        env.events().publish(
            (Symbol::new(&env, "EmergencyConfigured"),),
            (patient, is_enabled),
        );

        Ok(true)
    }

    // ==================== Audit Functions ====================

    /// Log an access event
    pub fn log_access(
        env: Env,
        accessor_chain: ChainId,
        accessor_address: String,
        patient: Address,
        record_id: u64,
        action: AccessAction,
        ip_hash: BytesN<32>,
        success: bool,
    ) -> Result<u64, Error> {
        Self::require_not_paused(env.clone())?;

        let now = env.ledger().timestamp();
        let entry_id = Self::get_and_increment_audit_count(env.clone());

        let entry = AuditEntry {
            entry_id,
            accessor_chain: accessor_chain.clone(),
            accessor_address: accessor_address.clone(),
            patient: patient.clone(),
            record_id,
            action: action.clone(),
            timestamp: now,
            ip_hash,
            success,
        };

        let mut audit_log: Map<u64, AuditEntry> = env
            .storage()
            .persistent()
            .get(&AUDIT_LOG)
            .unwrap_or(Map::new(&env));

        audit_log.set(entry_id, entry);
        env.storage().persistent().set(&AUDIT_LOG, &audit_log);

        env.events().publish(
            (Symbol::new(&env, "AccessLogged"),),
            (accessor_chain, patient, record_id, action, success),
        );

        Ok(entry_id)
    }

    // ==================== Verification Functions ====================

    /// Verify if an entity has access to a record
    pub fn verify_access(
        env: Env,
        accessor_chain: ChainId,
        accessor_address: String,
        patient: Address,
        record_id: u64,
        required_permission: PermissionLevel,
    ) -> bool {
        // Get all active grants for this grantee
        let grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        let now = env.ledger().timestamp();

        for grant_id in 1..=Self::get_grant_count(env.clone()) {
            if let Some(grant) = grants.get(grant_id) {
                // Check if grant matches
                if grant.grantor == patient
                    && grant.grantee_chain == accessor_chain
                    && grant.grantee_address == accessor_address
                    && grant.is_active
                    && now <= grant.expires_at
                {
                    // Check permission level
                    if Self::permission_sufficient(&grant.permission_level, &required_permission) {
                        // Check scope
                        if Self::record_in_scope(&grant.record_scope, record_id) {
                            // Check conditions
                            if Self::conditions_met(env.clone(), &grant.conditions, now) {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    // ==================== Query Functions ====================

    /// Get access grant by ID
    pub fn get_grant(env: Env, grant_id: u64) -> Option<AccessGrant> {
        let grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        grants.get(grant_id)
    }

    /// Get access request by ID
    pub fn get_request(env: Env, request_id: u64) -> Option<AccessRequest> {
        let requests: Map<u64, AccessRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));

        requests.get(request_id)
    }

    /// Get delegation
    pub fn get_delegation(env: Env, delegator: Address, delegate: Address) -> Option<Delegation> {
        let deleg_key = Self::delegation_key(env.clone(), &delegator, &delegate);
        let delegations: Map<Symbol, Delegation> = env
            .storage()
            .persistent()
            .get(&DELEGATIONS)
            .unwrap_or(Map::new(&env));

        delegations.get(deleg_key)
    }

    /// Get emergency configuration
    pub fn get_emergency_config(env: Env, patient: Address) -> Option<EmergencyConfig> {
        let config_key = Self::emergency_config_key(env.clone(), &patient);
        let configs: Map<Symbol, EmergencyConfig> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_CONFIG)
            .unwrap_or(Map::new(&env));

        configs.get(config_key)
    }

    /// Get audit entry
    pub fn get_audit_entry(env: Env, entry_id: u64) -> Option<AuditEntry> {
        let audit_log: Map<u64, AuditEntry> = env
            .storage()
            .persistent()
            .get(&AUDIT_LOG)
            .unwrap_or(Map::new(&env));

        audit_log.get(entry_id)
    }

    /// Check if contract is paused
    pub fn is_paused(env: Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

    // ==================== Admin Functions ====================

    /// Pause contract
    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(env.clone(), &caller)?;

        env.storage().persistent().set(&PAUSED, &true);

        env.events().publish(
            (symbol_short!("Paused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    /// Unpause contract
    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(env.clone(), &caller)?;

        env.storage().persistent().set(&PAUSED, &false);

        env.events().publish(
            (symbol_short!("Unpaused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    // ==================== Internal Helper Functions ====================

    fn require_admin(env: Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;

        if caller != &admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: Env) -> Result<(), Error> {
        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn get_and_increment_grant_count(env: Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&GRANT_COUNT).unwrap_or(0);
        env.storage().persistent().set(&GRANT_COUNT, &(count + 1));
        count + 1
    }

    fn get_grant_count(env: Env) -> u64 {
        env.storage().persistent().get(&GRANT_COUNT).unwrap_or(0)
    }

    fn get_and_increment_request_count(env: Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&REQUEST_COUNT).unwrap_or(0);
        env.storage().persistent().set(&REQUEST_COUNT, &(count + 1));
        count + 1
    }

    fn get_and_increment_audit_count(env: Env) -> u64 {
        let count: u64 = env.storage().persistent().get(&AUDIT_COUNT).unwrap_or(0);
        env.storage().persistent().set(&AUDIT_COUNT, &(count + 1));
        count + 1
    }

    fn delegation_key(env: Env, _delegator: &Address, _delegate: &Address) -> Symbol {
        Symbol::new(&env, "deleg_key")
    }

    fn emergency_config_key(env: Env, _patient: &Address) -> Symbol {
        Symbol::new(&env, "emerg_key")
    }

    fn can_revoke_access(env: Env, caller: &Address, grant: &AccessGrant) -> bool {
        // Grantor can always revoke
        if caller == &grant.grantor {
            return true;
        }

        // Check if admin
        if let Some(admin) = env.storage().persistent().get::<Symbol, Address>(&ADMIN) {
            if caller == &admin {
                return true;
            }
        }

        // Check delegation
        let deleg_key = Self::delegation_key(env.clone(), &grant.grantor, caller);
        let delegations: Map<Symbol, Delegation> = env
            .storage()
            .persistent()
            .get(&DELEGATIONS)
            .unwrap_or(Map::new(&env));

        if let Some(delegation) = delegations.get(deleg_key) {
            let now = env.ledger().timestamp();
            return delegation.is_active && delegation.can_revoke && now <= delegation.expires_at;
        }

        false
    }

    fn can_process_request(env: Env, caller: &Address, request: &AccessRequest) -> bool {
        // Patient can always process
        if caller == &request.patient {
            return true;
        }

        // Check delegation
        let deleg_key = Self::delegation_key(env.clone(), &request.patient, caller);
        let delegations: Map<Symbol, Delegation> = env
            .storage()
            .persistent()
            .get(&DELEGATIONS)
            .unwrap_or(Map::new(&env));

        if let Some(delegation) = delegations.get(deleg_key) {
            let now = env.ledger().timestamp();
            return delegation.is_active && delegation.can_grant && now <= delegation.expires_at;
        }

        false
    }

    fn handle_emergency_request(
        env: Env,
        request_id: u64,
        requester_address: &String,
        patient: &Address,
    ) -> Result<(), Error> {
        let config_key = Self::emergency_config_key(env.clone(), patient);
        let configs: Map<Symbol, EmergencyConfig> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_CONFIG)
            .unwrap_or(Map::new(&env));

        if let Some(config) = configs.get(config_key) {
            if config.is_enabled {
                // Check if requester is a trusted provider
                if config.trusted_providers.contains(requester_address) {
                    // Auto-approve for trusted providers
                    let mut requests: Map<u64, AccessRequest> = env
                        .storage()
                        .persistent()
                        .get(&REQUESTS)
                        .unwrap_or(Map::new(&env));

                    if let Some(mut request) = requests.get(request_id) {
                        let now = env.ledger().timestamp();
                        request.status = RequestStatus::Approved;
                        request.decision_at = Some(now);
                        requests.set(request_id, request);
                        env.storage().persistent().set(&REQUESTS, &requests);

                        env.events().publish(
                            (Symbol::new(&env, "EmergencyAutoApproved"),),
                            (request_id, patient.clone()),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn create_request_grant(env: Env, request: &AccessRequest) -> Result<(), Error> {
        let now = env.ledger().timestamp();
        let grant_id = Self::get_and_increment_grant_count(env.clone());

        let grant = AccessGrant {
            grant_id,
            grantor: request.patient.clone(),
            grantee_chain: request.requester_chain.clone(),
            grantee_address: request.requester_address.clone(),
            permission_level: PermissionLevel::Read,
            record_scope: AccessScope::SpecificRecords(request.requested_records.clone()),
            granted_at: now,
            expires_at: now + DEFAULT_GRANT_DURATION,
            is_active: true,
            conditions: Vec::new(&env),
        };

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        grants.set(grant_id, grant);
        env.storage().persistent().set(&GRANTS, &grants);

        Ok(())
    }

    #[allow(clippy::match_like_matches_macro)]
    fn permission_sufficient(granted: &PermissionLevel, required: &PermissionLevel) -> bool {
        match (granted, required) {
            (PermissionLevel::Admin, _) => true,
            (PermissionLevel::Write, PermissionLevel::Write) => true,
            (PermissionLevel::Write, PermissionLevel::ReadConfidential) => true,
            (PermissionLevel::Write, PermissionLevel::Read) => true,
            (PermissionLevel::ReadConfidential, PermissionLevel::ReadConfidential) => true,
            (PermissionLevel::ReadConfidential, PermissionLevel::Read) => true,
            (PermissionLevel::Read, PermissionLevel::Read) => true,
            _ => false,
        }
    }

    fn record_in_scope(scope: &AccessScope, record_id: u64) -> bool {
        match scope {
            AccessScope::AllRecords => true,
            AccessScope::SpecificRecords(ids) => ids.iter().any(|id| id == record_id),
            AccessScope::CategoryBased(_) => true, // Would need record info to verify
            AccessScope::TimeRanged(_, _) => true, // Would need record info to verify
        }
    }

    // ... inside contracts/cross_chain_access/src/lib.rs ...

    fn conditions_met(env: Env, conditions: &Vec<AccessCondition>, now: u64) -> bool {
        for condition in conditions.iter() {
            match condition {
                AccessCondition::TimeRestricted(start, end) => {
                    // Simplified: check if current time of day is within range
                    let time_of_day = now % 86_400;
                    
                    // FIX: Removed the '*' dereference here
                    if time_of_day < start || time_of_day > end {
                        return false;
                    }
                }
                AccessCondition::SingleUse => {
                    // Would need to track usage
                    return true;
                }
                _ => {}
            }
        }
        true
    }
}
}