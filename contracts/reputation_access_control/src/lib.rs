#![no_std]
#![allow(clippy::arithmetic_side_effects)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env,
    String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InsufficientReputation = 4,
    AccessDenied = 5,
    InvalidResource = 6,
    PolicyNotFound = 7,
    ProviderNotVerified = 8,
    CredentialExpired = 9,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResourceType {
    PatientRecords = 0,
    MedicalPrescriptions = 1,
    DiagnosticReports = 2,
    SurgicalProcedures = 3,
    EmergencyAccess = 4,
    ResearchData = 5,
    AdministrativeFunctions = 6,
    ProviderDirectory = 7,
    CredentialManagement = 8,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AccessLevel {
    None = 0,
    Read = 1,
    Write = 2,
    Update = 3,
    Delete = 4,
    Admin = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeRestrictionPolicy {
    None,
    Restricted(TimeRestriction),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessPolicy {
    pub resource_type: ResourceType,
    pub min_reputation_score: u32,
    pub required_credentials: Vec<Symbol>, // Credential types required
    pub access_level: AccessLevel,
    pub time_restriction: TimeRestrictionPolicy,
    pub special_conditions: Vec<Symbol>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeRestriction {
    pub start_hour: u32,
    pub end_hour: u32,
    pub allowed_days: Vec<u32>, // 1-7 for Mon-Sun
    pub timezone: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessRequest {
    pub request_id: BytesN<32>,
    pub provider: Address,
    pub resource_type: ResourceType,
    pub requested_access: AccessLevel,
    pub timestamp: u64,
    pub justification: String,
    pub status: RequestStatus,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestStatus {
    Pending = 0,
    Approved = 1,
    Denied = 2,
    Expired = 3,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    AccessPolicy(ResourceType),
    AccessRequest(BytesN<32>),
    ProviderRequests(Address),                  // Vec<BytesN<32>>
    ProviderAccessLevel(Address, ResourceType), // Current access level
    ReputationThreshold(ResourceType),
    EmergencyAccess(Address), // bool for emergency access granted
}

#[contract]
pub struct ReputationAccessControl;

#[contractimpl]
impl ReputationAccessControl {
    // Initialize access control system
    pub fn initialize(
        env: Env,
        admin: Address,
        _reputation_contract: Address,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        // Set default reputation thresholds
        Self::set_default_policies(&env)?;

        env.events()
            .publish((symbol_short!("REPUTAC"), symbol_short!("INIT")), admin);
        Ok(())
    }

    // Set access policy for a resource type
    pub fn set_access_policy(
        env: Env,
        admin: Address,
        resource_type: ResourceType,
        policy: AccessPolicy,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        env.storage()
            .persistent()
            .set(&DataKey::AccessPolicy(resource_type), &policy);

        env.events().publish(
            (symbol_short!("REPUTAC"), symbol_short!("POLICY")),
            resource_type,
        );
        Ok(())
    }

    // Check if provider has access to resource
    pub fn check_access(
        env: Env,
        provider: Address,
        resource_type: ResourceType,
        requested_access: AccessLevel,
    ) -> Result<bool, Error> {
        Self::require_initialized(&env)?;

        // Check if provider has emergency access
        if Self::has_emergency_access(&env, &provider)? {
            return Ok(true);
        }

        // Get access policy
        let policy: AccessPolicy = env
            .storage()
            .persistent()
            .get(&DataKey::AccessPolicy(resource_type))
            .ok_or(Error::PolicyNotFound)?;

        // Check reputation score requirement
        if !Self::check_reputation_requirement(&env, provider.clone(), policy.min_reputation_score)?
        {
            return Ok(false);
        }

        // Check credential requirements
        if !Self::check_credential_requirements(&env, provider, &policy.required_credentials)? {
            return Ok(false);
        }

        // Check time restrictions
        match &policy.time_restriction {
            TimeRestrictionPolicy::Restricted(time_restriction) => {
                if !Self::check_time_restriction(&env, time_restriction)? {
                    return Ok(false);
                }
            }
            TimeRestrictionPolicy::None => {}
        }

        // Check if requested access level is allowed
        if requested_access as u32 > policy.access_level as u32 {
            return Ok(false);
        }

        Ok(true)
    }

    // Request access to resource
    pub fn request_access(
        env: Env,
        provider: Address,
        resource_type: ResourceType,
        requested_access: AccessLevel,
        justification: String,
    ) -> Result<BytesN<32>, Error> {
        provider.require_auth();
        Self::require_initialized(&env)?;

        let request_id = Self::generate_request_id(&env, &provider);
        let current_time = env.ledger().timestamp();

        let request = AccessRequest {
            request_id: request_id.clone(),
            provider: provider.clone(),
            resource_type,
            requested_access,
            timestamp: current_time,
            justification,
            status: RequestStatus::Pending,
        };

        // Store request
        env.storage()
            .persistent()
            .set(&DataKey::AccessRequest(request_id.clone()), &request);

        // Update provider's request list
        let mut requests: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderRequests(provider.clone()))
            .unwrap_or(Vec::new(&env));
        requests.push_back(request_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ProviderRequests(provider), &requests);

        env.events().publish(
            (symbol_short!("REPUTAC"), symbol_short!("REQUEST")),
            request_id.clone(),
        );
        Ok(request_id)
    }

    // Approve access request
    pub fn approve_request(env: Env, admin: Address, request_id: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut request: AccessRequest = env
            .storage()
            .persistent()
            .get(&DataKey::AccessRequest(request_id.clone()))
            .ok_or(Error::InvalidResource)?;

        request.status = RequestStatus::Approved;
        env.storage()
            .persistent()
            .set(&DataKey::AccessRequest(request_id.clone()), &request);

        // Set provider's access level for this resource
        env.storage().persistent().set(
            &DataKey::ProviderAccessLevel(request.provider, request.resource_type),
            &request.requested_access,
        );

        env.events().publish(
            (symbol_short!("REPUTAC"), symbol_short!("APPROVED")),
            request_id,
        );
        Ok(())
    }

    // Deny access request
    pub fn deny_request(env: Env, admin: Address, request_id: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut request: AccessRequest = env
            .storage()
            .persistent()
            .get(&DataKey::AccessRequest(request_id.clone()))
            .ok_or(Error::InvalidResource)?;

        request.status = RequestStatus::Denied;
        env.storage()
            .persistent()
            .set(&DataKey::AccessRequest(request_id.clone()), &request);

        env.events().publish(
            (symbol_short!("REPUTAC"), symbol_short!("DENIED")),
            request_id,
        );
        Ok(())
    }

    // Grant emergency access
    pub fn grant_emergency_access(
        env: Env,
        admin: Address,
        provider: Address,
        _duration_hours: u32,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        env.storage()
            .persistent()
            .set(&DataKey::EmergencyAccess(provider.clone()), &true);

        env.events().publish(
            (symbol_short!("REPUTAC"), symbol_short!("EMERGENCY")),
            provider,
        );
        Ok(())
    }

    // Revoke emergency access
    pub fn revoke_emergency_access(
        env: Env,
        admin: Address,
        provider: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        env.storage()
            .persistent()
            .remove(&DataKey::EmergencyAccess(provider.clone()));

        env.events().publish(
            // shortened: "REVOKE_EM" = 9 chars (was "REVOKE_EMERGENCY" = 16)
            (symbol_short!("REPUTAC"), symbol_short!("REVOKE_EM")),
            provider,
        );
        Ok(())
    }

    // Get provider's current access level for resource
    pub fn get_provider_access_level(
        env: Env,
        provider: Address,
        resource_type: ResourceType,
    ) -> Result<AccessLevel, Error> {
        Self::require_initialized(&env)?;

        if let Some(access_level) = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderAccessLevel(provider, resource_type))
        {
            Ok(access_level)
        } else {
            Ok(AccessLevel::None)
        }
    }

    // Get provider's access requests
    pub fn get_provider_requests(env: Env, provider: Address) -> Result<Vec<AccessRequest>, Error> {
        Self::require_initialized(&env)?;

        let request_ids: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderRequests(provider))
            .unwrap_or(Vec::new(&env));

        let mut requests = Vec::new(&env);
        for request_id in request_ids.iter() {
            if let Some(request) = env
                .storage()
                .persistent()
                .get::<DataKey, AccessRequest>(&DataKey::AccessRequest(request_id.clone()))
            {
                requests.push_back(request);
            }
        }

        Ok(requests)
    }

    // Set reputation threshold for resource type
    pub fn set_reputation_threshold(
        env: Env,
        admin: Address,
        resource_type: ResourceType,
        threshold: u32,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        env.storage()
            .persistent()
            .set(&DataKey::ReputationThreshold(resource_type), &threshold);

        env.events().publish(
            (symbol_short!("REPUTAC"), symbol_short!("THRESHOLD")),
            (resource_type, threshold),
        );
        Ok(())
    }

    // Helper functions
    fn set_default_policies(env: &Env) -> Result<(), Error> {
        // Credential symbols – all ≤ 9 chars
        let med_lic = symbol_short!("MedLic");
        let state_lic = symbol_short!("StateLic");
        let dea_reg = symbol_short!("DEAReg");
        let hipaa = symbol_short!("HIPAA");
        let presc_auth = symbol_short!("PrescAuth");
        let emrg_just = symbol_short!("EmrgJust");

        // Patient Records - High security
        let patient_records_policy = AccessPolicy {
            resource_type: ResourceType::PatientRecords,
            min_reputation_score: 80,
            required_credentials: vec![env, med_lic.clone(), state_lic.clone()],
            access_level: AccessLevel::Read,
            time_restriction: TimeRestrictionPolicy::None,
            special_conditions: vec![env, hipaa.clone()],
        };

        // Medical Prescriptions - High security
        let prescriptions_policy = AccessPolicy {
            resource_type: ResourceType::MedicalPrescriptions,
            min_reputation_score: 85,
            required_credentials: vec![env, med_lic.clone(), dea_reg.clone(), state_lic.clone()],
            access_level: AccessLevel::Write,
            time_restriction: TimeRestrictionPolicy::None,
            special_conditions: vec![env, presc_auth.clone()],
        };

        // Emergency Access - Lower threshold for emergencies
        let emergency_policy = AccessPolicy {
            resource_type: ResourceType::EmergencyAccess,
            min_reputation_score: 60,
            required_credentials: vec![env, med_lic],
            access_level: AccessLevel::Admin,
            time_restriction: TimeRestrictionPolicy::None,
            special_conditions: vec![env, emrg_just],
        };

        env.storage().persistent().set(
            &DataKey::AccessPolicy(ResourceType::PatientRecords),
            &patient_records_policy,
        );
        env.storage().persistent().set(
            &DataKey::AccessPolicy(ResourceType::MedicalPrescriptions),
            &prescriptions_policy,
        );
        env.storage().persistent().set(
            &DataKey::AccessPolicy(ResourceType::EmergencyAccess),
            &emergency_policy,
        );

        Ok(())
    }

    fn check_reputation_requirement(
        _env: &Env,
        _provider: Address,
        _min_score: u32,
    ) -> Result<bool, Error> {
        // Cross-contract call placeholder
        Ok(true)
    }

    fn check_credential_requirements(
        _env: &Env,
        _provider: Address,
        _required_credentials: &Vec<Symbol>,
    ) -> Result<bool, Error> {
        // Cross-contract call placeholder
        Ok(true)
    }

    fn check_time_restriction(
        env: &Env,
        time_restriction: &TimeRestriction,
    ) -> Result<bool, Error> {
        let current_timestamp = env.ledger().timestamp();

        // Use checked arithmetic to satisfy clippy::arithmetic_side_effects
        let seconds_per_hour: u64 = 3600;
        let hours_per_day: u64 = 24;
        let seconds_per_day: u64 = seconds_per_hour.checked_mul(hours_per_day).unwrap_or(86400);

        let total_hours = current_timestamp.checked_div(seconds_per_hour).unwrap_or(0);
        let current_hour = total_hours.checked_rem(hours_per_day).unwrap_or(0);

        // Check if current hour is within allowed range
        if current_hour < time_restriction.start_hour as u64
            || current_hour > time_restriction.end_hour as u64
        {
            return Ok(false);
        }

        // Check day of week (simplified)
        let total_days = current_timestamp.checked_div(seconds_per_day).unwrap_or(0);
        let day_index = total_days.checked_rem(7).unwrap_or(0);
        let day_of_week = day_index.checked_add(1).unwrap_or(1) as u32;

        if !time_restriction.allowed_days.contains(day_of_week) {
            return Ok(false);
        }

        Ok(true)
    }

    fn has_emergency_access(env: &Env, provider: &Address) -> Result<bool, Error> {
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::EmergencyAccess(provider.clone()))
            .unwrap_or(false))
    }

    fn generate_request_id(env: &Env, _provider: &Address) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let mut data = [0u8; 32];
        data[0..8].copy_from_slice(&timestamp.to_be_bytes());
        BytesN::from_array(env, &data)
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin == *caller {
            Ok(())
        } else {
            Err(Error::NotAuthorized)
        }
    }
}
