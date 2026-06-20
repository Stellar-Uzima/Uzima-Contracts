#![no_std]
//! medical_record_hash_registry - Healthcare smart contract on Stellar blockchain.
#![allow(dead_code)]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Vec};

// ==================== Data Types ====================

#[derive(Clone)]
#[contracttype]
pub struct RecordEntry {
    pub patient_id: Address,
    pub record_hash: BytesN<32>,
    pub timestamp: u64,
    pub verified: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientRecords {
    pub records: Vec<RecordEntry>,
    pub record_count: u32,
}

#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    RecordStorage(Address), // patient_id -> PatientRecords
    HashIndex(BytesN<32>),  // record_hash -> patient_id
}

// ==================== Contract ====================

#[contract]
pub struct MedicalRecordHashRegistry;

#[contractimpl]
impl MedicalRecordHashRegistry {
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

    /// Store a medical record hash for a patient
    /// Returns an error if:
    /// - Contract is not initialized
    /// - The same hash already exists for this patient (duplicate detection)
    pub fn store_record(
        env: Env,
        caller: Address,
        patient_id: Address,
        record_hash: BytesN<32>,
    ) -> Result<(), Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let timestamp = env.ledger().timestamp();

        // Get or create patient records
        let mut patient_records: PatientRecords = env
            .storage()
            .persistent()
            .get(&DataKey::RecordStorage(patient_id.clone()))
            .unwrap_or(PatientRecords {
                records: Vec::new(&env),
                record_count: 0,
            });

        // Check for duplicate: scan existing records for this patient
        for record in patient_records.records.iter() {
            if record.record_hash == record_hash {
                events::publish_duplicate_rejected(&env, &patient_id, &record_hash);
                return Err(Error::DuplicateRecord);
            }
        }

        // Create new record entry
        let new_entry = RecordEntry {
            patient_id: patient_id.clone(),
            record_hash: record_hash.clone(),
            timestamp,
            verified: true,
        };

        // Add to patient's records
        patient_records.records.push_back(new_entry);
        patient_records.record_count += 1;

        // Store updated records
        env.storage().persistent().set(
            &DataKey::RecordStorage(patient_id.clone()),
            &patient_records,
        );

        // Store hash index for global lookup
        env.storage()
            .persistent()
            .set(&DataKey::HashIndex(record_hash.clone()), &patient_id);

        events::publish_record_stored(&env, &patient_id, &record_hash, timestamp);
        Ok(())
    }

    /// Verify if a record hash exists and is valid for a patient
    /// Returns true if the record exists and is verified, false otherwise
    pub fn verify_record(
        env: Env,
        patient_id: Address,
        record_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        Self::require_initialized(&env)?;

        let patient_records: Option<PatientRecords> = env
            .storage()
            .persistent()
            .get(&DataKey::RecordStorage(patient_id.clone()));

        match patient_records {
            Some(records) => {
                for record in records.records.iter() {
                    if record.record_hash == record_hash && record.patient_id == patient_id {
                        events::publish_record_verified(&env, &patient_id, &record_hash, true);
                        return Ok(record.verified);
                    }
                }
                events::publish_record_verified(&env, &patient_id, &record_hash, false);
                Ok(false)
            },
            None => {
                events::publish_record_verified(&env, &patient_id, &record_hash, false);
                Ok(false)
            },
        }
    }

    /// Get the patient ID associated with a specific record hash
    pub fn get_patient_by_hash(env: Env, record_hash: BytesN<32>) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::HashIndex(record_hash))
    }

    /// Get all records for a patient
    pub fn get_patient_records(env: Env, patient_id: Address) -> Option<PatientRecords> {
        env.storage()
            .persistent()
            .get(&DataKey::RecordStorage(patient_id))
    }

    /// Get the count of records for a patient
    pub fn get_record_count(env: Env, patient_id: Address) -> u32 {
        env.storage()
            .persistent()
            .get::<_, PatientRecords>(&DataKey::RecordStorage(patient_id))
            .map(|records| records.record_count)
            .unwrap_or(0)
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
}

// ============================================================
// Migratable trait implementation for standardized upgrades
// ============================================================

impl upgradeability::migration::Migratable for MedicalRecordHashRegistry {
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
