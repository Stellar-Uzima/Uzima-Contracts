use crate::storage::Storage;
use crate::types::{AccessLogEntry, AccessLogSummary, DataKey};
use soroban_sdk::{Address, BytesN, Env, Vec};

/// Query operations for health data access logging
pub struct Queries;

impl Queries {
    /// Retrieve all access logs for a specific patient
    pub fn get_access_logs(env: &Env, patient_id: &Address) -> Vec<AccessLogEntry> {
        let log_ids = Storage::get_patient_access_log_ids(env, patient_id);
        let mut logs = Vec::new(env);

        for log_id in log_ids.iter() {
            if let Some(log) = Storage::get_access_log(env, log_id) {
                logs.push_back(log);
            }
        }

        logs
    }

    /// Retrieve access logs for a patient within a time range
    pub fn get_access_logs_in_range(
        env: &Env,
        patient_id: &Address,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> Vec<AccessLogEntry> {
        let logs = Self::get_access_logs(env, patient_id);
        let mut filtered = Vec::new(env);

        for log in logs.iter() {
            if log.timestamp >= start_timestamp && log.timestamp <= end_timestamp {
                filtered.push_back(log);
            }
        }

        filtered
    }

    /// Retrieve access logs by a specific accessor for a patient
    pub fn get_logs_by_accessor(
        env: &Env,
        patient_id: &Address,
        accessor: &Address,
    ) -> Vec<AccessLogEntry> {
        let logs = Self::get_access_logs(env, patient_id);
        let mut filtered = Vec::new(env);

        for log in logs.iter() {
            if log.accessor_address == *accessor {
                filtered.push_back(log);
            }
        }

        filtered
    }

    /// Retrieve the most recent N access logs for a patient
    pub fn get_latest_access_logs(
        env: &Env,
        patient_id: &Address,
        limit: u32,
    ) -> Vec<AccessLogEntry> {
        let log_ids = Storage::get_patient_access_log_ids(env, patient_id);
        let mut logs = Vec::new(env);

        // Iterate from the end (most recent) up to limit
        let start_idx = if log_ids.len() > limit as usize {
            log_ids.len() - limit as usize
        } else {
            0
        };

        for i in (start_idx..log_ids.len()).rev() {
            if let Some(log) = Storage::get_access_log(env, log_ids.get_unchecked(i as u32)) {
                logs.push_back(log);
            }
        }

        logs
    }

    /// Get summary statistics for a patient's access logs
    pub fn get_access_log_summary(env: &Env, patient_id: &Address) -> AccessLogSummary {
        let logs = Self::get_access_logs(env, patient_id);
        let unique_accessors_count = Storage::get_unique_accessors_count(env, patient_id);
        let total_accesses = Storage::get_patient_log_count(env, patient_id) as u64;

        let (first_timestamp, last_timestamp) = if !logs.is_empty() {
            let first = logs.get_unchecked(0).timestamp;
            let last = logs.get_unchecked((logs.len() - 1) as u32).timestamp;
            (first, last)
        } else {
            (0, 0)
        };

        // Create summary hash
        let mut summary_data = patient_id.to_string();
        summary_data.append(&soroban_sdk::String::from_str(env, ":"));
        summary_data.append(&soroban_sdk::String::from_str(env, &total_accesses.to_string()));
        summary_data.append(&soroban_sdk::String::from_str(env, ":"));
        summary_data.append(&soroban_sdk::String::from_str(env, &first_timestamp.to_string()));
        summary_data.append(&soroban_sdk::String::from_str(env, ":"));
        summary_data.append(&soroban_sdk::String::from_str(env, &last_timestamp.to_string()));
        summary_data.append(&soroban_sdk::String::from_str(env, ":"));
        summary_data.append(&soroban_sdk::String::from_str(env, &unique_accessors_count.to_string()));
        let summary_hash: BytesN<32> = env.crypto().sha256(&summary_data.to_bytes()).into();

        AccessLogSummary {
            patient_id: patient_id.clone(),
            total_accesses,
            first_access_timestamp: first_timestamp,
            last_access_timestamp: last_timestamp,
            unique_accessors_count,
            summary_hash,
        }
    }

    /// Verify the integrity of all logs using rolling hash
    pub fn verify_logs_integrity(env: &Env) -> BytesN<32> {
        Storage::get_rolling_hash(env)
    }

    /// Get the count of accessors for a patient
    pub fn get_unique_accessors_count(env: &Env, patient_id: &Address) -> u32 {
        Storage::get_unique_accessors_count(env, patient_id)
    }

    /// Get all unique accessors for a patient
    pub fn get_unique_accessors(env: &Env, patient_id: &Address) -> Vec<Address> {
        Storage::get_patient_accessors(env, patient_id)
    }
}