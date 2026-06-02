//! # Healthcare Payment Events Module
//!
//! Comprehensive event emission for healthcare payment operations.
//! Tracks claims, pre-authorizations, payments, and disputes.

use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

// ── Event Type Definitions ─────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    /// Contract initialized
    Initialized,
    /// Claim submitted
    ClaimSubmitted,
    /// Claim status changed
    ClaimStatusChanged,
    /// Pre-authorization requested
    PreAuthRequested,
    /// Pre-authorization status changed
    PreAuthStatusChanged,
    /// Payment processed
    PaymentProcessed,
    /// Payment plan created
    PaymentPlanCreated,
    /// Payment plan updated
    PaymentPlanUpdated,
    /// Dispute filed
    DisputeFiled,
    /// Dispute resolved
    DisputeResolved,
    /// Circuit breaker triggered
    CircuitBreakerTriggered,
    /// AML review triggered
    AMLReviewTriggered,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    RecordOperations,
    Administrative,
    System,
}

// ── Event Data Structures ──────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub struct HealthcarePaymentEventData {
    /// Claim ID
    pub claim_id: Option<u64>,
    /// Pre-auth ID
    pub preauth_id: Option<u64>,
    /// Patient address
    pub patient: Option<Address>,
    /// Provider address
    pub provider: Option<Address>,
    /// Claim/Payment amount
    pub amount: i128,
    /// Current status
    pub status: Option<String>,
    /// Previous status
    pub previous_status: Option<String>,
    /// Service ID (claim service type)
    pub service_id: Option<String>,
    /// Policy ID
    pub policy_id: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcarePaymentEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcarePaymentEventData,
}

// ── Event Emission Functions ───────────────────────────────────────────────

/// Emit Initialized event
pub fn emit_initialized(env: &Env, admin: Address) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: admin.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: None,
            preauth_id: None,
            patient: Some(admin),
            provider: None,
            amount: 0,
            status: Some("initialized".into()),
            previous_status: None,
            service_id: None,
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("INIT")), event);
}

/// Emit ClaimSubmitted event
pub fn emit_claim_submitted(
    env: &Env,
    caller: Address,
    claim_id: u64,
    patient: Address,
    provider: Address,
    amount: i128,
    service_id: String,
    policy_id: String,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::ClaimSubmitted,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: Some(claim_id),
            preauth_id: None,
            patient: Some(patient),
            provider: Some(provider),
            amount,
            status: Some("submitted".into()),
            previous_status: None,
            service_id: Some(service_id),
            policy_id: Some(policy_id),
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("CLAIM")), event);
}

/// Emit ClaimStatusChanged event
pub fn emit_claim_status_changed(
    env: &Env,
    caller: Address,
    claim_id: u64,
    previous_status: String,
    new_status: String,
    patient: Address,
    provider: Address,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::ClaimStatusChanged,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: Some(claim_id),
            preauth_id: None,
            patient: Some(patient),
            provider: Some(provider),
            amount: 0,
            status: Some(new_status),
            previous_status: Some(previous_status),
            service_id: None,
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("CSTAT")), event);
}

/// Emit PreAuthRequested event
pub fn emit_preauth_requested(
    env: &Env,
    caller: Address,
    preauth_id: u64,
    patient: Address,
    provider: Address,
    estimated_cost: i128,
    service_id: String,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::PreAuthRequested,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: None,
            preauth_id: Some(preauth_id),
            patient: Some(patient),
            provider: Some(provider),
            amount: estimated_cost,
            status: Some("pending".into()),
            previous_status: None,
            service_id: Some(service_id),
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("PREAUTH")), event);
}

/// Emit PreAuthStatusChanged event
pub fn emit_preauth_status_changed(
    env: &Env,
    caller: Address,
    preauth_id: u64,
    previous_status: String,
    new_status: String,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::PreAuthStatusChanged,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: None,
            preauth_id: Some(preauth_id),
            patient: None,
            provider: None,
            amount: 0,
            status: Some(new_status),
            previous_status: Some(previous_status),
            service_id: None,
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("PSTAT")), event);
}

/// Emit PaymentProcessed event
pub fn emit_payment_processed(
    env: &Env,
    caller: Address,
    claim_id: u64,
    amount: i128,
    payee: Address,
    payment_ref: String,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::PaymentProcessed,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: Some(claim_id),
            preauth_id: None,
            patient: None,
            provider: Some(payee),
            amount,
            status: Some("paid".into()),
            previous_status: None,
            service_id: Some(payment_ref),
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("PAID")), event);
}

/// Emit PaymentPlanCreated event
pub fn emit_payment_plan_created(
    env: &Env,
    caller: Address,
    plan_id: u64,
    patient: Address,
    provider: Address,
    total_amount: i128,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::PaymentPlanCreated,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: Some(plan_id),
            preauth_id: None,
            patient: Some(patient),
            provider: Some(provider),
            amount: total_amount,
            status: Some("active".into()),
            previous_status: None,
            service_id: None,
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("PLAN")), event);
}

/// Emit DisputeFiled event
pub fn emit_dispute_filed(
    env: &Env,
    caller: Address,
    claim_id: u64,
    reason: Option<String>,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::DisputeFiled,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: Some(claim_id),
            preauth_id: None,
            patient: None,
            provider: None,
            amount: 0,
            status: Some("disputed".into()),
            previous_status: None,
            service_id: reason,
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("DISP")), event);
}

/// Emit CircuitBreakerTriggered event
pub fn emit_circuit_breaker_triggered(env: &Env, caller: Address) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::CircuitBreakerTriggered,
        category: OperationCategory::System,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: None,
            preauth_id: None,
            patient: None,
            provider: None,
            amount: 0,
            status: Some("circuit_open".into()),
            previous_status: None,
            service_id: None,
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("CIRC")), event);
}

/// Emit AMLReviewTriggered event
pub fn emit_aml_review_triggered(
    env: &Env,
    caller: Address,
    claim_id: u64,
    amount: i128,
) {
    let event = HealthcarePaymentEvent {
        event_type: EventType::AMLReviewTriggered,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: HealthcarePaymentEventData {
            claim_id: Some(claim_id),
            preauth_id: None,
            patient: None,
            provider: None,
            amount,
            status: Some("aml_review".into()),
            previous_status: None,
            service_id: None,
            policy_id: None,
        },
    };
    env.events()
        .publish((symbol_short!("HCP"), symbol_short!("AML")), event);
}
