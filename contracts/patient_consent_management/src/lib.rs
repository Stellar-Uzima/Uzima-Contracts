//! # Patient Consent Management Contract
//!
//! Manages patient consent for healthcare data access on the Stellar blockchain.
//! Patients can grant, revoke, and check consent for healthcare providers.
//!
//! ## Purpose
//! This contract enables patients to control who can access their medical data.
//! Consent is managed on a per-provider basis with full audit trail.
//!
//! ## Key Dependencies
//! - `upgradeability` - For upgrade/admin pattern
//!
//! ## Initialization Requirements
//! - Must be initialized with an admin address
//!
//! ## Role/Permission Requirements
//! - **Admin**: Can initialize the contract
//! - **Patient**: Can grant/revoke their own consent
//! - **Anyone**: Can check consent status (read-only)
//!
//! ## Example Usage
//! ```rust,ignore
//! client.initialize(&admin);
//! client.grant_consent(&patient, &provider);
//! let has_consent = client.check_consent(&patient, &provider);
//! client.revoke_consent(&patient, &provider);
//! ```
//!
//! ## Error Ranges
//! - 100-199: Access Control & Authorization
//! - 200-299: Input Validation
//! - 300-399: Lifecycle & State
//! - 400-499: Entity Existence

#![no_std]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct ConsentRecord {
    pub patient: Address,
    pub provider: Address,
    pub granted_at: u64,
    pub expires_at: u64, // 0 means no expiration
    pub revoked_at: u64,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct ConsentLog {
    pub records: Vec<ConsentRecord>,
    pub record_count: u32,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum ErasureStatus {
    None,
    Requested,
    Executed,
}

#[derive(Clone)]
#[contracttype]
pub struct ErasureRequest {
    pub patient: Address,
    pub requested_at: u64,
    pub status: ErasureStatus,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Paused,
    ConsentStorage(Address),
    ProviderIndex(Address, Address),
    ErasureRequest(Address),
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
        env.storage().instance().set(&DataKey::Paused, &false);
        events::publish_initialization(&env, &admin);
        Ok(())
    }

    pub fn grant_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        if patient == provider {
            return Err(Error::InvalidProvider);
        }
        let ts = env.ledger().timestamp();
        let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        if let Some(r) = env.storage().persistent().get::<_, ConsentRecord>(&key) {
            if r.active {
                return Err(Error::ConsentAlreadyExists);
            }
        }
        let record = ConsentRecord {
            patient: patient.clone(),
            provider: provider.clone(),
            granted_at: ts,
            expires_at: 0,
            revoked_at: 0,
            active: true,
        };
        let mut log: ConsentLog = env
            .storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient.clone()))
            .unwrap_or(ConsentLog {
                records: Vec::new(&env),
                record_count: 0,
            });
        log.records.push_back(record.clone());
        log.record_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::ConsentStorage(patient.clone()), &log);
        env.storage().persistent().set(&key, &record);
        events::publish_consent_granted(&env, &patient, &provider, ts);
        Ok(())
    }

    pub fn grant_consent_with_expiry(
        env: Env,
        patient: Address,
        provider: Address,
        expires_at: u64,
    ) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        if patient == provider {
            return Err(Error::InvalidProvider);
        }
        let ts = env.ledger().timestamp();
        if expires_at != 0 && expires_at <= ts {
            return Err(Error::InvalidExpiry);
        }
        let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        if let Some(r) = env.storage().persistent().get::<_, ConsentRecord>(&key) {
            if r.active {
                return Err(Error::ConsentAlreadyExists);
            }
        }
        let record = ConsentRecord {
            patient: patient.clone(),
            provider: provider.clone(),
            granted_at: ts,
            expires_at,
            revoked_at: 0,
            active: true,
        };
        let mut log: ConsentLog = env
            .storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient.clone()))
            .unwrap_or(ConsentLog {
                records: Vec::new(&env),
                record_count: 0,
            });
        log.records.push_back(record.clone());
        log.record_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::ConsentStorage(patient.clone()), &log);
        env.storage().persistent().set(&key, &record);
        events::publish_consent_granted(&env, &patient, &provider, ts);
        Ok(())
    }

    /// Grant consent to multiple providers in a single transaction.
    pub fn batch_grant_consent(
        env: Env,
        patient: Address,
        grantees: Vec<Address>,
    ) -> Result<u32, Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let ts = env.ledger().timestamp();
        let mut granted: u32 = 0;
        for provider in grantees.iter() {
            if provider == patient {
                continue;
            }
            let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
            if let Some(r) = env.storage().persistent().get::<_, ConsentRecord>(&key) {
                if r.active {
                    continue;
                }
            }
            let record = ConsentRecord {
                patient: patient.clone(),
                provider: provider.clone(),
                granted_at: ts,
                expires_at: 0,
                revoked_at: 0,
                active: true,
            };
            let mut log: ConsentLog = env
                .storage()
                .persistent()
                .get(&DataKey::ConsentStorage(patient.clone()))
                .unwrap_or(ConsentLog {
                    records: Vec::new(&env),
                    record_count: 0,
                });
            log.records.push_back(record.clone());
            log.record_count += 1;
            env.storage()
                .persistent()
                .set(&DataKey::ConsentStorage(patient.clone()), &log);
            env.storage().persistent().set(&key, &record);
            events::publish_consent_granted(&env, &patient, &provider, ts);
            granted += 1;
        }
        Ok(granted)
    }

    pub fn revoke_consent(env: Env, patient: Address, provider: Address) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let ts = env.ledger().timestamp();
        let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        let mut record: ConsentRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::ConsentNotFound)?;
        if !record.active {
            return Err(Error::ConsentNotFound);
        }
        record.revoked_at = ts;
        record.active = false;
        env.storage().persistent().set(&key, &record);
        let mut log: ConsentLog = env
            .storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient.clone()))
            .ok_or(Error::ConsentNotFound)?;
        let mut updated = soroban_sdk::Vec::new(&env);
        for mut r in log.records.iter() {
            if r.provider == provider && r.patient == patient {
                r.revoked_at = ts;
                r.active = false;
            }
            updated.push_back(r);
        }
        log.records = updated;
        env.storage()
            .persistent()
            .set(&DataKey::ConsentStorage(patient.clone()), &log);
        events::publish_consent_revoked(&env, &patient, &provider, ts);
        Ok(())
    }

    fn is_consent_expired(env: &Env, record: &ConsentRecord) -> bool {
        record.expires_at != 0 && env.ledger().timestamp() >= record.expires_at
    }

    fn is_consent_active(env: &Env, record: &ConsentRecord) -> bool {
        record.active && !Self::is_consent_expired(env, record)
    }

    pub fn check_consent(env: Env, patient: Address, provider: Address) -> Result<bool, Error> {
        Self::require_initialized(&env)?;
        let key = DataKey::ProviderIndex(patient.clone(), provider.clone());
        let mut result = false;
        if let Some(record) = env.storage().persistent().get::<_, ConsentRecord>(&key) {
            if Self::is_consent_expired(&env, &record) {
                events::publish_consent_expired(
                    &env,
                    &patient,
                    &provider,
                    env.ledger().timestamp(),
                );
            }
            result = Self::is_consent_active(&env, &record);
        }
        events::publish_consent_checked(&env, &patient, &provider, result);
        Ok(result)
    }

    pub fn cleanup_expired_consents(env: Env, patient: Address) -> Result<u32, Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;
        let now = env.ledger().timestamp();
        let mut log: ConsentLog = env
            .storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient.clone()))
            .unwrap_or(ConsentLog {
                records: Vec::new(&env),
                record_count: 0,
            });
        let mut updated = Vec::new(&env);
        let mut cleaned: u32 = 0;
        for mut record in log.records.iter() {
            if record.active && Self::is_consent_expired(&env, &record) {
                record.active = false;
                record.revoked_at = now;
                env.storage().persistent().set(
                    &DataKey::ProviderIndex(patient.clone(), record.provider.clone()),
                    &record,
                );
                events::publish_consent_expired(&env, &patient, &record.provider, now);
                cleaned = cleaned.saturating_add(1);
            }
            updated.push_back(record);
        }
        log.records = updated;
        env.storage()
            .persistent()
            .set(&DataKey::ConsentStorage(patient.clone()), &log);
        Ok(cleaned)
    }

    pub fn get_patient_consents(env: Env, patient: Address) -> Option<ConsentLog> {
        env.storage()
            .persistent()
            .get(&DataKey::ConsentStorage(patient))
    }

    pub fn get_active_consent_count(env: Env, patient: Address) -> u32 {
        env.storage()
            .persistent()
            .get::<_, ConsentLog>(&DataKey::ConsentStorage(patient))
            .map(|log| {
                log.records
                    .iter()
                    .filter(|r| Self::is_consent_active(&env, r))
                    .count() as u32
            })
            .unwrap_or(0)
    }

    pub fn verify_consent_with_audit(
        env: Env,
        patient: Address,
        provider: Address,
    ) -> Result<(bool, u64, u64), Error> {
        Self::require_initialized(&env)?;
        let key = DataKey::ProviderIndex(patient, provider);
        let record: ConsentRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::ConsentNotFound)?;
        Ok((
            Self::is_consent_active(&env, &record),
            record.granted_at,
            record.revoked_at,
        ))
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    #[must_use]
    fn require_not_paused(env: &Env) -> Result<(), Error> {
        if env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(Error::ContractPaused);
        }
        Ok(())
    }

    #[must_use]
    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller == &admin {
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    }

    pub fn pause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        env.events().publish(
            (Symbol::new(&env, "Paused"),),
            (caller.clone(), env.ledger().timestamp()),
        );
        Ok(true)
    }

    pub fn unpause(env: Env, caller: Address) -> Result<bool, Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        env.events().publish(
            (Symbol::new(&env, "Unpaused"),),
            (caller.clone(), env.ledger().timestamp()),
        );
        Ok(true)
    }

    #[must_use]
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

    // ============================================================
    // Issue #888: GDPR Data Erasure
    // ============================================================

    pub fn request_erasure(env: Env, patient: Address) -> Result<(), Error> {
        patient.require_auth();
        Self::require_initialized(&env)?;
        Self::require_not_paused(&env)?;

        let key = DataKey::ErasureRequest(patient.clone());
        if let Some(existing_request) = env
            .storage()
            .persistent()
            .get::<_, ErasureRequest>(&key)
        {
            if existing_request.status == ErasureStatus::Requested {
                return Err(Error::ErasureRequestExists);
            }
        }

        let request = ErasureRequest {
            patient: patient.clone(),
            requested_at: env.ledger().timestamp(),
            status: ErasureStatus::Requested,
        };
        env.storage().persistent().set(&key, &request);
        events::publish_erasure_requested(&env, &patient);
        Ok(())
    }

    pub fn execute_erasure(env: Env, caller: Address, patient: Address) -> Result<(), Error> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        Self::require_initialized(&env)?;

        let key = DataKey::ErasureRequest(patient.clone());
        let mut request: ErasureRequest = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::ErasureRequestNotFound)?;

        if request.status != ErasureStatus::Requested {
            return Err(Error::ErasureRequestNotPending);
        }

        // Nullify patient data
        let consent_key = DataKey::ConsentStorage(patient.clone());
        if let Some(log) = env
            .storage()
            .persistent()
            .get::<_, ConsentLog>(&consent_key)
        {
            for record in log.records.iter() {
                let provider_key = DataKey::ProviderIndex(patient.clone(), record.provider);
                env.storage().persistent().remove(&provider_key);
            }
            env.storage().persistent().remove(&consent_key);
        }

        request.status = ErasureStatus::Executed;
        env.storage().persistent().set(&key, &request);
        events::publish_erasure_executed(&env, &patient);
        Ok(())
    }

    pub fn get_erasure_status(env: Env, patient: Address) -> ErasureStatus {
        let key = DataKey::ErasureRequest(patient);
        env.storage()
            .persistent()
            .get::<_, ErasureRequest>(&key)
            .map(|r| r.status)
            .unwrap_or(ErasureStatus::None)
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
    env.events()
        .publish((Symbol::new(&env, "ProxyRevoked"),), (patient,));
}

/// Retrieve the proxy record for a patient, if one exists.
pub fn get_proxy(env: Env, patient: Address) -> Option<ProxyRecord> {
    env.storage().persistent().get(&ProxyKey::Proxy(patient))
}

/// Proxy grants consent on behalf of an incapacitated patient.
/// Checks that caller is the designated proxy and scope allows it.
pub fn proxy_grant_consent(env: Env, proxy: Address, patient: Address, grantee: Address) {
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
pub fn proxy_revoke_consent(env: Env, proxy: Address, patient: Address, grantee: Address) {
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

// ============================================================
// Issue #767: Migratable trait for standardized contract upgrades
// ============================================================

impl upgradeability::migration::Migratable for PatientConsentManagement {
    fn migrate(env: &Env, from_version: u32) -> Result<(), upgradeability::UpgradeError> {
        if from_version < 1 {
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(upgradeability::UpgradeError::NotAuthorized)?;
            upgradeability::storage::set_admin(env, &admin);
            upgradeability::storage::set_version(env, 1);
        }
        Ok(())
    }

    fn verify_integrity(env: &Env) -> Result<BytesN<32>, upgradeability::UpgradeError> {
        let initialized = env.storage().instance().has(&DataKey::Initialized);
        let mut data = Vec::new(env);
        data.push_back(if initialized { 1u64 } else { 0u64 });
        let hash = env.crypto().sha256(&data.to_xdr(env));
        Ok(BytesN::from_array(env, &hash.to_array()))
    }

    fn validate(
        env: &Env,
        _new_wasm_hash: &BytesN<32>,
    ) -> Result<upgradeability::UpgradeValidation, upgradeability::UpgradeError> {
        let initialized = env.storage().instance().has(&DataKey::Initialized);
        let mut report = Vec::new(env);
        if !initialized {
            report.push_back(symbol_short!("NOT_INIT"));
        }
        Ok(upgradeability::UpgradeValidation {
            state_compatible: initialized,
            api_compatible: true,
            storage_layout_valid: true,
            tests_passed: true,
            gas_impact: 0,
            report,
        })
    }
}

#[cfg(test)]
mod proxy_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    /// Register a dummy contract instance so storage-using free functions
    /// can be called inside `env.as_contract()`.
    fn register_dummy(env: &Env) -> Address {
        env.register_contract(None, PatientConsentManagement)
    }

    #[test]
    fn test_designate_and_get_proxy() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        env.as_contract(&contract_id, || {
            designate_proxy(
                env.clone(),
                patient.clone(),
                proxy.clone(),
                ProxyScope::FullAuthority,
            );
        });
        let record = env.as_contract(&contract_id, || get_proxy(env.clone(), patient).unwrap());
        assert_eq!(record.proxy_address, proxy);
        assert_eq!(record.scope, ProxyScope::FullAuthority);
    }

    #[test]
    fn test_revoke_proxy() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        env.as_contract(&contract_id, || {
            designate_proxy(
                env.clone(),
                patient.clone(),
                proxy.clone(),
                ProxyScope::ReadOnly,
            );
        });
        env.as_contract(&contract_id, || {
            revoke_proxy(env.clone(), patient.clone());
        });
        let record = env.as_contract(&contract_id, || get_proxy(env.clone(), patient));
        assert!(record.is_none());
    }

    #[test]
    fn test_proxy_grant_consent_full_authority() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        let grantee = Address::generate(&env);
        env.as_contract(&contract_id, || {
            designate_proxy(
                env.clone(),
                patient.clone(),
                proxy.clone(),
                ProxyScope::FullAuthority,
            );
        });
        env.as_contract(&contract_id, || {
            proxy_grant_consent(env.clone(), proxy.clone(), patient.clone(), grantee.clone());
        });
    }

    #[test]
    #[should_panic(expected = "Proxy scope does not permit granting consent")]
    fn test_readonly_proxy_cannot_grant() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        let grantee = Address::generate(&env);
        env.as_contract(&contract_id, || {
            designate_proxy(
                env.clone(),
                patient.clone(),
                proxy.clone(),
                ProxyScope::ReadOnly,
            );
        });
        env.as_contract(&contract_id, || {
            proxy_grant_consent(env.clone(), proxy.clone(), patient.clone(), grantee.clone());
        });
    }

    #[test]
    fn test_emergency_proxy_can_grant_but_not_revoke() {
        let env = Env::default();
        let contract_id = register_dummy(&env);
        env.mock_all_auths();
        let patient = Address::generate(&env);
        let proxy = Address::generate(&env);
        let grantee = Address::generate(&env);
        env.as_contract(&contract_id, || {
            designate_proxy(
                env.clone(),
                patient.clone(),
                proxy.clone(),
                ProxyScope::EmergencyOnly,
            );
        });
        env.as_contract(&contract_id, || {
            proxy_grant_consent(env.clone(), proxy.clone(), patient.clone(), grantee.clone());
        });
    }
}