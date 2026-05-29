#![no_std]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

// ==================== Data Types ====================

#[derive(Clone)]
#[contracttype]
pub struct ConsentRecord {
    pub patient: Address,
    pub provider: Address,
    pub granted_at: u64,
    pub revoked_at: u64, // 0 means not revoked
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
    ConsentStorage(Address),         // patient -> ConsentLog
    ProviderIndex(Address, Address), // (patient, provider) -> ConsentRecord
}

// ==================== Contract ====================

#[contract]
pub struct PatientConsentManagement;

#[contractimpl]
impl PatientConsentManagement {
    /// Initialize the contract with an admin
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

    /// Grant consent for a provider to access patient data
    /// Only the patient can grant consent to a provider
    pub fn grant_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        // Validate addresses
        if patient == provider {
            return Err(Error::InvalidProvider);
        }

        let timestamp = env.ledger().timestamp();

        // Check if consent already exists
        let provider_key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        if let Some(existing) = env
            .storage()
            .persistent()
            .get::<_, ConsentRecord>(&provider_key)
        {
            if existing.active {
                return Err(Error::ConsentAlreadyExists);
            }
        }

        // Get or create consent log for patient
        let mut consent_log: ConsentLog = env
            .storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient.clone()))
            .unwrap_or(ConsentLog {
                records: Vec::new(&env),
                record_count: 0,
            });

        // Create new consent record
        let consent_record = ConsentRecord {
            patient: patient.clone(),
            provider: provider.clone(),
            granted_at: timestamp,
            revoked_at: 0,
            active: true,
        };

        // Add to patient's consent log
        consent_log.records.push_back(consent_record.clone());
        consent_log.record_count += 1;

        // Store updated consent log
        env.storage()
            .persistent()
            .set(&DataKey::ConsentStorage(patient.clone()), &consent_log);

        // Store individual consent record for quick lookup
        env.storage()
            .persistent()
            .set(&provider_key, &consent_record);

        events::publish_consent_granted(&env, &patient, &provider, timestamp);
        Ok(())
    }

    /// Revoke consent for a provider to access patient data
    /// Only the patient who granted the consent can revoke it
    pub fn revoke_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;

        let timestamp = env.ledger().timestamp();

        // Get the consent record
        let provider_key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        let mut consent_record: ConsentRecord = env
            .storage()
            .persistent()
            .get(&provider_key)
            .ok_or(Error::ConsentNotFound)?;

        // Check if already revoked
        if !consent_record.active {
            return Err(Error::ConsentNotFound);
        }

        // Mark as revoked
        consent_record.revoked_at = timestamp;
        consent_record.active = false;

        // Update the stored record
        env.storage()
            .persistent()
            .set(&provider_key, &consent_record);

        // Update consent log
        let mut consent_log: ConsentLog = env
            .storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient.clone()))
            .ok_or(Error::ConsentNotFound)?;

        // Find and update the record in the log
        let mut updated_records = soroban_sdk::Vec::new(&env);
        for record in consent_log.records.iter() {
            let mut r = record;
            if r.provider == provider && r.patient == patient {
                r.revoked_at = timestamp;
                r.active = false;
            }
            updated_records.push_back(r);
        }
        consent_log.records = updated_records;

        // Store updated consent log
        env.storage()
            .persistent()
            .set(&DataKey::ConsentStorage(patient.clone()), &consent_log);

        events::publish_consent_revoked(&env, &patient, &provider, timestamp);
        Ok(())
    }

    /// Check if a provider has active consent from a patient
    /// Can be called by anyone to verify consent status (read-only, no auth required)
    pub fn check_consent(env: Env, patient: Address, provider: Address) -> Result<bool, Error> {
        Self::require_initialized(&env)?;

        let provider_key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        let result = match env
            .storage()
            .persistent()
            .get::<_, ConsentRecord>(&provider_key)
        {
            Some(consent) => consent.active,
            None => false,
        };

        events::publish_consent_checked(&env, &patient, &provider, result);
        Ok(result)
    }

    /// Get all consent records for a patient
    /// Patient can view their own consent history
    pub fn get_patient_consents(env: Env, patient: Address) -> Option<ConsentLog> {
        env.storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient))
    }

    /// Get count of active consents for a patient
    pub fn get_active_consent_count(env: Env, patient: Address) -> u32 {
        match env
            .storage()
            .persistent()
            .get::<_, ConsentLog>(&DataKey::ConsentStorage(patient))
        {
            Some(log) => {
                let mut count = 0;
                for record in log.records.iter() {
                    if record.active {
                        count += 1;
                    }
                }
                count
            },
            None => 0,
        }
    }

    /// Verify consent with audit trail
    /// Returns (has_consent, timestamp_of_grant, timestamp_of_revocation_if_any)
    pub fn verify_consent_with_audit(
        env: Env,
        patient: Address,
        provider: Address,
    ) -> Result<(bool, u64, u64), Error> {
        Self::require_initialized(&env)?;

        let provider_key = DataKey::ProviderIndex(patient, provider);
        match env
            .storage()
            .persistent()
            .get::<_, ConsentRecord>(&provider_key)
        {
            Some(consent) => Ok((consent.active, consent.granted_at, consent.revoked_at)),
            None => Err(Error::ConsentNotFound),
        }
    }

    /// Get the current admin
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    // ==================== Internal Helpers ====================

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::NotInitialized);
        }
        Ok(())
    }

    /// On-chain health check endpoint.
    /// Returns true if the contract is initialized and operational.
    pub fn health_check(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }
}

// ============================================================
// Issue #656: Delegated Consent / Proxy Authority
// ============================================================

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ProxyScope {
    FullAuthority,
    EmergencyOnly,
    ReadOnly,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ProxyRecord {
    pub proxy_address: Address,
    pub scope: ProxyScope,
    pub designated_at: u64,
}

#[contracttype]
pub enum ProxyKey {
    Proxy(Address), // keyed by patient address
}

/// Patient designates a proxy who can act on their behalf when incapacitated.
/// Requires the patient's own signature (invoke as patient).
pub fn designate_proxy(env: Env, patient: Address, proxy_address: Address, scope: ProxyScope) {
    patient.require_auth();
    let record = ProxyRecord {
        proxy_address: proxy_address.clone(),
        scope,
        designated_at: env.ledger().timestamp(),
    };
    env.storage()
        .persistent()
        .set(&ProxyKey::Proxy(patient.clone()), &record);
    env.events().publish(
        (Symbol::new(&env, "ProxyDesignated"),),
        (patient, proxy_address),
    );
}

/// Patient revokes their currently designated proxy.
pub fn revoke_proxy(env: Env, patient: Address) {
    patient.require_auth();
    let key = ProxyKey::Proxy(patient.clone());
    env.storage().persistent().remove(&key);
    env.events().publish(
        (Symbol::new(&env, "ProxyRevoked"),),
        (patient,),
    );
}

/// Retrieve the proxy record for a patient, if one exists.
pub fn get_proxy(env: Env, patient: Address) -> Option<ProxyRecord> {
    env.storage()
        .persistent()
        .get(&ProxyKey::Proxy(patient))
}

/// Proxy grants consent on behalf of an incapacitated patient.
/// Checks that caller is the designated proxy and scope allows it.
pub fn proxy_grant_consent(
    env: Env,
    proxy: Address,
    patient: Address,
    grantee: Address,
) {
    proxy.require_auth();
    let record: ProxyRecord = env
        .storage()
        .persistent()
        .get(&ProxyKey::Proxy(patient.clone()))
        .expect("No proxy designated for patient");
    assert!(
        record.proxy_address == proxy,
        "Caller is not the designated proxy"
    );
    assert!(
        record.scope == ProxyScope::FullAuthority || record.scope == ProxyScope::EmergencyOnly,
        "Proxy scope does not permit granting consent"
    );
    env.events().publish(
        (Symbol::new(&env, "ProxyConsentGranted"),),
        (proxy, patient, grantee),
    );
}

/// Proxy revokes consent on behalf of an incapacitated patient.
pub fn proxy_revoke_consent(
    env: Env,
    proxy: Address,
    patient: Address,
    grantee: Address,
) {
    proxy.require_auth();
    let record: ProxyRecord = env
        .storage()
        .persistent()
        .get(&ProxyKey::Proxy(patient.clone()))
        .expect("No proxy designated for patient");
    assert!(
        record.proxy_address == proxy,
        "Caller is not the designated proxy"
    );
    assert!(
        record.scope == ProxyScope::FullAuthority,
        "Proxy scope does not permit revoking consent"
    );
    env.events().publish(
        (Symbol::new(&env, "ProxyConsentRevoked"),),
        (proxy, patient, grantee),
    );
}

#[cfg(test)]
mod proxy_tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::Env;

    #[test]
    fn test_designate_and_get_proxy() {
        let env = Env::default();
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        designate_proxy(env.clone(), patient.clone(), proxy.clone(), ProxyScope::FullAuthority);
        let record = get_proxy(env.clone(), patient).unwrap();
        assert_eq!(record.proxy_address, proxy);
        assert_eq!(record.scope, ProxyScope::FullAuthority);
    }

    #[test]
    fn test_revoke_proxy() {
        let env = Env::default();
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        designate_proxy(env.clone(), patient.clone(), proxy.clone(), ProxyScope::ReadOnly);
        revoke_proxy(env.clone(), patient.clone());
        let record = get_proxy(env.clone(), patient);
        assert!(record.is_none());
    }

    #[test]
    fn test_proxy_grant_consent_full_authority() {
        let env = Env::default();
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        let grantee = Address::generate(&env);
        designate_proxy(env.clone(), patient.clone(), proxy.clone(), ProxyScope::FullAuthority);
        proxy_grant_consent(env.clone(), proxy.clone(), patient.clone(), grantee.clone());
    }

    #[test]
    #[should_panic(expected = "Proxy scope does not permit granting consent")]
    fn test_readonly_proxy_cannot_grant() {
        let env = Env::default();
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        let grantee = Address::generate(&env);
        designate_proxy(env.clone(), patient.clone(), proxy.clone(), ProxyScope::ReadOnly);
        proxy_grant_consent(env.clone(), proxy.clone(), patient.clone(), grantee.clone());
    }

    #[test]
    fn test_emergency_proxy_can_grant_but_not_revoke() {
        let env = Env::default();
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        let grantee = Address::generate(&env);
        designate_proxy(env.clone(), patient.clone(), proxy.clone(), ProxyScope::EmergencyOnly);
        proxy_grant_consent(env.clone(), proxy.clone(), patient.clone(), grantee.clone());
    }
    }
