#![no_std]
#![allow(dead_code)]

#[cfg(test)]
mod test;

mod errors;
mod events;

pub use errors::Error;

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, Vec,
};

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
            }
            None => {
                events::publish_record_verified(&env, &patient_id, &record_hash, false);
                Ok(false)
            }
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
