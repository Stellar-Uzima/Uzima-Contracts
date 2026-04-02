use crate::types::{AccessLogEntry, DataKey};
use soroban_sdk::{Address, BytesN, Env, Map, String, Vec};

/// Storage operations for health data access logging
pub struct Storage;

impl Storage {
    /// Get or initialize the next log ID
    pub fn get_next_log_id(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::AccessLogCount)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::AccessLogCount, &(count + 1));
        count
    }

    /// Save an access log entry to persistent storage
    pub fn save_access_log(env: &Env, log_entry: &AccessLogEntry) {
        env.storage()
            .persistent()
            .set(&DataKey::AccessLog(log_entry.id), log_entry);
    }

    /// Get an access log entry by ID
    pub fn get_access_log(env: &Env, log_id: u64) -> Option<AccessLogEntry> {
        env.storage().persistent().get(&DataKey::AccessLog(log_id))
    }

    /// Add log ID to patient's access logs index
    pub fn add_log_to_patient_index(env: &Env, patient_id: &Address, log_id: u64) {
        let mut logs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientAccessLogs(patient_id.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        logs.push_back(log_id);
        env.storage()
            .persistent()
            .set(&DataKey::PatientAccessLogs(patient_id.clone()), &logs);

        // Update patient log count
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::PatientLogCount(patient_id.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::PatientLogCount(patient_id.clone()), &(count + 1));
    }

    /// Get all log IDs for a patient
    pub fn get_patient_access_log_ids(env: &Env, patient_id: &Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientAccessLogs(patient_id.clone()))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get the count of logs for a patient
    pub fn get_patient_log_count(env: &Env, patient_id: &Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::PatientLogCount(patient_id.clone()))
            .unwrap_or(0)
    }

    /// Track unique accessors for a patient
    pub fn add_accessor_for_patient(env: &Env, patient_id: &Address, accessor: &Address) {
        let mut accessors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::PatientAccessors(patient_id.clone()))
            .unwrap_or_else(|| Vec::new(&env));

        // Only add if not already present
        if !accessors.iter().any(|a| a == accessor) {
            accessors.push_back(accessor.clone());
            env.storage()
                .persistent()
                .set(&DataKey::PatientAccessors(patient_id.clone()), &accessors);

            // Update unique accessors count
            let count: u32 = env
                .storage()
                .persistent()
                .get(&DataKey::UniqueAccessorsCount(patient_id.clone()))
                .unwrap_or(0);
            env.storage().persistent().set(
                &DataKey::UniqueAccessorsCount(patient_id.clone()),
                &(count + 1),
            );
        }
    }

    /// Get unique accessors for a patient
    pub fn get_patient_accessors(env: &Env, patient_id: &Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::PatientAccessors(patient_id.clone()))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get unique accessors count for a patient
    pub fn get_unique_accessors_count(env: &Env, patient_id: &Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::UniqueAccessorsCount(patient_id.clone()))
            .unwrap_or(0)
    }

    /// Update rolling hash for integrity verification
    pub fn update_rolling_hash(env: &Env, entry_hash: &BytesN<32>) {
        let current_hash: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::RollingHash)
            .unwrap_or_else(|| BytesN::from_array(env, &[0u8; 32]));

        // Combine current and new hash: SHA256(current_hash || entry_hash)
        let mut combined = Vec::<u8>::with_capacity(env, 64);
        combined.append_array(&BytesN::from_array(env, current_hash.as_ref()));
        combined.append_array(&BytesN::from_array(env, entry_hash.as_ref()));

        let new_hash: BytesN<32> = env.crypto().sha256(&combined.into()).into();
        env.storage()
            .persistent()
            .set(&DataKey::RollingHash, &new_hash);
    }

    /// Get the rolling hash for integrity verification
    pub fn get_rolling_hash(env: &Env) -> BytesN<32> {
        env.storage()
            .persistent()
            .get(&DataKey::RollingHash)
            .unwrap_or_else(|| BytesN::from_array(env, &[0u8; 32]))
    }

    /// Get admin address
    pub fn get_admin(env: &Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Admin)
            .expect("Admin not set")
    }

    /// Set admin address
    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().persistent().set(&DataKey::Admin, admin);
    }

    /// Get logging configuration
    pub fn get_config(env: &Env) -> Option<crate::types::LoggingConfig> {
        env.storage().persistent().get(&DataKey::Config)
    }

    /// Set logging configuration
    pub fn set_config(env: &Env, config: &crate::types::LoggingConfig) {
        env.storage().persistent().set(&DataKey::Config, config);
    }

    /// Check if contract is initialized
    pub fn is_initialized(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get::<_, bool>(&DataKey::IsInitialized)
            .unwrap_or(false)
    }

    /// Mark contract as initialized
    pub fn set_initialized(env: &Env) {
        env.storage()
            .persistent()
            .set(&DataKey::IsInitialized, &true);
    }
}
