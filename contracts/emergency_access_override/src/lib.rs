#![no_std]
#![allow(dead_code)]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Vec};

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
        if threshold == 0 || threshold > approvers.len() as u32 {
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
            return Err(Error::NotAuthorized);
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

        if current as u32 >= threshold {
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
            return Err(Error::NotAuthorized);
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
