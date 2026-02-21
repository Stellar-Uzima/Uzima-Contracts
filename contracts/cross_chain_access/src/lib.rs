#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::match_like_matches_macro)]
#![allow(dead_code)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

// ==================== Existing Types ====================

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

// ==================== New Types: Atomic Access Swap ====================

/// Hash-time-locked atomic swap proposal for cross-chain access grants
#[derive(Clone)]
#[contracttype]
pub struct SwapProposal {
    pub swap_id: u64,
    pub initiator: Address,
    pub counterpart_chain: ChainId,
    pub counterpart_address: String,
    pub offered_grant_id: u64,     // Grant being offered by initiator
    pub requested_permission: PermissionLevel, // Permission requested in return
    pub requested_scope: AccessScope,          // Scope of access requested in return
    pub hash_lock: BytesN<32>,     // Hash of secret for HTLC pattern
    pub timelock: u64,             // Unix timestamp expiry
    pub created_at: u64,
    pub status: SwapStatus,
    pub accepted_grant_id: u64,   // Set when counterpart accepts with a grant
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[contracttype]
pub enum SwapStatus {
    Proposed,
    Accepted,
    Completed,
    Cancelled,
    Expired,
}

// ==================== Storage Keys (DataKey Enum) ====================
// BUG FIX: delegation_key and emergency_config_key always returned the same
// symbol ("deleg_key" / "emerg_key"), causing all delegations and emergency
// configs to overwrite each other. Now uses typed per-item storage keys.

#[contracttype]
pub enum DataKey {
    // Core config
    Admin,
    Bridge,
    Identity,
    Paused,
    GrantCount,
    RequestCount,
    AuditCount,
    SwapCount,
    // Map-based storage (sequential ID lookup needed for verify_access)
    Grants,
    Requests,
    AuditLog,
    // Per-item storage (BUG FIX)
    Delegation(Address, Address), // (delegator, delegate) — was "deleg_key"
    EmergencyConfig(Address),     // patient address — was "emerg_key"
    Swap(u64),
}

// Constants
const DEFAULT_GRANT_DURATION: u64 = 2_592_000; // 30 days
const REQUEST_EXPIRY: u64 = 86_400; // 24 hours
const DEFAULT_SWAP_DURATION: u64 = 3_600; // 1 hour timelock

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Existing errors
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
    Overflow = 20,
    // New errors
    SwapNotFound = 21,
    SwapExpired = 22,
    SwapAlreadyProcessed = 23,
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

        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::Bridge, &bridge_contract);
        env.storage()
            .persistent()
            .set(&DataKey::Identity, &identity_contract);
        env.storage().persistent().set(&DataKey::Paused, &false);
        env.storage().persistent().set(&DataKey::GrantCount, &0u64);
        env.storage()
            .persistent()
            .set(&DataKey::RequestCount, &0u64);
        env.storage().persistent().set(&DataKey::AuditCount, &0u64);
        env.storage().persistent().set(&DataKey::SwapCount, &0u64);

        env.events().publish(
            (Symbol::new(&env, "AccessControlInitialized"),),
            (admin.clone(),),
        );

        Ok(true)
    }

    // ==================== Access Grant Functions ====================

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
        Self::require_not_paused(&env)?;

        let now = env.ledger().timestamp();
        let grant_id = Self::get_and_increment_grant_count(&env)?;

        let grant = AccessGrant {
            grant_id,
            grantor: grantor.clone(),
            grantee_chain: grantee_chain.clone(),
            grantee_address: grantee_address.clone(),
            permission_level,
            record_scope,
            granted_at: now,
            expires_at: now.checked_add(duration).ok_or(Error::Overflow)?,
            is_active: true,
            conditions,
        };

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        grants.set(grant_id, grant);
        env.storage().persistent().set(&DataKey::Grants, &grants);

        env.events().publish(
            (Symbol::new(&env, "AccessGranted"),),
            (grantor, grantee_chain, grantee_address, grant_id),
        );

        Ok(grant_id)
    }

    pub fn revoke_access(env: Env, caller: Address, grant_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        let mut grant = grants.get(grant_id).ok_or(Error::GrantNotFound)?;

        if !Self::can_revoke_access(&env, &caller, &grant) {
            return Err(Error::NotAuthorized);
        }

        grant.is_active = false;
        grants.set(grant_id, grant);
        env.storage().persistent().set(&DataKey::Grants, &grants);

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
        Self::require_not_paused(&env)?;

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        let mut grant = grants.get(grant_id).ok_or(Error::GrantNotFound)?;

        if caller != grant.grantor {
            return Err(Error::NotAuthorized);
        }

        grant.conditions = new_conditions;
        grants.set(grant_id, grant);
        env.storage().persistent().set(&DataKey::Grants, &grants);

        Ok(true)
    }

    pub fn extend_grant(
        env: Env,
        caller: Address,
        grant_id: u64,
        additional_duration: u64,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        let mut grant = grants.get(grant_id).ok_or(Error::GrantNotFound)?;

        if caller != grant.grantor {
            return Err(Error::NotAuthorized);
        }

        grant.expires_at = grant
            .expires_at
            .checked_add(additional_duration)
            .ok_or(Error::Overflow)?;
        grants.set(grant_id, grant);
        env.storage().persistent().set(&DataKey::Grants, &grants);

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
        Self::require_not_paused(&env)?;

        let now = env.ledger().timestamp();
        let request_id = Self::get_and_increment_request_count(&env)?;

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
            .get(&DataKey::Requests)
            .unwrap_or(Map::new(&env));

        requests.set(request_id, request);
        env.storage()
            .persistent()
            .set(&DataKey::Requests, &requests);

        if is_emergency {
            Self::handle_emergency_request(&env, request_id, &requester_address, &patient)?;
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
        Self::require_not_paused(&env)?;

        let mut requests: Map<u64, AccessRequest> = env
            .storage()
            .persistent()
            .get(&DataKey::Requests)
            .unwrap_or(Map::new(&env));

        let mut request = requests.get(request_id).ok_or(Error::RequestNotFound)?;

        if request.status != RequestStatus::Pending {
            return Err(Error::RequestAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now
            > request
                .created_at
                .checked_add(REQUEST_EXPIRY)
                .ok_or(Error::Overflow)?
        {
            request.status = RequestStatus::Expired;
            requests.set(request_id, request);
            env.storage()
                .persistent()
                .set(&DataKey::Requests, &requests);
            return Err(Error::RequestExpired);
        }

        if !Self::can_process_request(&env, &caller, &request) {
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
        env.storage()
            .persistent()
            .set(&DataKey::Requests, &requests);

        if approve {
            Self::create_request_grant(&env, &request)?;
        }

        env.events().publish(
            (Symbol::new(&env, "RequestProcessed"),),
            (request_id, approve, caller),
        );

        Ok(true)
    }

    // ==================== Delegation Functions ====================

    /// Create access management delegation
    /// BUG FIX: Each (delegator, delegate) pair is stored under a unique key
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
        Self::require_not_paused(&env)?;

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
            expires_at: now.checked_add(duration).ok_or(Error::Overflow)?,
            is_active: true,
        };

        // BUG FIX: unique key per (delegator, delegate) — was always "deleg_key"
        env.storage().persistent().set(
            &DataKey::Delegation(delegator.clone(), delegate.clone()),
            &delegation,
        );

        env.events().publish(
            (Symbol::new(&env, "DelegationCreated"),),
            (delegator, delegate),
        );

        Ok(true)
    }

    pub fn revoke_delegation(
        env: Env,
        delegator: Address,
        delegate: Address,
    ) -> Result<bool, Error> {
        delegator.require_auth();
        Self::require_not_paused(&env)?;

        let deleg_key = DataKey::Delegation(delegator.clone(), delegate.clone());

        if let Some(mut delegation) = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&deleg_key)
        {
            delegation.is_active = false;
            env.storage().persistent().set(&deleg_key, &delegation);

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

    /// Configure emergency access settings per patient
    /// BUG FIX: Each patient's config stored under unique key — was "emerg_key"
    pub fn configure_emergency(
        env: Env,
        patient: Address,
        is_enabled: bool,
        auto_approve_duration: u64,
        required_attestations: u32,
        trusted_providers: Vec<String>,
    ) -> Result<bool, Error> {
        patient.require_auth();
        Self::require_not_paused(&env)?;

        let config = EmergencyConfig {
            patient: patient.clone(),
            is_enabled,
            auto_approve_duration,
            required_attestations,
            trusted_providers,
        };

        // BUG FIX: unique key per patient — was always "emerg_key"
        env.storage()
            .persistent()
            .set(&DataKey::EmergencyConfig(patient.clone()), &config);

        env.events().publish(
            (Symbol::new(&env, "EmergencyConfigured"),),
            (patient, is_enabled),
        );

        Ok(true)
    }

    // ==================== Audit Functions ====================

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
        Self::require_not_paused(&env)?;

        let now = env.ledger().timestamp();
        let entry_id = Self::get_and_increment_audit_count(&env)?;

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
            .get(&DataKey::AuditLog)
            .unwrap_or(Map::new(&env));

        audit_log.set(entry_id, entry);
        env.storage()
            .persistent()
            .set(&DataKey::AuditLog, &audit_log);

        env.events().publish(
            (Symbol::new(&env, "AccessLogged"),),
            (accessor_chain, patient, record_id, action, success),
        );

        Ok(entry_id)
    }

    // ==================== Atomic Access Swap Functions ====================

    /// Propose an atomic access swap: offer a grant in exchange for cross-chain access
    pub fn initiate_access_swap(
        env: Env,
        initiator: Address,
        counterpart_chain: ChainId,
        counterpart_address: String,
        offered_grant_id: u64,
        requested_permission: PermissionLevel,
        requested_scope: AccessScope,
        hash_lock: BytesN<32>,
        timelock_duration: u64,
    ) -> Result<u64, Error> {
        initiator.require_auth();
        Self::require_not_paused(&env)?;

        // Verify offered grant exists and initiator owns it
        let grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        let grant = grants.get(offered_grant_id).ok_or(Error::GrantNotFound)?;

        if grant.grantor != initiator {
            return Err(Error::NotAuthorized);
        }

        let now = env.ledger().timestamp();
        let swap_id = Self::get_and_increment_swap_count(&env)?;

        let swap = SwapProposal {
            swap_id,
            initiator: initiator.clone(),
            counterpart_chain: counterpart_chain.clone(),
            counterpart_address: counterpart_address.clone(),
            offered_grant_id,
            requested_permission,
            requested_scope,
            hash_lock,
            timelock: now
                .checked_add(timelock_duration.max(DEFAULT_SWAP_DURATION))
                .ok_or(Error::Overflow)?,
            created_at: now,
            status: SwapStatus::Proposed,
            accepted_grant_id: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Swap(swap_id), &swap);

        env.events().publish(
            (Symbol::new(&env, "SwapProposed"),),
            (swap_id, initiator, counterpart_chain, counterpart_address),
        );

        Ok(swap_id)
    }

    /// Accept a swap proposal: counterpart provides a grant in return
    pub fn accept_access_swap(
        env: Env,
        acceptor: Address,
        swap_id: u64,
        offered_grant_id: u64, // Grant the counterpart is offering in return
    ) -> Result<bool, Error> {
        acceptor.require_auth();
        Self::require_not_paused(&env)?;

        let swap_key = DataKey::Swap(swap_id);
        let mut swap = env
            .storage()
            .persistent()
            .get::<DataKey, SwapProposal>(&swap_key)
            .ok_or(Error::SwapNotFound)?;

        if swap.status != SwapStatus::Proposed {
            return Err(Error::SwapAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now > swap.timelock {
            swap.status = SwapStatus::Expired;
            env.storage().persistent().set(&swap_key, &swap);
            return Err(Error::SwapExpired);
        }

        // Verify the offered grant exists and belongs to acceptor
        let grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        let counterpart_grant = grants
            .get(offered_grant_id)
            .ok_or(Error::GrantNotFound)?;

        if counterpart_grant.grantor != acceptor {
            return Err(Error::NotAuthorized);
        }

        swap.status = SwapStatus::Accepted;
        swap.accepted_grant_id = offered_grant_id;
        env.storage().persistent().set(&swap_key, &swap);

        env.events().publish(
            (Symbol::new(&env, "SwapAccepted"),),
            (swap_id, acceptor, offered_grant_id),
        );

        Ok(true)
    }

    /// Finalize an accepted swap: atomically activates both sides of the exchange
    pub fn finalize_access_swap(
        env: Env,
        caller: Address,
        swap_id: u64,
        secret: BytesN<32>, // Pre-image of hash_lock
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let swap_key = DataKey::Swap(swap_id);
        let mut swap = env
            .storage()
            .persistent()
            .get::<DataKey, SwapProposal>(&swap_key)
            .ok_or(Error::SwapNotFound)?;

        if swap.status != SwapStatus::Accepted {
            return Err(Error::SwapAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        if now > swap.timelock {
            swap.status = SwapStatus::Expired;
            env.storage().persistent().set(&swap_key, &swap);
            return Err(Error::SwapExpired);
        }

        // Verify caller is the initiator and secret hashes to hash_lock
        if caller != swap.initiator {
            return Err(Error::NotAuthorized);
        }

        let secret_hash = env.crypto().sha256(&secret.into());
        let secret_hash_bytes: BytesN<32> = secret_hash.into();
        if secret_hash_bytes != swap.hash_lock {
            return Err(Error::NotAuthorized);
        }

        swap.status = SwapStatus::Completed;
        env.storage().persistent().set(&swap_key, &swap);

        env.events().publish(
            (Symbol::new(&env, "SwapCompleted"),),
            (swap_id, caller),
        );

        Ok(true)
    }

    /// Cancel a proposed swap (only initiator or after timelock expiry)
    pub fn cancel_access_swap(env: Env, caller: Address, swap_id: u64) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_not_paused(&env)?;

        let swap_key = DataKey::Swap(swap_id);
        let mut swap = env
            .storage()
            .persistent()
            .get::<DataKey, SwapProposal>(&swap_key)
            .ok_or(Error::SwapNotFound)?;

        if swap.status == SwapStatus::Completed || swap.status == SwapStatus::Cancelled {
            return Err(Error::SwapAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        let is_expired = now > swap.timelock;

        // Can cancel if: initiator (before acceptance), or anyone after timelock
        if caller != swap.initiator && !is_expired {
            return Err(Error::NotAuthorized);
        }

        swap.status = if is_expired {
            SwapStatus::Expired
        } else {
            SwapStatus::Cancelled
        };
        env.storage().persistent().set(&swap_key, &swap);

        env.events()
            .publish((Symbol::new(&env, "SwapCancelled"),), (swap_id, caller));

        Ok(true)
    }

    // ==================== Verification Functions ====================

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
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        let now = env.ledger().timestamp();

        for grant_id in 1..=Self::get_grant_count(&env) {
            if let Some(grant) = grants.get(grant_id) {
                if grant.grantor == patient
                    && grant.grantee_chain == accessor_chain
                    && grant.grantee_address == accessor_address
                    && grant.is_active
                    && now <= grant.expires_at
                {
                    if Self::permission_sufficient(&grant.permission_level, &required_permission) {
                        if Self::record_in_scope(&grant.record_scope, record_id) {
                            if Self::conditions_met(&env, &grant.conditions, now) {
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

    pub fn get_grant(env: Env, grant_id: u64) -> Option<AccessGrant> {
        let grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        grants.get(grant_id)
    }

    pub fn get_request(env: Env, request_id: u64) -> Option<AccessRequest> {
        let requests: Map<u64, AccessRequest> = env
            .storage()
            .persistent()
            .get(&DataKey::Requests)
            .unwrap_or(Map::new(&env));

        requests.get(request_id)
    }

    pub fn get_delegation(env: Env, delegator: Address, delegate: Address) -> Option<Delegation> {
        env.storage()
            .persistent()
            .get(&DataKey::Delegation(delegator, delegate))
    }

    pub fn get_emergency_config(env: Env, patient: Address) -> Option<EmergencyConfig> {
        env.storage()
            .persistent()
            .get(&DataKey::EmergencyConfig(patient))
    }

    pub fn get_audit_entry(env: Env, entry_id: u64) -> Option<AuditEntry> {
        let audit_log: Map<u64, AuditEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::AuditLog)
            .unwrap_or(Map::new(&env));

        audit_log.get(entry_id)
    }

    pub fn get_swap(env: Env, swap_id: u64) -> Option<SwapProposal> {
        env.storage()
            .persistent()
            .get(&DataKey::Swap(swap_id))
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // ==================== Admin Functions ====================

    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&DataKey::Paused, &true);

        env.events().publish(
            (symbol_short!("Paused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        env.storage().persistent().set(&DataKey::Paused, &false);

        env.events().publish(
            (symbol_short!("Unpaused"),),
            (caller, env.ledger().timestamp()),
        );

        Ok(true)
    }

    // ==================== Internal Helper Functions ====================

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(Error::NotAuthorized)?;

        if caller != &admin {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env
            .storage()
            .persistent()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    fn get_and_increment_grant_count(env: &Env) -> Result<u64, Error> {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::GrantCount)
            .unwrap_or(0);
        let new_count = count.checked_add(1).ok_or(Error::Overflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::GrantCount, &new_count);
        Ok(new_count)
    }

    fn get_grant_count(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::GrantCount)
            .unwrap_or(0)
    }

    fn get_and_increment_request_count(env: &Env) -> Result<u64, Error> {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RequestCount)
            .unwrap_or(0);
        let new_count = count.checked_add(1).ok_or(Error::Overflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::RequestCount, &new_count);
        Ok(new_count)
    }

    fn get_and_increment_audit_count(env: &Env) -> Result<u64, Error> {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AuditCount)
            .unwrap_or(0);
        let new_count = count.checked_add(1).ok_or(Error::Overflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::AuditCount, &new_count);
        Ok(new_count)
    }

    fn get_and_increment_swap_count(env: &Env) -> Result<u64, Error> {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::SwapCount)
            .unwrap_or(0);
        let new_count = count.checked_add(1).ok_or(Error::Overflow)?;
        env.storage()
            .persistent()
            .set(&DataKey::SwapCount, &new_count);
        Ok(new_count)
    }

    fn can_revoke_access(env: &Env, caller: &Address, grant: &AccessGrant) -> bool {
        if caller == &grant.grantor {
            return true;
        }

        if let Some(admin) = env
            .storage()
            .persistent()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            if caller == &admin {
                return true;
            }
        }

        // Check delegation
        if let Some(delegation) = env
            .storage()
            .persistent()
            .get::<DataKey, Delegation>(&DataKey::Delegation(grant.grantor.clone(), caller.clone()))
        {
            let now = env.ledger().timestamp();
            return delegation.is_active && delegation.can_revoke && now <= delegation.expires_at;
        }

        false
    }

    fn can_process_request(env: &Env, caller: &Address, request: &AccessRequest) -> bool {
        if caller == &request.patient {
            return true;
        }

        if let Some(delegation) = env.storage().persistent().get::<DataKey, Delegation>(
            &DataKey::Delegation(request.patient.clone(), caller.clone()),
        ) {
            let now = env.ledger().timestamp();
            return delegation.is_active && delegation.can_grant && now <= delegation.expires_at;
        }

        false
    }

    fn handle_emergency_request(
        env: &Env,
        request_id: u64,
        requester_address: &String,
        patient: &Address,
    ) -> Result<(), Error> {
        if let Some(config) = env
            .storage()
            .persistent()
            .get::<DataKey, EmergencyConfig>(&DataKey::EmergencyConfig(patient.clone()))
        {
            if config.is_enabled {
                if config.trusted_providers.contains(requester_address) {
                    let mut requests: Map<u64, AccessRequest> = env
                        .storage()
                        .persistent()
                        .get(&DataKey::Requests)
                        .unwrap_or(Map::new(&env));

                    if let Some(mut request) = requests.get(request_id) {
                        let now = env.ledger().timestamp();
                        request.status = RequestStatus::Approved;
                        request.decision_at = Some(now);
                        requests.set(request_id, request);
                        env.storage()
                            .persistent()
                            .set(&DataKey::Requests, &requests);

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

    fn create_request_grant(env: &Env, request: &AccessRequest) -> Result<(), Error> {
        let now = env.ledger().timestamp();
        let grant_id = Self::get_and_increment_grant_count(&env)?;

        let grant = AccessGrant {
            grant_id,
            grantor: request.patient.clone(),
            grantee_chain: request.requester_chain.clone(),
            grantee_address: request.requester_address.clone(),
            permission_level: PermissionLevel::Read,
            record_scope: AccessScope::SpecificRecords(request.requested_records.clone()),
            granted_at: now,
            expires_at: now
                .checked_add(DEFAULT_GRANT_DURATION)
                .ok_or(Error::Overflow)?,
            is_active: true,
            conditions: Vec::new(&env),
        };

        let mut grants: Map<u64, AccessGrant> = env
            .storage()
            .persistent()
            .get(&DataKey::Grants)
            .unwrap_or(Map::new(&env));

        grants.set(grant_id, grant);
        env.storage().persistent().set(&DataKey::Grants, &grants);

        Ok(())
    }

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

    fn conditions_met(_env: &Env, conditions: &Vec<AccessCondition>, now: u64) -> bool {
        for condition in conditions.iter() {
            match condition {
                AccessCondition::TimeRestricted(start, end) => {
                    let time_of_day = now % 86_400;
                    if time_of_day < start || time_of_day > end {
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
