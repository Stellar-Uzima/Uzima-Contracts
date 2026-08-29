#![no_std]
//! emergency_access_override - Healthcare smart contract on Stellar blockchain.
#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};
// Shared multi-sig helpers (Phase 4 migration: see issue #830)
use governance_commons::{multi_sig, ApprovalStatus};

// ==================== Rate Limiting Types (Issue #1173) ====================

/// Configurable rate-limit parameters for emergency access requests.
#[derive(Clone, Debug)]
#[contracttype]
pub struct RateLimitConfig {
    /// Maximum requests a single requester can submit per time window.
    pub max_requests_per_window: u32,
    /// Duration of the rate-limit window in seconds.
    pub window_seconds: u64,
    /// Cooldown period in seconds between successive requests from the same requester.
    pub cooldown_seconds: u64,
    /// Maximum total grants across all requesters before circuit breaker trips.
    pub max_grants_per_window: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_window: 5,
            window_seconds: 3_600,
            cooldown_seconds: 86_400,
            max_grants_per_window: 10,
        }
    }
}

/// Rolling rate-limit window state tracked per requester.
#[derive(Clone, Debug)]
#[contracttype]
pub struct RequesterWindow {
    /// Timestamp when the current window started.
    pub window_start: u64,
    /// Number of requests submitted within the current window.
    pub request_count: u32,
}

/// An emergency access request with full metadata for audit and rate-limiting.
#[derive(Clone, Debug)]
#[contracttype]
pub struct EmergencyAccessRequest {
    /// Unique request identifier.
    pub request_id: u64,
    /// Patient whose data is being requested.
    pub patient: Address,
    /// Provider (doctor/hospital) requesting access.
    pub provider: Address,
    /// Reason code for the emergency access.
    pub reason: Symbol,
    /// Address that created the request.
    pub requester: Address,
    /// List of approvers who have signed off.
    pub approvals: Vec<Address>,
    /// Timestamp when the request was created.
    pub created_at: u64,
    /// Whether the request has been granted.
    pub granted: bool,
    /// Timestamp when the request was granted (0 if not yet granted).
    pub granted_at: u64,
    /// Timestamp when the access expires.
    pub expires_at: u64,
}

/// A record of an access approval with full audit metadata.
#[derive(Clone, Debug)]
#[contracttype]
pub struct AccessApproval {
    /// The request this approval is for.
    pub request_id: u64,
    /// Address that approved the request.
    pub approver: Address,
    /// Timestamp when the approval was given.
    pub approved_at: u64,
    /// Ledger sequence when the approval was recorded.
    pub ledger_sequence: u32,
    /// Whether this approval was the one that granted access (reached threshold).
    pub final_approval: bool,
}

// ==================== Data Types ====================

#[derive(Clone)]
#[contracttype]
pub struct EmergencyAccessRecord {
    pub patient: Address,
    pub provider: Address,
    pub requested_duration: u64,
    pub granted_at: u64,
    pub expiry_at: u64,
    pub approved: bool,
    pub approvers: Vec<Address>,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    ApprovalThreshold,
    TrustedApprovers,                  // Vec<Address> of configured approvers
    EmergencyAccess(Address, Address), // (patient, provider)
    Cooldown(Address),                 // approver -> last_used timestamp
    CooldownPeriod,                    // configurable cooldown in seconds (default 86400 = 24h)
    GlobalGrantCount,                  // total grants in current window
    GlobalGrantWindowStart,            // timestamp when current window started
    CircuitBreakerTripped,             // bool: auto-paused due to rate limit
    AccessRequest(u64),                // request_id -> EmergencyAccessRequest
    RequestCounter,                    // next request id (instance)
    RateLimitConfig,                   // instance: configurable rate-limit parameters
    RequesterWindow(Address),          // requester -> RequesterWindow (rolling window state)
    AuditCount,                        // next audit entry id (instance)
    AuditEntry(u64),                   // audit id -> AccessApproval
    AuditForRequest(u64),              // request_id -> Vec<AccessApproval>
}

// ==================== Contract ====================

/// Default cooldown period: 24 hours in seconds.
const DEFAULT_COOLDOWN_SECONDS: u64 = 86_400;
/// Default expiry for legacy multi-sig emergency requests (1 hour).
const DEFAULT_EXPIRY_SECONDS: u64 = 3_600;
/// Global rate limit: max grants per rolling window.
const GLOBAL_GRANT_LIMIT: u64 = 10;
/// Rolling window duration for global rate limit: 1 hour.
const GLOBAL_GRANT_WINDOW_SECONDS: u64 = 3_600;

#[contract]
pub struct EmergencyAccessOverride;

#[contractimpl]
impl EmergencyAccessOverride {
    pub fn initialize(
        env: Env,
        admin: Address,
        approvers: Vec<Address>,
        threshold: u32,
    ) -> Result<(), Error> {
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
        admin.require_auth();
        // Delegate approval-set validation to the shared multi-sig helper.
        multi_sig::validate_approval_set(&approvers, threshold)
            .map_err(|_| Error::InvalidThreshold)?;

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ApprovalThreshold, &threshold);
        env.storage()
            .instance()
            .set(&DataKey::CooldownPeriod, &DEFAULT_COOLDOWN_SECONDS);
        // Store the approver set as a Vec<Address> in instance storage so
        // that the shared multi-sig helper can validate membership. The
        // previous implementation kept a per-address boolean in persistent
        // storage; under soroban-sdk 21.x we consolidate to a single
        // instance entry which the multi-sig helpers can iterate directly.
        env.storage()
            .instance()
            .set(&DataKey::TrustedApprovers, &approvers);

        events::record_audit_entry(
            &env,
            events::AuditAction::Initialized,
            &admin,
            None,
            None,
            symbol_short!("OK"),
        );
        events::publish_initialization(&env, &admin);
        Ok(())
    }

    pub fn grant_emergency_access(
        env: Env,
        approver: Address,
        patient: Address,
        provider: Address,
        duration_seconds: u64,
    ) -> Result<bool, Error> {
        approver.require_auth();
        Self::require_initialized(&env)?;

        if duration_seconds == 0 {
            return Err(Error::InvalidDuration);
        }

        // Delegate approver-membership check to the shared multi-sig helper.
        let trusted: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::TrustedApprovers)
            .ok_or(Error::NotInitialized)?;
        multi_sig::validate_approver(&approver, &trusted).map_err(|_| Error::Unauthorized)?;

        let now = env.ledger().timestamp();

        // Circuit breaker: reject if auto-paused due to rate limit
        let tripped: bool = env
            .storage()
            .instance()
            .get(&DataKey::CircuitBreakerTripped)
            .unwrap_or(false);
        if tripped {
            return Err(Error::RateLimitExceeded);
        }

        // Rate limiting: enforce per-caller cooldown
        let cooldown_period: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CooldownPeriod)
            .unwrap_or(DEFAULT_COOLDOWN_SECONDS);

        // Check cooldown only if the approver has been used before.
        // Using `Option` distinguishes "never called" (None) from
        // "called at timestamp 0" (Some(0)), which matters in tests
        // where the ledger timestamp starts at 0.
        if let Some(last_used) = env
            .storage()
            .persistent()
            .get::<_, u64>(&DataKey::Cooldown(approver.clone()))
        {
            let next_allowed_at = last_used.saturating_add(cooldown_period);
            if now < next_allowed_at {
                events::publish_rate_limit_exceeded(&env, &approver, next_allowed_at, now);
                return Err(Error::RateLimitExceeded);
            }
        }

        // Record this invocation timestamp
        env.storage()
            .persistent()
            .set(&DataKey::Cooldown(approver.clone()), &now);

        let key = DataKey::EmergencyAccess(patient.clone(), provider.clone());
        let mut record: EmergencyAccessRecord =
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(EmergencyAccessRecord {
                    patient: patient.clone(),
                    provider: provider.clone(),
                    requested_duration: duration_seconds,
                    granted_at: 0,
                    expiry_at: 0,
                    approved: false,
                    approvers: Vec::new(&env),
                });

        if record.approved && now < record.expiry_at {
            // Already granted and still valid
            return Ok(true);
        }

        // Use shared helper for idempotent approval. add_approval returns
        // false if the approver already approved; we preserve the existing
        // duplicate-approval event so callers can observe rejection.
        let newly_added = multi_sig::add_approval(approver.clone(), &mut record.approvers);
        if !newly_added {
            events::publish_duplicate_approval(&env, &patient, &provider, &approver, now);
            return Ok(false);
        }

        // Determine if approval threshold reached
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ApprovalThreshold)
            .ok_or(Error::NotInitialized)?;

        if multi_sig::check_approval_status(&record.approvers, threshold, false)
            == ApprovalStatus::Ready
        {
            record.approved = true;
            record.granted_at = now;
            record.expiry_at = now.saturating_add(duration_seconds);
            env.storage().persistent().set(&key, &record);

            // Global rate limit: track grants in rolling window
            let window_start: u64 = env
                .storage()
                .instance()
                .get(&DataKey::GlobalGrantWindowStart)
                .unwrap_or(now);
            let mut grant_count: u64 = env
                .storage()
                .instance()
                .get(&DataKey::GlobalGrantCount)
                .unwrap_or(0);

            if now.saturating_sub(window_start) >= GLOBAL_GRANT_WINDOW_SECONDS {
                // Reset window
                grant_count = 0;
                env.storage()
                    .instance()
                    .set(&DataKey::GlobalGrantWindowStart, &now);
            }
            grant_count = grant_count.saturating_add(1);
            env.storage()
                .instance()
                .set(&DataKey::GlobalGrantCount, &grant_count);

            if grant_count >= GLOBAL_GRANT_LIMIT {
                // Trip circuit breaker and emit alert
        env.storage()
            .instance()
            .set(&DataKey::CircuitBreakerTripped, &true);
        events::record_audit_entry(
            &env,
            events::AuditAction::CircuitTripped,
            &approver,
            Some(patient.clone()),
            Some(provider.clone()),
            symbol_short!("GRANT"),
        );
        events::publish_rate_limit_exceeded(&env, &approver, now, now);
        return Err(Error::RateLimitExceeded);
            }

            events::record_audit_entry(
                &env,
                events::AuditAction::GrantGranted,
                &approver,
                Some(patient.clone()),
                Some(provider.clone()),
                symbol_short!("OK"),
            );
            events::publish_emergency_access_granted(
                &env,
                &patient,
                &provider,
                record.expiry_at,
                now,
            );
            return Ok(true);
        }

        env.storage().persistent().set(&key, &record);
        events::record_audit_entry(
            &env,
            events::AuditAction::GrantApproved,
            &approver,
            Some(patient.clone()),
            Some(provider.clone()),
            symbol_short!("OK"),
        );
        events::publish_emergency_access_approved(&env, &patient, &provider, &approver, now);
        Ok(false)
    }

    /// Reset the circuit breaker. Only callable by admin after investigation.
    pub fn reset_circuit_breaker(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if stored_admin != admin {
            return Err(Error::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DataKey::CircuitBreakerTripped, &false);
        env.storage()
            .instance()
            .set(&DataKey::GlobalGrantCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::GlobalGrantWindowStart, &env.ledger().timestamp());
        events::record_audit_entry(
            &env,
            events::AuditAction::CircuitReset,
            &admin,
            None,
            None,
            symbol_short!("OK"),
        );
        Ok(())
    }

    /// Update the cooldown period. Only callable by admin (governance-gated).
    pub fn update_cooldown_period(
        env: Env,
        admin: Address,
        new_period_seconds: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if stored_admin != admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .set(&DataKey::CooldownPeriod, &new_period_seconds);

        events::record_audit_entry(
            &env,
            events::AuditAction::CooldownUpdated,
            &admin,
            None,
            None,
            symbol_short!("OK"),
        );
        events::publish_cooldown_updated(&env, &admin, new_period_seconds);
        Ok(())
    }

    /// Get the current cooldown period in seconds.
    pub fn get_cooldown_period(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::CooldownPeriod)
            .unwrap_or(DEFAULT_COOLDOWN_SECONDS)
    }

    pub fn check_emergency_access(
        env: Env,
        patient: Address,
        provider: Address,
    ) -> Result<bool, Error> {
        Self::require_initialized(&env)?;

        let now = env.ledger().timestamp();
        let key = DataKey::EmergencyAccess(patient.clone(), provider.clone());

        if let Some(record) = env
            .storage()
            .persistent()
            .get::<_, EmergencyAccessRecord>(&key)
        {
            if record.approved && record.expiry_at > now {
                events::record_audit_entry(
                    &env,
                    events::AuditAction::AccessCheck,
                    &env.current_contract_address(),
                    Some(patient.clone()),
                    Some(provider.clone()),
                    symbol_short!("TRUE"),
                );
                events::publish_emergency_access_checked(&env, &patient, &provider, true, now);
                return Ok(true);
            }
        }

        events::record_audit_entry(
            &env,
            events::AuditAction::AccessCheck,
            &env.current_contract_address(),
            Some(patient.clone()),
            Some(provider.clone()),
            symbol_short!("FALSE"),
        );
        events::publish_emergency_access_checked(&env, &patient, &provider, false, now);
        Ok(false)
    }

    pub fn revoke_emergency_access(
        env: Env,
        admin: Address,
        patient: Address,
        provider: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let is_admin = env.storage().instance().get(&DataKey::Admin);
        if is_admin != Some(admin.clone()) {
            return Err(Error::Unauthorized);
        }

        let key = DataKey::EmergencyAccess(patient.clone(), provider.clone());
        let mut record: EmergencyAccessRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::RecordNotFound)?;

        record.approved = false;
        record.expiry_at = 0;
        record.granted_at = 0;
        record.approvers = Vec::new(&env);

        env.storage().persistent().set(&key, &record);

        events::record_audit_entry(
            &env,
            events::AuditAction::GrantRevoked,
            &admin,
            Some(patient.clone()),
            Some(provider.clone()),
            symbol_short!("OK"),
        );
        events::publish_emergency_access_revoked(
            &env,
            &patient,
            &provider,
            env.ledger().timestamp(),
        );
        Ok(())
    }

    pub fn get_emergency_access_record(
        env: Env,
        patient: Address,
        provider: Address,
    ) -> Option<EmergencyAccessRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::EmergencyAccess(patient, provider))
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Submit an emergency access request subject to per-requester rate limiting.
    /// Returns the assigned request id on success.
    pub fn submit_emergency_access_request(
        env: Env,
        requester: Address,
        patient: Address,
        provider: Address,
        reason: Symbol,
        duration_seconds: u64,
    ) -> Result<u64, Error> {
        requester.require_auth();
        Self::require_initialized(&env)?;

        let config: RateLimitConfig = env
            .storage()
            .instance()
            .get(&DataKey::RateLimitConfig)
            .unwrap_or_default();

        let now = env.ledger().timestamp();
        let window_key = DataKey::RequesterWindow(requester.clone());
        let mut window: RequesterWindow = env
            .storage()
            .persistent()
            .get(&window_key)
            .unwrap_or(RequesterWindow {
                window_start: now,
                request_count: 0,
            });

        // Reset the window once it has elapsed.
        if now.saturating_sub(window.window_start) >= config.window_seconds {
            window.window_start = now;
            window.request_count = 0;
        }

        if window.request_count >= config.max_requests_per_window {
            return Err(Error::RateLimitExceeded);
        }
        window.request_count = window.request_count.saturating_add(1);
        env.storage().persistent().set(&window_key, &window);

        let request_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RequestCounter)
            .unwrap_or(0)
            .saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::RequestCounter, &request_id);

        let request = EmergencyAccessRequest {
            request_id,
            patient: patient.clone(),
            provider: provider.clone(),
            reason,
            requester: requester.clone(),
            approvals: Vec::new(&env),
            created_at: now,
            granted: false,
            granted_at: 0,
            expires_at: now.saturating_add(duration_seconds),
        };
        env.storage()
            .persistent()
            .set(&DataKey::AccessRequest(request_id), &request);

        Ok(request_id)
    }

    /// Read an emergency access request by id.
    pub fn get_access_request(env: Env, request_id: u64) -> Option<EmergencyAccessRequest> {
        env.storage()
            .persistent()
            .get(&DataKey::AccessRequest(request_id))
    }

    /// Update the rate-limit configuration. Only callable by admin.
    pub fn update_rate_limit_config(
        env: Env,
        admin: Address,
        config: RateLimitConfig,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if stored_admin != admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .set(&DataKey::RateLimitConfig, &config);
        Ok(())
    }

    /// Read the current rate-limit configuration.
    pub fn get_rate_limit_config(env: Env) -> RateLimitConfig {
        env.storage()
            .instance()
            .get(&DataKey::RateLimitConfig)
            .unwrap_or_default()
    }

    /// Approve an emergency access request. Returns true when this approval
    /// reaches the configured threshold and access is granted.
    pub fn approve_access_request(
        env: Env,
        approver: Address,
        request_id: u64,
    ) -> Result<bool, Error> {
        approver.require_auth();
        Self::require_initialized(&env)?;

        let trusted: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::TrustedApprovers)
            .ok_or(Error::NotInitialized)?;
        multi_sig::validate_approver(&approver, &trusted).map_err(|_| Error::Unauthorized)?;

        let mut request: EmergencyAccessRequest = env
            .storage()
            .persistent()
            .get(&DataKey::AccessRequest(request_id))
            .ok_or(Error::RecordNotFound)?;

        if request.granted {
            return Ok(true);
        }

        if !multi_sig::add_approval(approver.clone(), &mut request.approvers) {
            events::publish_duplicate_approval(
                &env,
                &request.patient,
                &request.provider,
                &approver,
                env.ledger().timestamp(),
            );
            return Ok(false);
        }

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ApprovalThreshold)
            .ok_or(Error::NotInitialized)?;
        let reached = multi_sig::check_approval_status(&request.approvers, threshold, false)
            == ApprovalStatus::Ready;

        if reached {
            request.granted = true;
            request.granted_at = env.ledger().timestamp();
        }
        env.storage()
            .persistent()
            .set(&DataKey::AccessRequest(request_id), &request);

        // Append an immutable audit-trail entry for this approval.
        let audit_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AuditCount)
            .unwrap_or(0)
            .saturating_add(1);
        env.storage().instance().set(&DataKey::AuditCount, &audit_id);

        let entry = AccessApproval {
            request_id,
            approver: approver.clone(),
            approved_at: env.ledger().timestamp(),
            ledger_sequence: env.ledger().sequence(),
            final_approval: reached,
        };
        env.storage()
            .persistent()
            .set(&DataKey::AuditEntry(audit_id), &entry);

        let mut trail: Vec<AccessApproval> = env
            .storage()
            .persistent()
            .get(&DataKey::AuditForRequest(request_id))
            .unwrap_or_else(|| Vec::new(&env));
        trail.push_back(entry);
        env.storage()
            .persistent()
            .set(&DataKey::AuditForRequest(request_id), &trail);

        events::record_audit_entry(
            &env,
            if reached {
                events::AuditAction::GrantGranted
            } else {
                events::AuditAction::GrantApproved
            },
            &approver,
            Some(request.patient.clone()),
            Some(request.provider.clone()),
            symbol_short!("OK"),
        );
        if reached {
            events::publish_emergency_access_granted(
                &env,
                &request.patient,
                &request.provider,
                request.expires_at,
                env.ledger().timestamp(),
            );
        }

        Ok(reached)
    }

    /// Return the audit trail for a request in approval order.
    pub fn get_approval_audit_trail(env: Env, request_id: u64) -> Vec<AccessApproval> {
        env.storage()
            .persistent()
            .get(&DataKey::AuditForRequest(request_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Total number of audit-trail entries recorded across all requests.
    pub fn get_audit_trail_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AuditCount)
            .unwrap_or(0)
    }

    /// Read a single audit-trail entry by its sequential id.
    pub fn get_audit_trail_entry(env: Env, audit_id: u64) -> Option<AccessApproval> {
        env.storage()
            .persistent()
            .get(&DataKey::AuditEntry(audit_id))
    }

    #[must_use]
    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !governance_commons::is_initialized(env) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    /// On-chain health check endpoint.
    /// Returns true if the contract is initialized and operational.
    pub fn health_check(env: Env) -> bool {
        governance_commons::is_initialized(&env)
    }
}

// ============================================================
// Issue #655: M-of-N Multi-Sig Emergency Access Override
// ============================================================


#[derive(Clone, Debug)]
#[contracttype]
pub struct EmergencyRequest {
    pub patient_id: Symbol,
    pub reason: Symbol,
    pub requester: Address,
    pub approvals: Vec<Address>,
    pub created_at: u64,
    pub granted: bool,
}

#[contracttype]
pub enum EmergencyKey {
    Request(u64), // keyed by request_id
    Config,       // stores (approvers: Vec<Address>, required: u32)
    RequestCounter,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct MultiSigConfig {
    pub approvers: Vec<Address>,
    pub required_approvals: u32,
    pub expiry_seconds: u64,
}

/// Governance sets the approver set and required M.
pub fn configure_multisig(
    env: Env,
    admin: Address,
    approvers: Vec<Address>,
    required_approvals: u32,
    expiry_seconds: u64,
) {
    admin.require_auth();
    let config = MultiSigConfig {
        approvers,
        required_approvals,
        expiry_seconds,
    };
    env.storage()
        .persistent()
        .set(&EmergencyKey::Config, &config);
}

/// Any party creates a pending emergency access request.
pub fn request_emergency_access(
    env: Env,
    requester: Address,
    patient_id: Symbol,
    reason: Symbol,
) -> u64 {
    requester.require_auth();
    let counter: u64 = env
        .storage()
        .persistent()
        .get(&EmergencyKey::RequestCounter)
        .unwrap_or(0u64);
    let request_id = counter + 1;
    let request = EmergencyRequest {
        patient_id: patient_id.clone(),
        reason: reason.clone(),
        requester: requester.clone(),
        approvals: Vec::new(&env),
        created_at: env.ledger().timestamp(),
        granted: false,
    };
    env.storage()
        .persistent()
        .set(&EmergencyKey::Request(request_id), &request);
    env.storage()
        .persistent()
        .set(&EmergencyKey::RequestCounter, &request_id);
    env.events().publish(
        (Symbol::new(&env, "EmergencyRequested"),),
        (request_id, requester, patient_id),
    );
    request_id
}

/// An approver signs off on a pending request.
/// Access is granted automatically once M approvals are collected.
pub fn approve_emergency_access(
    env: Env,
    approver: Address,
    request_id: u64,
) -> Result<bool, Error> {
    approver.require_auth();
    let config: MultiSigConfig = env
        .storage()
        .persistent()
        .get(&EmergencyKey::Config)
        .ok_or(Error::NotInitialized)?;
    // Delegate approver-membership check to the shared multi-sig helper.
    multi_sig::validate_approver(&approver, &config.approvers).map_err(|_| Error::Unauthorized)?;
    let mut request: EmergencyRequest = env
        .storage()
        .persistent()
        .get(&EmergencyKey::Request(request_id))
        .ok_or(Error::RecordNotFound)?;
    if request.granted {
        return Err(Error::AlreadyInitialized); // reuse: already granted
    }
    let elapsed = env.ledger().timestamp() - request.created_at;
    if elapsed > config.expiry_seconds {
        return Err(Error::InvalidDuration); // reuse: expired
    }
    // add_approval is idempotent: returns false if approver already signed.
    if !multi_sig::add_approval(approver.clone(), &mut request.approvals) {
        return Err(Error::RateLimitExceeded); // reuse: already signed
    }
    env.events().publish(
        (Symbol::new(&env, "EmergencyApproval"),),
        (request_id, approver.clone()),
    );
    if multi_sig::check_approval_status(&request.approvals, config.required_approvals, false)
        == ApprovalStatus::Ready
    {
        request.granted = true;
        env.events().publish(
            (Symbol::new(&env, "EmergencyAccessGranted"),),
            (request_id, request.patient_id.clone()),
        );
    }
    env.storage()
        .persistent()
        .set(&EmergencyKey::Request(request_id), &request);
    Ok(request.granted)
}

/// Read a request's current state.
pub fn get_emergency_request(env: Env, request_id: u64) -> Option<EmergencyRequest> {
    env.storage()
        .persistent()
        .get(&EmergencyKey::Request(request_id))
}

#[cfg(test)]
mod multisig_tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Env, Symbol, Vec};

    /// Register a dummy contract and return its ID, so storage-using calls
    /// can be wrapped in `env.as_contract(&contract_id, || ...)`.
    fn register_dummy(env: &Env) -> Address {
        env.register_contract(None, EmergencyAccessOverride)
    }

    fn setup_config(env: &Env, contract_id: &Address, approvers: Vec<Address>, m: u32) -> Address {
        let admin = Address::generate(env);
        env.as_contract(contract_id, || {
            configure_multisig(
                env.clone(),
                admin.clone(),
                approvers,
                m,
                DEFAULT_EXPIRY_SECONDS,
            );
        });
        admin
    }

    #[test]
    fn test_m_approvals_grant_access() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let a1 = Address::generate(&env);
        let a2 = Address::generate(&env);
        let a3 = Address::generate(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1.clone());
        approvers.push_back(a2.clone());
        approvers.push_back(a3.clone());
        setup_config(&env, &contract_id, approvers, 2);
        let requester = Address::generate(&env);

        let id = env.as_contract(&contract_id, || {
            request_emergency_access(
                env.clone(),
                requester.clone(),
                Symbol::new(&env, "P001"),
                Symbol::new(&env, "cardiac_arrest"),
            )
        });

        env.as_contract(&contract_id, || {
            approve_emergency_access(env.clone(), a1.clone(), id).unwrap();
        });

        let granted = env.as_contract(&contract_id, || {
            approve_emergency_access(env.clone(), a2.clone(), id).unwrap()
        });
        assert!(granted);

        let req = env.as_contract(&contract_id, || {
            get_emergency_request(env.clone(), id).unwrap()
        });
        assert!(req.granted);
    }

    #[test]
    fn test_m_minus_1_does_not_grant() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let a1 = Address::generate(&env);
        let a2 = Address::generate(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1.clone());
        approvers.push_back(a2.clone());
        setup_config(&env, &contract_id, approvers, 2);
        let requester = Address::generate(&env);

        let id = env.as_contract(&contract_id, || {
            request_emergency_access(
                env.clone(),
                requester.clone(),
                Symbol::new(&env, "P002"),
                Symbol::new(&env, "reason"),
            )
        });

        let granted = env.as_contract(&contract_id, || {
            approve_emergency_access(env.clone(), a1.clone(), id).unwrap()
        });
        assert!(!granted);

        let req = env.as_contract(&contract_id, || {
            get_emergency_request(env.clone(), id).unwrap()
        });
        assert!(!req.granted);
    }

    #[test]
    fn test_expired_request_rejected() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let a1 = Address::generate(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1.clone());

        env.as_contract(&contract_id, || {
            configure_multisig(env.clone(), Address::generate(&env), approvers, 1, 10);
            // 10s expiry
        });

        let requester = Address::generate(&env);
        let id = env.as_contract(&contract_id, || {
            request_emergency_access(
                env.clone(),
                requester.clone(),
                Symbol::new(&env, "P003"),
                Symbol::new(&env, "reason"),
            )
        });

        // Fast-forward time past expiry
        env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(100);
        });

        let result = env.as_contract(&contract_id, || {
            approve_emergency_access(env.clone(), a1.clone(), id)
        });
        assert!(result.is_err());
    }
}
