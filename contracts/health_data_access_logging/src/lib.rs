#![no_std]

pub mod types;
pub mod storage;
pub mod queries;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Env, Map, String, Symbol, Vec,
};
use types::{AccessLogEntry, LoggingConfig, DataKey};
use storage::Storage;
use queries::Queries;

#[contract]
pub struct HealthDataAccessLogging;

#[contractimpl]
impl HealthDataAccessLogging {
    /// Initialize the health data access logging contract
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address (usually the contract deployer)
    /// * `config` - Logging configuration
    ///
    /// # Panics
    /// Panics if already initialized
    pub fn initialize(env: Env, admin: Address, config: LoggingConfig) {
        if Storage::is_initialized(&env) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        Storage::set_admin(&env, &admin);
        Storage::set_config(&env, &config);
        Storage::set_initialized(&env);

        env.events().publish(
            (symbol_short!("INIT"), symbol_short!("HDL")),
            &admin,
        );
    }

    /// Log an access to patient health data
    ///
    /// This function records when someone accesses a patient's health data.
    /// Every access is logged with immutable records to create an audit trail.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The address/ID of the patient whose data was accessed
    /// * `accessor_address` - The address of whoever accessed the data
    /// * `access_type` - Type of access (e.g., "read", "write", "export")
    /// * `metadata` - Optional metadata about the access (reason, context, etc.)
    ///
    /// # Returns
    /// The ID of the created access log entry
    ///
    /// # Panics
    /// Panics if contract is not initialized
    pub fn log_access(
        env: Env,
        patient_id: Address,
        accessor_address: Address,
        access_type: String,
        metadata: Map<String, String>,
    ) -> u64 {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        // Require authorization from accessor
        accessor_address.require_auth();

        // Get the next log ID
        let log_id = Storage::get_next_log_id(&env);

        // Create hash of the log entry for integrity
        let log_data = format!(
            "{}:{}:{}:{}:{}",
            log_id,
            patient_id.to_string(),
            accessor_address.to_string(),
            env.ledger().timestamp(),
            access_type.clone()
        );
        let entry_hash: soroban_sdk::BytesN<32> = env.crypto().sha256(log_data.as_bytes()).into();

        // Create the access log entry
        let access_log = AccessLogEntry {
            id: log_id,
            patient_id: patient_id.clone(),
            accessor_address: accessor_address.clone(),
            timestamp: env.ledger().timestamp(),
            access_type: access_type.clone(),
            metadata: metadata.clone(),
            entry_hash: entry_hash.clone(),
        };

        // Save the access log to persistent storage (immutable)
        Storage::save_access_log(&env, &access_log);

        // Add log ID to patient's access log index
        Storage::add_log_to_patient_index(&env, &patient_id, log_id);

        // Track unique accessors
        Storage::add_accessor_for_patient(&env, &patient_id, &accessor_address);

        // Update rolling hash for tamper-evidence
        Storage::update_rolling_hash(&env, &entry_hash);

        // Emit event for off-chain indexing
        env.events().publish(
            (symbol_short!("ACCESS"), symbol_short!("LOG")),
            (
                log_id,
                patient_id.clone(),
                accessor_address.clone(),
                access_type,
                env.ledger().timestamp(),
            ),
        );

        log_id
    }

    /// Retrieve all access logs for a specific patient
    ///
    /// Returns a vector of all access log entries for the specified patient.
    /// Caller must be either the patient themselves or have authorization.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The patient whose logs to retrieve
    ///
    /// # Returns
    /// Vector of AccessLogEntry items
    pub fn get_access_logs(env: Env, patient_id: Address) -> Vec<AccessLogEntry> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        // Require auth from patient to view their own logs
        patient_id.require_auth();

        Queries::get_access_logs(&env, &patient_id)
    }

    /// Retrieve access logs for a patient within a specific time range
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The patient whose logs to retrieve
    /// * `start_timestamp` - Start of time range (inclusive)
    /// * `end_timestamp` - End of time range (inclusive)
    ///
    /// # Returns
    /// Vector of AccessLogEntry items within the range
    pub fn get_access_logs_in_range(
        env: Env,
        patient_id: Address,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> Vec<AccessLogEntry> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        patient_id.require_auth();

        Queries::get_access_logs_in_range(&env, &patient_id, start_timestamp, end_timestamp)
    }

    /// Retrieve access logs by a specific accessor for a patient
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The patient whose logs to retrieve
    /// * `accessor` - Filter logs to only this accessor
    ///
    /// # Returns
    /// Vector of AccessLogEntry items from the specified accessor
    pub fn get_logs_by_accessor(
        env: Env,
        patient_id: Address,
        accessor: Address,
    ) -> Vec<AccessLogEntry> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        patient_id.require_auth();

        Queries::get_logs_by_accessor(&env, &patient_id, &accessor)
    }

    /// Retrieve the most recent N access logs for a patient
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The patient whose logs to retrieve
    /// * `limit` - Maximum number of logs to return
    ///
    /// # Returns
    /// Vector of up to N most recent AccessLogEntry items
    pub fn get_latest_access_logs(
        env: Env,
        patient_id: Address,
        limit: u32,
    ) -> Vec<AccessLogEntry> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        patient_id.require_auth();

        Queries::get_latest_access_logs(&env, &patient_id, limit)
    }

    /// Get summary statistics for a patient's access logs
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The patient whose summary to retrieve
    ///
    /// # Returns
    /// AccessLogSummary with statistics and integrity hash
    pub fn get_access_log_summary(
        env: Env,
        patient_id: Address,
    ) -> types::AccessLogSummary {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        patient_id.require_auth();

        Queries::get_access_log_summary(&env, &patient_id)
    }

    /// Get the count of unique accessors for a patient
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The patient to query
    ///
    /// # Returns
    /// Number of unique addresses that have accessed this patient's data
    pub fn get_unique_accessors_count(env: Env, patient_id: Address) -> u32 {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        patient_id.require_auth();

        Queries::get_unique_accessors_count(&env, &patient_id)
    }

    /// Get all unique accessors for a patient
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `patient_id` - The patient to query
    ///
    /// # Returns
    /// Vector of all unique accessor addresses
    pub fn get_unique_accessors(env: Env, patient_id: Address) -> Vec<Address> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        patient_id.require_auth();

        Queries::get_unique_accessors(&env, &patient_id)
    }

    /// Verify the integrity of the access logs using the rolling hash
    ///
    /// Returns the current rolling hash which can be compared against
    /// expected values to detect tampering.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    ///
    /// # Returns
    /// The rolling hash of all access logs
    pub fn verify_logs_integrity(env: Env) -> soroban_sdk::BytesN<32> {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Queries::verify_logs_integrity(&env)
    }

    /// Update the logging configuration (admin only)
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `config` - New logging configuration
    pub fn update_config(env: Env, config: LoggingConfig) {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        let admin = Storage::get_admin(&env);
        admin.require_auth();

        Storage::set_config(&env, &config);

        env.events().publish(
            (symbol_short!("CONFIG"), symbol_short!("UPDATE")),
            config,
        );
    }

    /// Get the current logging configuration
    ///
    /// # Returns
    /// The current LoggingConfig
    pub fn get_config(env: Env) -> LoggingConfig {
        if !Storage::is_initialized(&env) {
            panic!("Contract not initialized");
        }

        Storage::get_config(&env).expect("Config not set")
    }
}
