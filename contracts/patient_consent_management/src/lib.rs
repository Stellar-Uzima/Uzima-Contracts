#![no_std]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

#[derive(Clone)]
#[contracttype]
pub struct ConsentRecord {
    pub patient: Address,
    pub provider: Address,
    pub granted_at: u64,
    pub revoked_at: u64,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct ConsentLog {
    pub records: Vec<ConsentRecord>,
    pub record_count: u32,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    ConsentStorage(Address),
    ProviderIndex(Address, Address),
}

#[contract]
pub struct PatientConsentManagement;

#[contractimpl]
impl PatientConsentManagement {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        events::publish_initialization(&env, &admin);
        Ok(())
    }

    pub fn grant_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        if patient == provider {
            return Err(Error::InvalidProvider);
        }
        let ts = env.ledger().timestamp();
        let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        if let Some(r) = env.storage().persistent().get::<_, ConsentRecord>(&key) {
            if r.active { return Err(Error::ConsentAlreadyExists); }
        }
        let record = ConsentRecord { patient: patient.clone(), provider: provider.clone(), granted_at: ts, revoked_at: 0, active: true };
        let mut log: ConsentLog = env.storage().persistent().get(&DataKey::ConsentStorage(patient.clone())).unwrap_or(ConsentLog { records: Vec::new(&env), record_count: 0 });
        log.records.push_back(record.clone());
        log.record_count += 1;
        env.storage().persistent().set(&DataKey::ConsentStorage(patient.clone()), &log);
        env.storage().persistent().set(&key, &record);
        events::publish_consent_granted(&env, &patient, &provider, ts);
        Ok(())
    }

    /// Grant consent to multiple providers in a single transaction.
    pub fn batch_grant_consent(env: Env, patient: Address, grantees: Vec<Address>) -> Result<u32, Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        let ts = env.ledger().timestamp();
        let mut granted: u32 = 0;
        for provider in grantees.iter() {
            if provider == patient { continue; }
            let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
            if let Some(r) = env.storage().persistent().get::<_, ConsentRecord>(&key) {
                if r.active { continue; }
            }
            let record = ConsentRecord { patient: patient.clone(), provider: provider.clone(), granted_at: ts, revoked_at: 0, active: true };
            let mut log: ConsentLog = env.storage().persistent().get(&DataKey::ConsentStorage(patient.clone())).unwrap_or(ConsentLog { records: Vec::new(&env), record_count: 0 });
            log.records.push_back(record.clone());
            log.record_count += 1;
            env.storage().persistent().set(&DataKey::ConsentStorage(patient.clone()), &log);
            env.storage().persistent().set(&key, &record);
            events::publish_consent_granted(&env, &patient, &provider, ts);
            granted += 1;
        }
        Ok(granted)
    }

    pub fn revoke_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        let ts = env.ledger().timestamp();
        let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        let mut record: ConsentRecord = env.storage().persistent().get(&key).ok_or(Error::ConsentNotFound)?;
        if !record.active { return Err(Error::ConsentNotFound); }
        record.revoked_at = ts;
        record.active = false;
        env.storage().persistent().set(&key, &record);
        let mut log: ConsentLog = env.storage().persistent().get(&DataKey::ConsentStorage(patient.clone())).ok_or(Error::ConsentNotFound)?;
        let mut updated = soroban_sdk::Vec::new(&env);
        for mut r in log.records.iter() {
            if r.provider == provider && r.patient == patient { r.revoked_at = ts; r.active = false; }
            updated.push_back(r);
        }
        log.records = updated;
        env.storage().persistent().set(&DataKey::ConsentStorage(patient.clone()), &log);
        events::publish_consent_revoked(&env, &patient, &provider, ts);
        Ok(())
    }

    pub fn check_consent(env: Env, patient: Address, provider: Address) -> Result<bool, Error> {
        Self::require_initialized(&env)?;
        let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        let result = env.storage().persistent().get::<_, ConsentRecord>(&key).map(|r| r.active).unwrap_or(false);
        events::publish_consent_checked(&env, &patient, &provider, result);
        Ok(result)
    }

    pub fn get_patient_consents(env: Env, patient: Address) -> Option<ConsentLog> {
        env.storage().persistent().get(&DataKey::ConsentStorage(patient))
    }

    pub fn get_active_consent_count(env: Env, patient: Address) -> u32 {
        env.storage().persistent().get::<_, ConsentLog>(&DataKey::ConsentStorage(patient))
            .map(|log| log.records.iter().filter(|r| r.active).count() as u32)
            .unwrap_or(0)
    }

    pub fn verify_consent_with_audit(env: Env, patient: Address, provider: Address) -> Result<(bool, u64, u64), Error> {
        Self::require_initialized(&env)?;
        let key = DataKey::ProviderIndex(patient, provider);
        env.storage().persistent().get::<_, ConsentRecord>(&key)
            .map(|r| (r.active, r.granted_at, r.revoked_at))
            .ok_or(Error::ConsentNotFound)
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)
    }

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) { return Err(Error::NotInitialized); }
        Ok(())
    }
}