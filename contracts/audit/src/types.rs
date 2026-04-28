use soroban_sdk::{contracttype, Address, BytesN, Map, String, Vec};

// ─── Action Types (Issue #399) ────────────────────────────────────────────────

/// Granular action classification required for HIPAA/GDPR compliance logging.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ActionType {
    // Data access events
    DataRead = 0,
    DataWrite = 1,
    DataDelete = 2,
    DataExport = 3,
    // Permission changes
    PermissionGrant = 4,
    PermissionRevoke = 5,
    RoleAssign = 6,
    RoleRevoke = 7,
    // Record modifications
    RecordCreate = 8,
    RecordUpdate = 9,
    RecordArchive = 10,
    RecordRestore = 11,
    // Authentication attempts
    AuthSuccess = 12,
    AuthFailure = 13,
    AuthLogout = 14,
    AuthTokenRefresh = 15,
    // Cross-chain transfers
    CrossChainTransferInitiated = 16,
    CrossChainTransferCompleted = 17,
    CrossChainTransferFailed = 18,
    CrossChainTransferReverted = 19,
    // Compliance-specific
    ConsentGranted = 20,
    ConsentRevoked = 21,
    DataBreach = 22,
    RetentionViolation = 23,
}

/// Outcome of the audited operation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum OperationResult {
    Success = 0,
    Failure = 1,
    Denied = 2,
    Pending = 3,
}

// ─── Legacy AuditType (kept for backward compatibility) ──────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AuditType {
    Event = 0,
    StateChange = 1,
    AdminAction = 2,
    SecurityAlert = 3,
    ComplianceReport = 4,
}

// ─── Core Log Schema (Issue #399) ────────────────────────────────────────────

/// Comprehensive audit log entry satisfying HIPAA/GDPR requirements.
/// Matches the schema specified in issue #399.
#[derive(Clone)]
#[contracttype]
pub struct AuditLog {
    pub id: u64,
    pub timestamp: u64,
    pub actor: Address,
    pub action: ActionType,
    /// SHA-256 hash of the target resource identifier (32 bytes).
    pub target: BytesN<32>,
    pub result: OperationResult,
    /// Flexible key-value metadata (e.g. ip_address, session_id, resource_type).
    pub metadata: Map<String, String>,
}

// ─── Legacy AuditRecord (kept for reference) ─────────────────────────────────
// NOTE: This struct uses Option<BytesN<32>> which has a known upstream
// incompatibility with the soroban-sdk 21 #[contracttype] macro. It is kept
// here for documentation purposes only and is not used by the new API.
//
// #[derive(Clone)]
// #[contracttype]
// pub struct AuditRecord { ... }

// ─── Compliance Structures ────────────────────────────────────────────────────

/// Retention policy enforced on-chain.
#[derive(Clone)]
#[contracttype]
pub struct RetentionPolicy {
    /// Minimum seconds a log must be retained (e.g. 7 years = 220_752_000 s).
    pub min_retention_seconds: u64,
    /// Maximum seconds before a log must be purged (0 = no upper bound).
    pub max_retention_seconds: u64,
}

/// Access control entry for log readers.
#[derive(Clone)]
#[contracttype]
pub struct LogAccessEntry {
    pub reader: Address,
    pub granted_at: u64,
    pub granted_by: Address,
}

/// Export bundle returned by `export_logs`.
#[derive(Clone)]
#[contracttype]
pub struct ExportBundle {
    pub logs: Vec<AuditLog>,
    pub exported_at: u64,
    pub exported_by: Address,
    pub integrity_hash: BytesN<32>,
}

// ─── Summary / Analytics ─────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub struct AuditSummary {
    pub start_time: u64,
    pub end_time: u64,
    pub total_records: u64,
    pub event_count: u32,
    pub admin_action_count: u32,
    pub root_hash: BytesN<32>,
}

// ─── Storage Keys ─────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    // Shared / config
    Admin,
    RecordCount,
    Config,
    RollingHash,
    RetentionPolicy,
    // Legacy AuditRecord index
    Record(u64),
    ContractAudits(Address),
    UserAudits(Address),
    // New AuditLog index
    LogCount,
    Log(u64),
    ActorLogs(Address),
    ActionLogs(u32), // keyed by ActionType discriminant
    // Access control
    LogReader(Address),
    LogReaderList,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditConfig {
    pub archive_threshold: u64,
    pub enabled_types: Vec<AuditType>,
}
