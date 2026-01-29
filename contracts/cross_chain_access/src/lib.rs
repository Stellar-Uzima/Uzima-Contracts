#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

// ==================== Data Structures ====================

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum PermissionLevel {
    None,
    Read,             
    ReadConfidential, 
    Write,            
    Admin,            
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ChainId {
    None, 
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
pub struct AccessGrant {
    pub grant_id: u64,
    pub grantor: Address,        
    pub grantee_chain: ChainId,  
    pub grantee_address: String, 
    pub permission_level: PermissionLevel,
    pub record_scope: AccessScope,
    pub granted_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
    pub conditions: Vec<AccessCondition>,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AccessScope {
    AllRecords,                    
    SpecificRecords(Vec<u64>), 
    CategoryBased(String),     
    TimeRanged(u64, u64),      
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AccessCondition {
    EmergencyOnly,                
    RequireConsent,           
    AuditRequired,            
    SingleUse,                
    TimeRestricted(u64, u64), 
}

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

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum RequestStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Revoked,
}

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
    pub ip_hash: BytesN<32>, 
    pub success: bool,
}

#[derive(Clone, PartialEq, Eq)]
#[contracttype]
pub enum AccessAction {
    View,
    Download,
    Share,
    Export,
    EmergencyAccess,
}

#[derive(Clone)]
#[contracttype]
pub struct Delegation {
    pub delegator: Address,       
    pub delegate: Address,        
    pub delegate_chain: ChainId,  
    pub delegate_address: String, 
    pub can_grant: bool,
    pub can_revoke: bool,
    pub can_manage_emergency: bool,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct EmergencyConfig {
    pub patient: Address,
    pub is_enabled: bool,
    pub auto_approve_duration: u64, 
    pub required_attestations: u32, 
    pub trusted_providers: Vec<String>, 
}

// ==================== Helper Structs for Arguments ====================

#[derive(Clone)]
#[contracttype]
pub struct GrantAccessArgs {
    pub grantor: Address,
    pub grantee_chain: ChainId,
    pub grantee_address: String,
    pub permission_level: PermissionLevel,
    pub record_scope: AccessScope,
    pub duration: u64,
    pub conditions: Vec<AccessCondition>,
}

#[derive(Clone)]
#[contracttype]
pub struct DelegationArgs {
    pub delegator: Address,
    pub delegate: Address,
    pub delegate_chain: ChainId,
    pub delegate_address: String,
    pub can_grant: bool,
    pub can_revoke: bool,
    pub can_manage_emergency: bool,
    pub duration: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct LogAccessArgs {
    pub accessor_chain: ChainId,
    pub accessor_address: String,
    pub patient: Address,
    pub record_id: u64,
    pub action: AccessAction,
    pub ip_hash: BytesN<32>,
    pub success: bool,
}

// ==================== Storage Keys & Constants ====================

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

const DEFAULT_GRANT_DURATION: u64 = 2_592_000; 
const REQUEST_EXPIRY: u64 = 86_400; 

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
impl CrossChainAccessContract {
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
        env.storage().persistent().set(&IDENTITY, &identity_contract);
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

    pub fn grant_access(env: Env, args: GrantAccessArgs) -> Result<u64, Error> {
        args.grantor.require_auth();
        Self::require_not_paused(env.clone())?;

        let now = env.ledger().timestamp();
        let grant_id = Self::get_and_increment_grant_count(env.clone());

        let grant = AccessGrant {
            grant_id,
            grantor: args.grantor.clone(),
            grantee_chain: args.grantee_chain.clone(),
            grantee_address: args.grantee_address.clone(),
            permission_level: args.permission_level,
            record_scope: args.record_scope,
            granted_at: now,
            expires_at: now + args.duration,
            is_active: true,
            conditions: args.conditions,
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
            (args.grantor, args.grantee_chain, args.grantee_address, grant_id),
        );

        Ok(grant_id)
    }

    pub fn revoke_access(env: Env, caller: Address, grant_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(env.clone())?;

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        let mut grant = grants.get(grant_id).ok_or(Error::GrantNotFound)?;

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

        if caller != grant.grantor {
            return Err(Error::NotAuthorized);
        }

        grant.conditions = new_conditions;
        grants.set(grant_id, grant);
        env.storage().persistent().set(&GRANTS, &grants);

        Ok(true)
    }

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

        if request.status != RequestStatus::Pending {
            return Err(Error::RequestAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now > request.created_at + REQUEST_EXPIRY {
            request.status = RequestStatus::Expired;
            requests.set(request_id, request);
            env.storage().persistent().set(&REQUESTS, &requests);
            return Err(Error::RequestExpired);
        }

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

    pub fn create_delegation(env: Env, args: DelegationArgs) -> Result<bool, Error> {
        args.delegator.require_auth();
        Self::require_not_paused(env.clone())?;

        let now = env.ledger().timestamp();

        let delegation = Delegation {
            delegator: args.delegator.clone(),
            delegate: args.delegate.clone(),
            delegate_chain: args.delegate_chain,
            delegate_address: args.delegate_address,
            can_grant: args.can_grant,
            can_revoke: args.can_revoke,
            can_manage_emergency: args.can_manage_emergency,
            created_at: now,
            expires_at: now + args.duration,
            is_active: true,
        };

        let deleg_key = Self::delegation_key(env.clone(), &args.delegator, &args.delegate);
        let mut delegations: Map<Symbol, Delegation> = env
            .storage()
            .persistent()
            .get(&DELEGATIONS)
            .unwrap_or(Map::new(&env));

        delegations.set(deleg_key, delegation);
        env.storage().persistent().set(&DELEGATIONS, &delegations);

        env.events().publish(
            (Symbol::new(&env, "DelegationCreated"),),
            (args.delegator, args.delegate),
        );

        Ok(true)
    }

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

    pub fn log_access(env: Env, args: LogAccessArgs) -> Result<u64, Error> {
        Self::require_not_paused(env.clone())?;

        let now = env.ledger().timestamp();
        let entry_id = Self::get_and_increment_audit_count(env.clone());

        let entry = AuditEntry {
            entry_id,
            accessor_chain: args.accessor_chain.clone(),
            accessor_address: args.accessor_address.clone(),
            patient: args.patient.clone(),
            record_id: args.record_id,
            action: args.action.clone(),
            timestamp: now,
            ip_hash: args.ip_hash,
            success: args.success,
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
            (args.accessor_chain, args.patient, args.record_id, args.action, args.success),
        );

        Ok(entry_id)
    }

    // ==================== Verification & Query Functions ====================

    pub fn verify_access(
        env: Env,
        accessor_chain: ChainId,
        accessor_address: String,
        patient: Address,
        record_id: u64,
        required_permission: PermissionLevel,
    ) -> bool {
        let grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));

        let now = env.ledger().timestamp();

        for grant_id in 1..=Self::get_grant_count(env.clone()) {
            if let Some(grant) = grants.get(grant_id) {
                if grant.grantor == patient
                    && grant.grantee_chain == accessor_chain
                    && grant.grantee_address == accessor_address
                    && grant.is_active
                    && now <= grant.expires_at
                {
                    if Self::permission_sufficient(&grant.permission_level, &required_permission) {
                        if Self::record_in_scope(&grant.record_scope, record_id) {
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

    pub fn get_grant(env: Env, grant_id: u64) -> Option<AccessGrant> {
        let grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&GRANTS)
            .unwrap_or(Map::new(&env));
        grants.get(grant_id)
    }

    pub fn get_request(env: Env, request_id: u64) -> Option<AccessRequest> {
        let requests: Map<u64, AccessRequest> = env
            .storage()
            .persistent()
            .get(&REQUESTS)
            .unwrap_or(Map::new(&env));
        requests.get(request_id)
    }

    pub fn get_delegation(env: Env, delegator: Address, delegate: Address) -> Option<Delegation> {
        let deleg_key = Self::delegation_key(env.clone(), &delegator, &delegate);
        let delegations: Map<Symbol, Delegation> = env
            .storage()
            .persistent()
            .get(&DELEGATIONS)
            .unwrap_or(Map::new(&env));
        delegations.get(deleg_key)
    }

    pub fn get_emergency_config(env: Env, patient: Address) -> Option<EmergencyConfig> {
        let config_key = Self::emergency_config_key(env.clone(), &patient);
        let configs: Map<Symbol, EmergencyConfig> = env
            .storage()
            .persistent()
            .get(&EMERGENCY_CONFIG)
            .unwrap_or(Map::new(&env));
        configs.get(config_key)
    }

    pub fn get_audit_entry(env: Env, entry_id: u64) -> Option<AuditEntry> {
        let audit_log: Map<u64, AuditEntry> = env
            .storage()
            .persistent()
            .get(&AUDIT_LOG)
            .unwrap_or(Map::new(&env));
        audit_log.get(entry_id)
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().persistent().get(&PAUSED).unwrap_or(false)
    }

    // ==================== Admin Functions ====================

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

    // ==================== Internal Helpers ====================

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
        if caller == &grant.grantor {
            return true;
        }
        if let Some(admin) = env.storage().persistent().get::<Symbol, Address>(&ADMIN) {
            if caller == &admin {
                return true;
            }
        }
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
        if caller == &request.patient {
            return true;
        }
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
                if config.trusted_providers.contains(requester_address) {
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
            AccessScope::CategoryBased(_) => true, 
            AccessScope::TimeRanged(_, _) => true, 
        }
    }

    fn conditions_met(_env: Env, conditions: &Vec<AccessCondition>, now: u64) -> bool {
        for condition in conditions.iter() {
            match condition {
                AccessCondition::TimeRestricted(start, end) => {
                    let time_of_day = now % 86_400;
                    if time_of_day < *start || time_of_day > *end {
                        return false;
                    }
                }
                AccessCondition::SingleUse => {
                    return true;
                }
                _ => {}
            }
        }
        true
    }
}