use soroban_sdk::{contracttype, Address, BytesN, Map, String, Vec};

/// Represents a single access log entry for patient data
#[derive(Clone)]
#[contracttype]
pub struct AccessLogEntry {
    /// Unique identifier for this log entry
    pub id: u64,
    /// Patient whose data was accessed
    pub patient_id: Address,
    /// Address of the accessor (who accessed the data)
    pub accessor_address: Address,
    /// Timestamp of the access
    pub timestamp: u64,
    /// Type of access (e.g., "read", "write", "export")
    pub access_type: String,
    /// Optional metadata about the access (reason, context, etc.)
    pub metadata: Map<String, String>,
    /// Hash of the access log entry for integrity verification
    pub entry_hash: BytesN<32>,
}

/// Configuration for the health data access logging contract
#[derive(Clone)]
#[contracttype]
pub struct LoggingConfig {
    /// Maximum number of logs to store per patient
    pub max_logs_per_patient: u32,
    /// Whether to allow log queries by non-admins
    pub allow_public_queries: bool,
    /// Retention period in seconds (0 = no automatic deletion)
    pub retention_period: u64,
}

/// Summary of access logs for a patient
#[derive(Clone)]
#[contracttype]
pub struct AccessLogSummary {
    /// Patient ID
    pub patient_id: Address,
    /// Total number of access logs
    pub total_accesses: u64,
    /// Timestamp of the first access
    pub first_access_timestamp: u64,
    /// Timestamp of the most recent access
    pub last_access_timestamp: u64,
    /// Number of unique accessors
    pub unique_accessors_count: u32,
    /// Hash for integrity verification
    pub summary_hash: BytesN<32>,
}

/// Storage keys for the contract
#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Global logging configuration
    Config,
    /// Total count of all access logs
    AccessLogCount,
    /// Individual access log entry (AccessLogEntry)
    AccessLog(u64),
    /// Index of all logs for a specific patient (Vec<u64> of log IDs)
    PatientAccessLogs(Address),
    /// Count of logs for a specific patient
    PatientLogCount(Address),
    /// Count of unique accessors for a patient
    UniqueAccessorsCount(Address),
    /// Set of unique accessors for a patient
    PatientAccessors(Address),
    /// Rolling hash for integrity verification
    RollingHash,
    /// Initialization flag
    IsInitialized,
}
