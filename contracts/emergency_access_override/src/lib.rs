#![no_std]
#![allow(dead_code)]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

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
    TrustedApprover(Address),          // approver -> bool (exists)
    EmergencyAccess(Address, Address), // (patient, provider)
}

// ==================== Contract ====================

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
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        if threshold == 0 || threshold > approvers.len() {
            return Err(Error::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ApprovalThreshold, &threshold);

        for approver in approvers.iter() {
            env.storage()
                .persistent()
                .set(&DataKey::TrustedApprover(approver.clone()), &true);
        }

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

        let is_trusted: Option<bool> = env
            .storage()
            .persistent()
            .get(&DataKey::TrustedApprover(approver.clone()));

        if is_trusted != Some(true) {
            return Err(Error::Unauthorized);
        }

        let now = env.ledger().timestamp();

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

        // Avoid duplicate approver records
        for a in record.approvers.iter() {
            if a == approver {
                // already approved by this approver, no changes
                events::publish_duplicate_approval(&env, &patient, &provider, &approver, now);
                return Ok(false);
            }
        }

        record.approvers.push_back(approver.clone());

        // Determine if approval threshold reached
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ApprovalThreshold)
            .unwrap();
        let current = record.approvers.len();

        if current >= threshold {
            record.approved = true;
            record.granted_at = now;
            record.expiry_at = now.saturating_add(duration_seconds);
            env.storage().persistent().set(&key, &record);
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
        events::publish_emergency_access_approved(&env, &patient, &provider, &approver, now);
        Ok(false)
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
                events::publish_emergency_access_checked(&env, &patient, &provider, true, now);
                return Ok(true);
            }
        }

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

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }
}


// ============================================================
// Issue #655: M-of-N Multi-Sig Emergency Access Override
// ============================================================

const DEFAULT_EXPIRY_SECONDS: u64 = 3600; // 1 hour

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
    Request(u64),       // keyed by request_id
    Config,             // stores (approvers: Vec<Address>, required: u32)
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
pub fn approve_emergency_access(env: Env, approver: Address, request_id: u64) {
    approver.require_auth();
    let config: MultiSigConfig = env
        .storage()
        .persistent()
        .get(&EmergencyKey::Config)
        .expect("MultiSig not configured");
    assert!(
        config.approvers.contains(&approver),
        "Caller is not a designated approver"
    );
    let mut request: EmergencyRequest = env
        .storage()
        .persistent()
        .get(&EmergencyKey::Request(request_id))
        .expect("Request not found");
    assert!(!request.granted, "Request already granted");
    let elapsed = env.ledger().timestamp() - request.created_at;
    assert!(elapsed <= config.expiry_seconds, "Request has expired");
    assert!(
        !request.approvals.contains(&approver),
        "Approver already signed"
    );
    request.approvals.push_back(approver.clone());
    env.events().publish(
        (Symbol::new(&env, "EmergencyApproval"),),
        (request_id, approver.clone()),
    );
    if request.approvals.len() >= config.required_approvals {
        request.granted = true;
        env.events().publish(
            (Symbol::new(&env, "EmergencyAccessGranted"),),
            (request_id, request.patient_id.clone()),
        );
    }
    env.storage()
        .persistent()
        .set(&EmergencyKey::Request(request_id), &request);
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
    use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
    use soroban_sdk::{Env, Symbol, Vec};

    fn setup_config(env: &Env, approvers: Vec<Address>, m: u32) -> Address {
        let admin = Address::generate(env);
        configure_multisig(
            env.clone(),
            admin.clone(),
            approvers,
            m,
            DEFAULT_EXPIRY_SECONDS,
        );
        admin
    }

    #[test]
    fn test_m_approvals_grant_access() {
        let env = Env::default();
        env.mock_all_auths();
        let a1 = Address::generate(&env);
        let a2 = Address::generate(&env);
        let a3 = Address::generate(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1.clone());
        approvers.push_back(a2.clone());
        approvers.push_back(a3.clone());
        setup_config(&env, approvers, 2);
        let requester = Address::generate(&env);
        let id = request_emergency_access(
            env.clone(),
            requester,
            Symbol::new(&env, "P001"),
            Symbol::new(&env, "cardiac_arrest"),
        );
        approve_emergency_access(env.clone(), a1.clone(), id);
        approve_emergency_access(env.clone(), a2.clone(), id);
        let req = get_emergency_request(env.clone(), id).unwrap();
        assert!(req.granted);
    }

    #[test]
    fn test_m_minus_1_does_not_grant() {
        let env = Env::default();
        env.mock_all_auths();
        let a1 = Address::generate(&env);
        let a2 = Address::generate(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1.clone());
        approvers.push_back(a2.clone());
        setup_config(&env, approvers, 2);
        let requester = Address::generate(&env);
        let id = request_emergency_access(
            env.clone(),
            requester,
            Symbol::new(&env, "P002"),
            Symbol::new(&env, "reason"),
        );
        approve_emergency_access(env.clone(), a1.clone(), id);
        let req = get_emergency_request(env.clone(), id).unwrap();
        assert!(!req.granted);
    }

    #[test]
    #[should_panic(expected = "Request has expired")]
    fn test_expired_request_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let a1 = Address::generate(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(a1.clone());
        configure_multisig(env.clone(), Address::generate(&env), approvers, 1, 10); // 10s expiry
        let requester = Address::generate(&env);
        let id = request_emergency_access(
            env.clone(),
            requester,
            Symbol::new(&env, "P003"),
            Symbol::new(&env, "reason"),
        );
        // Fast-forward time past expiry
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 100,
            ..env.ledger().get()
        });
        approve_emergency_access(env.clone(), a1.clone(), id);
    }
}
