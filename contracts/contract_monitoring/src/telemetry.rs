//! Versioned telemetry event schema for the Uzima monitoring contract.
//!
//! This module defines a structured, versioned event format for contract
//! observability and incident response. Each event carries:
//!
//! - A **schema version** for forward/backward compatibility
//! - A **correlation ID** to link events across contracts and transactions
//! - **Contract metadata** (name, version) for context
//! - An **event classification** (operational vs security)
//! - A **severity level** to distinguish informational from critical events
//!
//! ## Schema Versioning
//!
//! The schema version follows semver: `MAJOR.MINOR.PATCH`.
//! - **MAJOR**: Breaking changes to event fields (all consumers must update)
//! - **MINOR**: Backward-compatible additions (new optional fields)
//! - **PATCH**: Bug fixes and documentation
//!
//! ## Event Classification
//!
//! Events are classified as either:
//! - **Operational**: Routine business events (calls, completions, state changes)
//! - **Security**: Events requiring attention (auth failures, threshold breaches)
//!
//! ## Correlation IDs
//!
//! Every event includes a `correlation_id` (BytesN<32>) that links related
//! events across a single transaction or cross-contract call chain. This
//! enables off-chain systems to reconstruct the full execution trace.

use soroban_sdk::{contracttype, symbol_short, BytesN, Env, String, Symbol};

// ==================== Schema Constants ====================

/// Current telemetry schema version.
pub const SCHEMA_VERSION_MAJOR: u32 = 1;
pub const SCHEMA_VERSION_MINOR: u32 = 0;
pub const SCHEMA_VERSION_PATCH: u32 = 0;

/// Symbol constant for telemetry events.
pub const TELEMETRY_TOPIC: Symbol = symbol_short!("TEL");

// ==================== Event Classification ====================

/// Classifies a telemetry event as operational or security-relevant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum EventClass {
    /// Routine operational events (function calls, completions, metrics).
    Operational = 0,
    /// Security-relevant events (auth failures, threshold breaches, anomalies).
    Security = 1,
}

// ==================== Severity Levels ====================

/// Severity level for telemetry events, aligned with standard syslog levels.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum TelemetrySeverity {
    /// Informational: normal operation.
    Info = 0,
    /// Warning: something unexpected but non-critical.
    Warn = 1,
    /// Error: operation failed.
    Error = 2,
    /// Critical: system-level failure requiring immediate attention.
    Critical = 3,
}

// ==================== Event Types ====================

/// The specific type of telemetry event being recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum TelemetryEventType {
    // Operational events (class = Operational)
    /// A contract function was invoked.
    FunctionInvoked = 0,
    /// A contract function completed successfully.
    FunctionCompleted = 1,
    /// A state transition occurred.
    StateTransition = 2,
    /// A metric was updated.
    MetricUpdated = 3,

    // Security events (class = Security)
    /// An authentication check failed.
    AuthFailure = 10,
    /// An authorization check failed.
    AuthorizationFailure = 11,
    /// A threshold was breached.
    ThresholdBreached = 12,
    /// An anomalous pattern was detected.
    AnomalyDetected = 13,
    /// A configuration change was attempted.
    ConfigChange = 14,
}

impl TelemetryEventType {
    /// Returns the event class for this event type.
    pub fn class(&self) -> EventClass {
        match self {
            Self::FunctionInvoked
            | Self::FunctionCompleted
            | Self::StateTransition
            | Self::MetricUpdated => EventClass::Operational,
            Self::AuthFailure
            | Self::AuthorizationFailure
            | Self::ThresholdBreached
            | Self::AnomalyDetected
            | Self::ConfigChange => EventClass::Security,
        }
    }
}

// ==================== Structured Event ====================

/// A single structured telemetry event with full context.
///
/// This is the canonical format for all on-chain observability events.
/// Off-chain consumers should parse events matching this structure.
#[derive(Clone)]
#[contracttype]
pub struct TelemetryEvent {
    /// Schema version for this event (`MAJOR.MINOR.PATCH`).
    pub schema_version: u32,
    /// Unique correlation ID linking related events in a transaction chain.
    pub correlation_id: BytesN<32>,
    /// The contract that emitted this event.
    pub contract_name: String,
    /// Version of the emitting contract.
    pub contract_version: String,
    /// The specific type of event.
    pub event_type: TelemetryEventType,
    /// Operational or security classification.
    pub event_class: EventClass,
    /// Severity level.
    pub severity: TelemetrySeverity,
    /// Name of the function or operation.
    pub operation: String,
    /// Additional context as a key-value pair.
    pub detail: String,
    /// Ledger timestamp.
    pub timestamp: u64,
}

/// Summary snapshot of recent telemetry for dashboard consumption.
#[derive(Clone)]
#[contracttype]
pub struct TelemetrySnapshot {
    /// Schema version.
    pub schema_version: u32,
    /// Total events recorded.
    pub total_events: u64,
    /// Total operational events.
    pub operational_count: u64,
    /// Total security events.
    pub security_count: u64,
    /// Total errors.
    pub error_count: u64,
    /// Total critical events.
    pub critical_count: u64,
    /// Ledger timestamp of the snapshot.
    pub snapshot_at: u64,
}

// ==================== Helper Functions ====================

/// Pack the schema version into a single u32 for storage: `MAJOR * 10000 + MINOR * 100 + PATCH`.
pub fn pack_schema_version(major: u32, minor: u32, patch: u32) -> u32 {
    major * 10_000 + minor * 100 + patch
}

/// Get the current schema version as a packed u32.
pub fn current_schema_version() -> u32 {
    pack_schema_version(
        SCHEMA_VERSION_MAJOR,
        SCHEMA_VERSION_MINOR,
        SCHEMA_VERSION_PATCH,
    )
}

/// Generate a correlation ID from the caller address and timestamp.
/// This is a simple deterministic derivation; more sophisticated systems
/// may use transaction hashes.
pub fn derive_correlation_id(env: &Env, caller_bytes: &soroban_sdk::Bytes) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    data.extend_from_slice(caller_bytes);
    let timestamp = env.ledger().timestamp().to_be_bytes();
    data.extend_from_slice(&timestamp);

    // Hash to get a fixed-size correlation ID
    let hash = env.crypto().sha256(&data);
    hash
}

/// Build a TelemetryEvent with all fields populated.
pub fn build_event(
    env: &Env,
    contract_name: &str,
    contract_version: &str,
    event_type: TelemetryEventType,
    severity: TelemetrySeverity,
    operation: &str,
    detail: &str,
    correlation_id: BytesN<32>,
) -> TelemetryEvent {
    let class = event_type.class();
    TelemetryEvent {
        schema_version: current_schema_version(),
        correlation_id,
        contract_name: String::from_str(env, contract_name),
        contract_version: String::from_str(env, contract_version),
        event_type,
        event_class: class,
        severity,
        operation: String::from_str(env, operation),
        detail: String::from_str(env, detail),
        timestamp: env.ledger().timestamp(),
    }
}

/// Emit a telemetry event with the standard topic structure.
///
/// Events are emitted with topic `(TEL, <event_type_symbol>)` and the
/// full `TelemetryEvent` as the data payload.
pub fn emit_telemetry_event(env: &Env, event: &TelemetryEvent) {
    let type_symbol = match event.event_type {
        TelemetryEventType::FunctionInvoked => symbol_short!("FN_INVOKE"),
        TelemetryEventType::FunctionCompleted => symbol_short!("FN_DONE"),
        TelemetryEventType::StateTransition => symbol_short!("STATE"),
        TelemetryEventType::MetricUpdated => symbol_short!("METRIC"),
        TelemetryEventType::AuthFailure => symbol_short!("AUTH_FAIL"),
        TelemetryEventType::AuthorizationFailure => symbol_short!("AUTHZ_FAIL"),
        TelemetryEventType::ThresholdBreached => symbol_short!("THRESHOLD"),
        TelemetryEventType::AnomalyDetected => symbol_short!("ANOMALY"),
        TelemetryEventType::ConfigChange => symbol_short!("CFG_CHG"),
    };

    env.events()
        .publish((TELEMETRY_TOPIC, type_symbol), event);
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_packing() {
        assert_eq!(pack_schema_version(1, 0, 0), 10_000);
        assert_eq!(pack_schema_version(1, 2, 3), 10_203);
        assert_eq!(pack_schema_version(2, 0, 0), 20_000);
    }

    #[test]
    fn test_current_schema_version() {
        assert_eq!(current_schema_version(), 10_000);
    }

    #[test]
    fn test_event_type_classification() {
        assert_eq!(
            TelemetryEventType::FunctionInvoked.class(),
            EventClass::Operational
        );
        assert_eq!(
            TelemetryEventType::AuthFailure.class(),
            EventClass::Security
        );
        assert_eq!(
            TelemetryEventType::ThresholdBreached.class(),
            EventClass::Security
        );
        assert_eq!(
            TelemetryEventType::StateTransition.class(),
            EventClass::Operational
        );
    }

    #[test]
    fn test_severity_ordering() {
        assert!(TelemetrySeverity::Info < TelemetrySeverity::Warn);
        assert!(TelemetrySeverity::Warn < TelemetrySeverity::Error);
        assert!(TelemetrySeverity::Error < TelemetrySeverity::Critical);
    }

    #[test]
    fn test_event_class_ordering() {
        assert!(EventClass::Operational < EventClass::Security);
    }
}
