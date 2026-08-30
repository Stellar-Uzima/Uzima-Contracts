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
//! ## Trace IDs
//!
//! Every event carries a `trace_id` (BytesN<32>) that is invariant across a
//! single transaction's cross-contract call chain. Off-chain systems group
//! events by `trace_id` to reconstruct a full execution trace.
//!
//! `trace_id` is derived from an invariant of the top-level submission — the
//! caller of the top-level invocation plus the current ledger sequence — and
//! is **forwarded** (not recomputed) by every downstream contract in the chain.
//! This is what makes it constant across a chain where each hop observes a
//! different direct caller.
//!
//! The legacy `correlation_id` field is retained only for backward
//! compatibility. It is derived per-contract from the direct caller and the
//! ledger timestamp and is **not** a reliable trace key; consumers must use
//! `trace_id` instead.

use soroban_sdk::{contracttype, symbol_short, Address, Bytes, BytesN, Env, String, Symbol};

// ==================== Schema Constants ====================

/// Current telemetry schema version.
///
/// v2.0.0 (MAJOR bump from 1.0.0): `TelemetryEvent` gained a required
/// `trace_id` field, changing the packed event payload. All consumers must be
/// updated to emit and parse the new field.
pub const SCHEMA_VERSION_MAJOR: u32 = 2;
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
    /// Legacy per-contract correlation ID. Retained for backward compatibility;
    /// **not** a reliable trace key.
    pub correlation_id: BytesN<32>,
    /// Trace ID invariant across a single transaction's cross-contract call
    /// chain. Off-chain systems should group events by this field.
    pub trace_id: BytesN<32>,
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

/// Generate a per-contract correlation ID from the caller address and ledger
/// timestamp.
///
/// **Legacy.** This is derived from the *direct* caller, which differs at every
/// hop of a cross-contract call chain, so it cannot be used to reconstruct a
/// trace. It is retained only for backward compatibility. Use
/// [`derive_trace_id`] for trace reconstruction.
pub fn derive_correlation_id(env: &Env, caller_bytes: &soroban_sdk::Bytes) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    data.extend_from_slice(caller_bytes);
    let timestamp = env.ledger().timestamp().to_be_bytes();
    data.extend_from_slice(&timestamp);

    // Hash to get a fixed-size correlation ID
    let hash = env.crypto().sha256(&data);
    hash
}

/// Derive a `trace_id` that is invariant across a cross-contract call chain.
///
/// The value is a pure function of the **top-level caller** and the **current
/// ledger sequence**:
///
/// ```text
/// trace_id = sha256(top_level_caller_address || ledger_sequence)
/// ```
///
/// Why these inputs:
///
/// - **Top-level caller** is the one invariant piece of a chain that the
///   emitter can observe. The direct caller differs at every hop (contract A's
///   caller is the end user, contract B's caller is A), so an ID derived from
///   the direct caller cannot merge a chain. By deriving from the top-level
///   caller — and **forwarding** that value down the chain rather than
///   recomputing it per hop — every contract in one logical transaction emits
///   an identical `trace_id`.
/// - **Ledger sequence** distinguishes submissions in different ledgers, so two
///   transactions initiated by the same top-level caller in different ledgers
///   do not collide. (A Soroban `no_std` contract has no transaction-hash
///   primitive on `Env`, so this is the strongest ledger-invariant available.)
///
/// A top-level contract computes this once (from the address that authorized
/// the submission) and passes the result to [`crate::ContractMonitoring::record_call`] /
/// [`crate::ContractMonitoring::record_error`]; every downstream contract
/// reuses the value it was passed instead of re-deriving it.
pub fn derive_trace_id(env: &Env, top_level_caller: &Address) -> BytesN<32> {
    let caller_bytes: Bytes = top_level_caller.to_buffer().into();
    let mut data = Bytes::new(env);
    data.extend_from_slice(&caller_bytes);
    let ledger_sequence = env.ledger().sequence().to_be_bytes();
    data.extend_from_slice(&ledger_sequence);

    env.crypto().sha256(&data)
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
    trace_id: BytesN<32>,
) -> TelemetryEvent {
    let class = event_type.class();
    TelemetryEvent {
        schema_version: current_schema_version(),
        correlation_id,
        trace_id,
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
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn test_schema_version_packing() {
        assert_eq!(pack_schema_version(1, 0, 0), 10_000);
        assert_eq!(pack_schema_version(1, 2, 3), 10_203);
        assert_eq!(pack_schema_version(2, 0, 0), 20_000);
    }

    #[test]
    fn test_current_schema_version() {
        assert_eq!(current_schema_version(), 20_000);
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

    #[test]
    fn test_derive_trace_id_differs_across_callers() {
        // Two events raised by *different* callers must not share a `trace_id`,
        // fixing the false-collision the legacy `correlation_id` allowed.
        let env = Env::default();
        let caller_a = Address::generate(&env);
        let caller_b = Address::generate(&env);

        let trace_a = derive_trace_id(&env, &caller_a);
        let trace_b = derive_trace_id(&env, &caller_b);

        assert_ne!(trace_a, trace_b);
    }

    #[test]
    fn test_derive_trace_id_stable_for_one_logical_transaction() {
        // The same top-level caller derived at the same ledger sequence yields
        // an identical `trace_id`. This is the invariance that lets a caller
        // and callee in one logical transaction emit the same trace id when the
        // top-level value is forwarded down the chain.
        let env = Env::default();
        let top_level_caller = Address::generate(&env);

        let first = derive_trace_id(&env, &top_level_caller);
        let second = derive_trace_id(&env, &top_level_caller);

        assert_eq!(first, second);
    }

    #[test]
    fn test_derive_trace_id_separates_ledgers() {
        // Advancing the ledger sequence changes the trace id for the same
        // caller, so submissions in different ledgers do not collide.
        let env = Env::default();
        let caller = Address::generate(&env);
        let before = derive_trace_id(&env, &caller);

        env.ledger().set_sequence_number(env.ledger().sequence() + 1);
        let after = derive_trace_id(&env, &caller);

        assert_ne!(before, after);
    }
}

#![no_std]
use soroban_sdk::{contracterror, contracttype, Env, Symbol, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum TelemetryError {
    InvalidCPUInstructions = 1,
    InvalidMemoryBytes = 2,
    MaxRetryThresholdExceeded = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Retried,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionTelemetry {
    pub contract_id: BytesN<32>,
    pub function_name: Symbol,
    pub cpu_instructions: u64,
    pub memory_bytes: u64,
    pub status: ExecutionStatus,
    pub retry_count: u32,
    pub error_code: u32,
    pub timestamp: u64,
}

pub struct TelemetryLogger;

impl TelemetryLogger {
    /// Logs structured telemetry events to Soroban contract topics for off-chain ingestion
    pub fn emit_execution_telemetry(
        env: &Env,
        contract_id: BytesN<32>,
        function_name: Symbol,
        cpu_instructions: u64,
        memory_bytes: u64,
        status: ExecutionStatus,
        retry_count: u32,
        error_code: u32,
    ) {
        let timestamp = env.ledger().timestamp();

        let telemetry = ExecutionTelemetry {
            contract_id: contract_id.clone(),
            function_name: function_name.clone(),
            cpu_instructions,
            memory_bytes,
            status: status.clone(),
            retry_count,
            error_code,
            timestamp,
        };

        // Topic 1: Symbol "telemetry"
        // Topic 2: Target Function Name
        // Topic 3: Execution Status
        env.events().publish(
            (
                Symbol::new(env, "telemetry"),
                function_name,
                status,
            ),
            telemetry,
        );
    }
}