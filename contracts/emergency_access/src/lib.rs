#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, String,
    Vec,
};

use soroban_sdk::vec;
use soroban_sdk::IntoVal;
// ==================== Emergency Access Types ====================

/// Emergency Access Request Status
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum EmergencyRequestStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Revoked,
    Disputed,
}

/// Multi-Factor Authentication Level
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum MFAFactor {
    Password,
    Biometric,
    HardwareToken,
    DIDVerification,
    MedicalLicense,
}

/// Emergency Access Request
#[derive(Clone)]
#[contracttype]
pub struct EmergencyRequest {
    pub request_id: u64,
    pub requester: Address,          // Healthcare provider requesting access
    pub patient: Address,            // Patient whose records are needed
    pub emergency_type: String,      // Type of emergency (cardiac, trauma, etc.)
    pub justification: String,       // Clinical justification
    pub record_scope: Vec<u64>,      // Specific record IDs (empty = all records)
    pub requested_duration: u64,     // Requested access duration in seconds
    pub mfa_factors: Vec<MFAFactor>, // MFA factors provided
    pub status: EmergencyRequestStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub approved_by: Vec<Address>, // Approvers
    pub required_approvals: u32,   // Number of approvals needed
}

/// Emergency Access Grant
#[derive(Clone)]
#[contracttype]
pub struct EmergencyGrant {
    pub grant_id: u64,
    pub request_id: u64,
    pub grantee: Address,
    pub patient: Address,
    pub record_scope: Vec<u64>,
    pub granted_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
    pub access_count: u32,
    pub last_access: u64,
}

/// Emergency Authority Configuration
#[derive(Clone)]
#[contracttype]
pub struct EmergencyAuthority {
    pub address: Address,
    pub role: String,      // "doctor", "nurse", "admin", "emergency_coordinator"
    pub specialty: String, // Medical specialty
    pub license_number: String,
    pub is_active: bool,
    pub approval_weight: u32, // Weight for approval quorum
}

/// Compliance Report for Emergency Access
#[derive(Clone)]
#[contracttype]
pub struct EmergencyComplianceReport {
    pub incident_id: u64,
    pub request_id: u64,
    pub grant_id: u64,
    pub patient: Address,
    pub requester: Address,
    pub approvers: Vec<Address>,
    pub access_start: u64,
    pub access_end: u64,
    pub records_accessed: Vec<u64>,
    pub compliance_violations: Vec<String>,
    pub audit_trail: Vec<String>,
}

// ==================== Contract Storage Keys ====================

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    RequestCounter,
    GrantCounter,
    IncidentCounter,
    EmergencyAuthorities,
    PendingRequests(Address),  // patient -> requests
    ActiveGrants(Address),     // patient -> grants
    RequestDetails(u64),       // request_id -> details
    GrantDetails(u64),         // grant_id -> details
    AuthorityDetails(Address), // address -> authority info
    ComplianceReports(u64),    // incident_id -> report
    ExpiredGrants,             // For cleanup
}

// ==================== Contract Configuration ====================

#[derive(Clone)]
#[contracttype]
pub struct EmergencyConfig {
    pub admin: Address,
    pub medical_records_contract: Address,
    pub identity_registry_contract: Address,
    pub governor_contract: Address,
    pub dispute_contract: Address,
    pub compliance_contract: Address,
    pub max_request_duration: u64,   // Maximum allowed duration
    pub min_approvals_required: u32, // Minimum approvals needed
    pub emergency_cooldown: u64,     // Cooldown between requests for same patient
    pub auto_expiration_enabled: bool,
    pub mfa_required: bool,
    pub audit_enabled: bool,
}

// ==================== Error Definitions ====================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    NotEmergencyAuthority = 4,
    RequestNotFound = 5,
    GrantNotFound = 6,
    RequestAlreadyProcessed = 7,
    InsufficientApprovals = 8,
    InvalidDuration = 9,
    MFANotSatisfied = 10,
    EmergencyCooldownActive = 11,
    PatientNotFound = 12,
    AuthorityNotActive = 13,
    InvalidEmergencyType = 14,
    AccessExpired = 15,
    GrantRevoked = 16,
    DisputePending = 17,
    ComplianceViolation = 18,
}

// ==================== Contract Implementation ====================

#[contract]
pub struct EmergencyAccess;

#[contractimpl]
impl EmergencyAccess {
    /// Initialize the emergency access contract
    pub fn initialize(
        env: Env,
        admin: Address,
        medical_records_contract: Address,
        identity_registry_contract: Address,
        governor_contract: Address,
        dispute_contract: Address,
        compliance_contract: Address,
        max_request_duration: u64,
        min_approvals_required: u32,
        emergency_cooldown: u64,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(Error::AlreadyInitialized);
        }

        let config = EmergencyConfig {
            admin: admin.clone(),
            medical_records_contract,
            identity_registry_contract,
            governor_contract,
            dispute_contract,
            compliance_contract,
            max_request_duration,
            min_approvals_required,
            emergency_cooldown,
            auto_expiration_enabled: true,
            mfa_required: true,
            audit_enabled: true,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage()
            .instance()
            .set(&DataKey::RequestCounter, &0u64);
        env.storage().instance().set(&DataKey::GrantCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::IncidentCounter, &0u64);

        // Initialize empty collections
        env.storage().persistent().set(
            &DataKey::EmergencyAuthorities,
            &Map::<Address, EmergencyAuthority>::new(&env),
        );
        env.storage()
            .persistent()
            .set(&DataKey::ExpiredGrants, &Vec::<u64>::new(&env));

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("init")),
            // (admin, env.ledger().timestamp()),
            (admin.clone(), env.ledger().timestamp()),
        );

        Ok(())
    }

    /// Register an emergency authority
    pub fn register_emergency_authority(
        env: Env,
        caller: Address,
        authority_address: Address,
        role: String,
        specialty: String,
        license_number: String,
        approval_weight: u32,
    ) -> Result<(), Error> {
        caller.require_auth();
        let config = Self::get_config(&env)?;
        if caller != config.admin {
            return Err(Error::NotAuthorized);
        }

        let authority = EmergencyAuthority {
            address: authority_address.clone(),
            role,
            specialty,
            license_number,
            is_active: true,
            approval_weight,
        };

        let mut authorities = env
            .storage()
            .persistent()
            .get(&DataKey::EmergencyAuthorities)
            .unwrap_or(Map::new(&env));
        authorities.set(authority_address.clone(), authority);
        env.storage()
            .persistent()
            .set(&DataKey::EmergencyAuthorities, &authorities);

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("auth_reg")),
            (authority_address, env.ledger().timestamp()),
        );

        Ok(())
    }

    /// Submit emergency access request
    pub fn request_emergency_access(
        env: Env,
        caller: Address,
        patient: Address,
        emergency_type: String,
        justification: String,
        record_scope: Vec<u64>,
        requested_duration: u64,
        mfa_factors: Vec<MFAFactor>,
    ) -> Result<u64, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_emergency_authority(&env, &caller)?;
        Self::validate_emergency_request(&env, requested_duration, &emergency_type)?;

        // Check cooldown
        if Self::is_emergency_cooldown_active(&env, &caller, &patient) {
            return Err(Error::EmergencyCooldownActive);
        }

        // Validate MFA if required
        let config = Self::get_config(&env)?;
        if config.mfa_required && !Self::validate_mfa_factors(&mfa_factors) {
            return Err(Error::MFANotSatisfied);
        }

        let request_id = Self::increment_counter(&env, &DataKey::RequestCounter);

        let request = EmergencyRequest {
            request_id,
            requester: caller.clone(),
            patient: patient.clone(),
            emergency_type,
            justification,
            record_scope,
            requested_duration,
            mfa_factors,
            status: EmergencyRequestStatus::Pending,
            created_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp().saturating_add(requested_duration),
            approved_by: Vec::new(&env),
            required_approvals: config.min_approvals_required,
        };

        // Store request details
        env.storage()
            .persistent()
            .set(&DataKey::RequestDetails(request_id), &request);

        // Add to pending requests for patient
        let mut pending = env
            .storage()
            .persistent()
            .get(&DataKey::PendingRequests(patient.clone()))
            .unwrap_or(Vec::new(&env));
        pending.push_back(request_id);
        env.storage()
            .persistent()
            .set(&DataKey::PendingRequests(patient.clone()), &pending);

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("req_sub")),
            (request_id, caller, patient, env.ledger().timestamp()),
        );

        Ok(request_id)
    }

    /// Approve emergency access request
    pub fn approve_emergency_request(
        env: Env,
        caller: Address,
        request_id: u64,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;
        Self::require_emergency_authority(&env, &caller)?;

        let mut request: EmergencyRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RequestDetails(request_id))
            .ok_or(Error::RequestNotFound)?;

        if request.status != EmergencyRequestStatus::Pending {
            return Err(Error::RequestAlreadyProcessed);
        }

        // Check if already approved by this authority
        for i in 0..request.approved_by.len() {
            if request.approved_by.get(i).unwrap() == caller {
                return Err(Error::NotAuthorized); // Already approved
            }
        }

        request.approved_by.push_back(caller.clone());

        // Check if we have enough approvals
        if request.approved_by.len() >= request.required_approvals as u32 {
            request.status = EmergencyRequestStatus::Approved;
            Self::create_emergency_grant(&env, &request)?;
        }

        env.storage()
            .persistent()
            .set(&DataKey::RequestDetails(request_id), &request);

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("req_appr")),
            (request_id, caller, env.ledger().timestamp()),
        );

        Ok(())
    }

    /// Check if emergency access is granted
    pub fn has_emergency_access(
        env: Env,
        grantee: Address,
        patient: Address,
        record_id: u64,
    ) -> bool {
        let grants: Map<Address, Vec<EmergencyGrant>> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveGrants(patient))
            .unwrap_or(Map::new(&env));

        let patient_grants = match grants.get(grantee) {
            Some(g) => g,
            None => return false,
        };

        let now = env.ledger().timestamp();
        for i in 0..patient_grants.len() {
            let grant = patient_grants.get(i).unwrap();
            if grant.is_active && grant.expires_at > now {
                if grant.record_scope.is_empty() || grant.record_scope.contains(record_id) {
                    return true;
                }
            }
        }
        false
    }

    /// Revoke emergency access grant
    pub fn revoke_emergency_access(env: Env, caller: Address, grant_id: u64) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let mut grant: EmergencyGrant = env
            .storage()
            .persistent()
            .get(&DataKey::GrantDetails(grant_id))
            .ok_or(Error::GrantNotFound)?;

        // Only admin or the requester can revoke
        let config = Self::get_config(&env)?;
        let request: EmergencyRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RequestDetails(grant.request_id))
            .ok_or(Error::RequestNotFound)?;

        if caller != config.admin && caller != request.requester {
            return Err(Error::NotAuthorized);
        }

        grant.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::GrantDetails(grant_id), &grant);

        // Update active grants
        Self::update_active_grants(&env, &grant.patient, &grant.grantee);

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("grant_rev")),
            (grant_id, caller, env.ledger().timestamp()),
        );

        Ok(())
    }

    /// Dispute emergency access grant
    pub fn dispute_emergency_access(
        env: Env,
        caller: Address,
        grant_id: u64,
        reason: String,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let grant: EmergencyGrant = env
            .storage()
            .persistent()
            .get(&DataKey::GrantDetails(grant_id))
            .ok_or(Error::GrantNotFound)?;

        // Call dispute resolution contract
        let _config = Self::get_config(&env)?;
        // Note: This would integrate with the dispute_resolution contract
        // For now, we'll mark as disputed
        let mut request: EmergencyRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RequestDetails(grant.request_id))
            .ok_or(Error::RequestNotFound)?;

        request.status = EmergencyRequestStatus::Disputed;
        env.storage()
            .persistent()
            .set(&DataKey::RequestDetails(grant.request_id), &request);

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("grant_dp")),
            (grant_id, caller, reason, env.ledger().timestamp()),
        );

        Ok(())
    }

    /// Generate compliance report
    pub fn generate_compliance_report(
        env: Env,
        caller: Address,
        grant_id: u64,
    ) -> Result<EmergencyComplianceReport, Error> {
        caller.require_auth();
        let config = Self::get_config(&env)?;
        if caller != config.admin {
            return Err(Error::NotAuthorized);
        }

        let grant: EmergencyGrant = env
            .storage()
            .persistent()
            .get(&DataKey::GrantDetails(grant_id))
            .ok_or(Error::GrantNotFound)?;

        let request: EmergencyRequest = env
            .storage()
            .persistent()
            .get(&DataKey::RequestDetails(grant.request_id))
            .ok_or(Error::RequestNotFound)?;

        let incident_id = Self::increment_counter(&env, &DataKey::IncidentCounter);

        let report = EmergencyComplianceReport {
            incident_id,
            request_id: grant.request_id,
            grant_id,
            patient: grant.patient,
            requester: request.requester,
            approvers: request.approved_by,
            access_start: grant.granted_at,
            access_end: grant.expires_at,
            records_accessed: grant.record_scope,
            compliance_violations: Vec::new(&env), // Would be populated based on audit
            audit_trail: vec![
                &env,
                "Emergency access granted".into_val(&env),
                "Access logged".into_val(&env),
            ],
        };

        env.storage()
            .persistent()
            .set(&DataKey::ComplianceReports(incident_id), &report);

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("comp_rpt")),
            (incident_id, grant_id, env.ledger().timestamp()),
        );

        Ok(report)
    }

    /// Cleanup expired grants
    pub fn cleanup_expired_grants(env: Env, caller: Address) -> Result<u32, Error> {
        caller.require_auth();
        let config = Self::get_config(&env)?;
        if caller != config.admin {
            return Err(Error::NotAuthorized);
        }

        let now = env.ledger().timestamp();
        let mut cleaned = 0u32;

        // Get all patients with active grants (this is simplified - in practice would need indexing)
        // For now, we'll iterate through known grants
        let grant_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::GrantCounter)
            .unwrap_or(0);

        for grant_id in 1..=grant_count {
            if let Some(mut grant) = env
                .storage()
                .persistent()
                .get::<_, EmergencyGrant>(&DataKey::GrantDetails(grant_id))
            {
                if grant.is_active && grant.expires_at <= now {
                    grant.is_active = false;
                    env.storage()
                        .persistent()
                        .set(&DataKey::GrantDetails(grant_id), &grant);
                    Self::update_active_grants(&env, &grant.patient, &grant.grantee);
                    cleaned += 1;
                }
            }
        }

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("cleanup")),
            (cleaned, env.ledger().timestamp()),
        );

        Ok(cleaned)
    }

    // ==================== Helper Functions ====================

    fn get_config(env: &Env) -> Result<EmergencyConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::NotInitialized)
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Config) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    fn require_emergency_authority(env: &Env, address: &Address) -> Result<(), Error> {
        let authorities: Map<Address, EmergencyAuthority> = env
            .storage()
            .persistent()
            .get(&DataKey::EmergencyAuthorities)
            .unwrap_or(Map::new(env));

        let authority = authorities
            .get(address.clone())
            .ok_or(Error::NotEmergencyAuthority)?;
        if !authority.is_active {
            return Err(Error::AuthorityNotActive);
        }
        Ok(())
    }

    fn validate_emergency_request(
        env: &Env,
        duration: u64,
        emergency_type: &String,
    ) -> Result<(), Error> {
        let config = Self::get_config(env)?;
        if duration > config.max_request_duration {
            return Err(Error::InvalidDuration);
        }
        // Validate emergency type (could check against allowed types)
        if emergency_type.len() == 0 {
            return Err(Error::InvalidEmergencyType);
        }
        Ok(())
    }

    fn validate_mfa_factors(factors: &Vec<MFAFactor>) -> bool {
        // Require at least 2 factors
        factors.len() >= 2
    }

    fn is_emergency_cooldown_active(env: &Env, requester: &Address, patient: &Address) -> bool {
        let config = Self::get_config(env).unwrap();
        let pending: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PendingRequests(patient.clone()))
            .unwrap_or(Vec::new(env));

        let now = env.ledger().timestamp();
        for i in 0..pending.len() {
            let request_id = pending.get(i).unwrap();
            if let Some(request) = env
                .storage()
                .persistent()
                .get::<_, EmergencyRequest>(&DataKey::RequestDetails(request_id))
            {
                if request.requester == *requester
                    && (now - request.created_at) < config.emergency_cooldown
                {
                    return true;
                }
            }
        }
        false
    }

    fn increment_counter(env: &Env, key: &DataKey) -> u64 {
        let mut count: u64 = env.storage().instance().get(key).unwrap_or(0);
        count += 1;
        env.storage().instance().set(key, &count);
        count
    }

    fn create_emergency_grant(env: &Env, request: &EmergencyRequest) -> Result<(), Error> {
        let grant_id = Self::increment_counter(env, &DataKey::GrantCounter);

        let grant = EmergencyGrant {
            grant_id,
            request_id: request.request_id,
            grantee: request.requester.clone(),
            patient: request.patient.clone(),
            record_scope: request.record_scope.clone(),
            granted_at: env.ledger().timestamp(),
            expires_at: env
                .ledger()
                .timestamp()
                .saturating_add(request.requested_duration),
            is_active: true,
            access_count: 0,
            last_access: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::GrantDetails(grant_id), &grant);

        // Add to active grants
        let mut active_grants: Map<Address, Vec<EmergencyGrant>> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveGrants(request.patient.clone()))
            .unwrap_or(Map::new(env));

        let mut patient_grants = active_grants
            .get(request.requester.clone())
            .unwrap_or(Vec::new(env));
        patient_grants.push_back(grant);
        active_grants.set(request.requester.clone(), patient_grants);
        env.storage().persistent().set(
            &DataKey::ActiveGrants(request.patient.clone()),
            &active_grants,
        );

        env.events().publish(
            (symbol_short!("emerg"), symbol_short!("grant_cr")),
            (grant_id, request.request_id, env.ledger().timestamp()),
        );

        Ok(())
    }

    fn update_active_grants(env: &Env, patient: &Address, grantee: &Address) {
        let active_grants: Map<Address, Vec<EmergencyGrant>> = env
            .storage()
            .persistent()
            .get(&DataKey::ActiveGrants(patient.clone()))
            .unwrap_or(Map::new(env));

        if let Some(patient_grants) = active_grants.get(grantee.clone()) {
            // Remove inactive grants
            let mut filtered = Vec::new(env);
            for i in 0..patient_grants.len() {
                let grant = patient_grants.get(i).unwrap();
                if grant.is_active {
                    filtered.push_back(grant);
                }
            }
            let mut updated_active = active_grants.clone();
            updated_active.set(grantee.clone(), filtered);
            env.storage()
                .persistent()
                .set(&DataKey::ActiveGrants(patient.clone()), &updated_active);
        }
    }
}
