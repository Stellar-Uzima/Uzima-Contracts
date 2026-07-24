//! ZKP Verification Telemetry
//!
//! Types and helpers for tracking ZKP verification metrics across consent
//! and identity flows. Telemetry events are emitted alongside verification
//! results to enable off-chain analytics and compliance reporting.
use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, String};

/// Classification of a telemetry event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum TelemetryEventType {
    /// Proof submitted for verification.
    ProofSubmitted,
    /// Proof passed verification.
    VerificationPassed,
    /// Proof failed verification.
    VerificationFailed,
    /// Batch verification completed.
    BatchVerificationCompleted,
    /// Range proof verified.
    RangeProofVerified,
    /// Credential proof verified.
    CredentialProofVerified,
    /// Recursive proof composed.
    RecursiveProofComposed,
    /// Consent-gated ZKP check performed.
    ConsentZkpCheck,
}

/// A single verification metric snapshot.
#[derive(Clone)]
#[contracttype]
pub struct VerificationMetric {
    /// Proof type that was verified.
    pub proof_type: u32,
    /// Whether verification succeeded.
    pub success: bool,
    /// Gas consumed for this verification.
    pub gas_used: u64,
    /// Timestamp of the metric.
    pub recorded_at: u64,
}

/// A telemetry event emitted during ZKP verification flows.
#[derive(Clone)]
#[contracttype]
pub struct TelemetryEvent {
    /// Unique event identifier (derived from proof_id + event_type).
    pub event_id: BytesN<32>,
    /// Type of telemetry event.
    pub event_type: TelemetryEventType,
    /// Address that triggered the verification.
    pub actor: Address,
    /// Proof identifier this event relates to.
    pub proof_id: BytesN<32>,
    /// Additional context (e.g., circuit_id, consent_id).
    pub context: String,
    /// Timestamp.
    pub timestamp: u64,
    /// Gas used (0 for non-verification events).
    pub gas_used: u64,
}

/// Aggregated ZKP verification telemetry for a time window.
#[derive(Clone)]
#[contracttype]
pub struct ZkpVerificationTelemetry {
    /// Total verifications attempted.
    pub total_attempts: u64,
    /// Total verifications passed.
    pub total_passed: u64,
    /// Total verifications failed.
    pub total_failed: u64,
    /// Total gas consumed across all verifications.
    pub total_gas: u64,
    /// Average gas per verification.
    pub avg_gas: u64,
    /// Number of telemetry events recorded.
    pub event_count: u64,
    /// Timestamp of the first event in this window.
    pub window_start: u64,
    /// Timestamp of the last event in this window.
    pub window_end: u64,
}

/// Storage keys for telemetry data.
#[derive(Clone)]
#[contracttype]
pub enum TelemetryKey {
    /// Total telemetry events counter.
    EventCounter,
    /// Individual telemetry event by ID.
    Event(BytesN<32>),
    /// Aggregated telemetry snapshot.
    AggregatedTelemetry,
    /// Per-user verification metrics.
    UserMetrics(Address),
    /// Per-circuit verification metrics.
    CircuitMetrics(String),
}

/// Emission helper: record a telemetry event and update aggregated metrics.
pub fn emit_telemetry_event(
    env: &Env,
    event_type: TelemetryEventType,
    actor: &Address,
    proof_id: &BytesN<32>,
    context: &String,
    gas_used: u64,
) {
    let timestamp = env.ledger().timestamp();

    // Derive event_id from proof_id + event_type for uniqueness
    let event_id = derive_event_id(env, proof_id, event_type);

    let event = TelemetryEvent {
        event_id: event_id.clone(),
        event_type,
        actor: actor.clone(),
        proof_id: proof_id.clone(),
        context: context.clone(),
        timestamp,
        gas_used,
    };

    // Store the event
    env.storage()
        .persistent()
        .set(&TelemetryKey::Event(event_id.clone()), &event);

    // Increment counter
    let counter: u64 = env
        .storage()
        .persistent()
        .get(&TelemetryKey::EventCounter)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&TelemetryKey::EventCounter, &(counter + 1));

    // Update aggregated metrics
    update_aggregated_metrics(env, event_type, gas_used, timestamp);

    // Emit Soroban event for off-chain indexing
    let type_tag = match event_type {
        TelemetryEventType::ProofSubmitted => symbol_short!("TEL_SUB"),
        TelemetryEventType::VerificationPassed => symbol_short!("TEL_PASS"),
        TelemetryEventType::VerificationFailed => symbol_short!("TEL_FAIL"),
        TelemetryEventType::BatchVerificationCompleted => symbol_short!("TEL_BATCH"),
        TelemetryEventType::RangeProofVerified => symbol_short!("TEL_RNG"),
        TelemetryEventType::CredentialProofVerified => symbol_short!("TEL_CRED"),
        TelemetryEventType::RecursiveProofComposed => symbol_short!("TEL_RECR"),
        TelemetryEventType::ConsentZkpCheck => symbol_short!("TEL_CONS"),
    };

    env.events()
        .publish((symbol_short!("zkp_telemetry"), type_tag), (event_id, proof_id, gas_used));
}

/// Derive a deterministic event ID from proof_id and event_type.
fn derive_event_id(env: &Env, proof_id: &BytesN<32>, event_type: TelemetryEventType) -> BytesN<32> {
    let mut payload = soroban_sdk::Bytes::new(env);
    payload.append(&soroban_sdk::Bytes::from_slice(env, &proof_id.to_array()));
    let type_byte = event_type as u8;
    payload.append(&soroban_sdk::Bytes::from_slice(env, &[type_byte]));
    env.crypto().sha256(&payload).into()
}

/// Update the aggregated telemetry snapshot.
fn update_aggregated_metrics(
    env: &Env,
    event_type: TelemetryEventType,
    gas_used: u64,
    timestamp: u64,
) {
    let mut agg: ZkpVerificationTelemetry = env
        .storage()
        .persistent()
        .get(&TelemetryKey::AggregatedTelemetry)
        .unwrap_or(ZkpVerificationTelemetry {
            total_attempts: 0,
            total_passed: 0,
            total_failed: 0,
            total_gas: 0,
            avg_gas: 0,
            event_count: 0,
            window_start: timestamp,
            window_end: timestamp,
        });

    agg.event_count += 1;
    agg.window_end = timestamp;
    agg.total_gas = agg.total_gas.saturating_add(gas_used);

    match event_type {
        TelemetryEventType::ProofSubmitted | TelemetryEventType::ConsentZkpCheck => {
            agg.total_attempts += 1;
        }
        TelemetryEventType::VerificationPassed
        | TelemetryEventType::RangeProofVerified
        | TelemetryEventType::CredentialProofVerified
        | TelemetryEventType::RecursiveProofComposed
        | TelemetryEventType::BatchVerificationCompleted => {
            agg.total_passed += 1;
        }
        TelemetryEventType::VerificationFailed => {
            agg.total_failed += 1;
        }
    }

    if agg.total_attempts > 0 {
        agg.avg_gas = agg.total_gas / agg.total_attempts;
    }

    env.storage()
        .persistent()
        .set(&TelemetryKey::AggregatedTelemetry, &agg);
}

/// Retrieve aggregated telemetry snapshot.
pub fn get_aggregated_telemetry(env: &Env) -> ZkpVerificationTelemetry {
    env.storage()
        .persistent()
        .get(&TelemetryKey::AggregatedTelemetry)
        .unwrap_or(ZkpVerificationTelemetry {
            total_attempts: 0,
            total_passed: 0,
            total_failed: 0,
            total_gas: 0,
            avg_gas: 0,
            event_count: 0,
            window_start: 0,
            window_end: 0,
        })
}

/// Retrieve a specific telemetry event by ID.
pub fn get_telemetry_event(env: &Env, event_id: &BytesN<32>) -> Option<TelemetryEvent> {
    env.storage()
        .persistent()
        .get(&TelemetryKey::Event(event_id.clone()))
}

/// Record a consent-gated ZKP verification metric.
pub fn record_consent_zkp_metric(
    env: &Env,
    actor: &Address,
    proof_id: &BytesN<32>,
    consent_id: &String,
    success: bool,
    gas_used: u64,
) {
    let event_type = if success {
        TelemetryEventType::ConsentZkpCheck
    } else {
        TelemetryEventType::VerificationFailed
    };
    emit_telemetry_event(env, event_type, actor, proof_id, consent_id, gas_used);
}
