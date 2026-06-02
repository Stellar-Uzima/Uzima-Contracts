//! # Escrow Contract Events Module
//!
//! Comprehensive event emission for escrow operations.
//! Critical for tracking payment states and audit trails.

use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

// ── Event Type Definitions ─────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    /// Contract initialized
    Initialized,
    /// Escrow created
    EscrowCreated,
    /// Escrow status changed
    EscrowStatusChanged,
    /// Funds deposited into escrow
    FundsDeposited,
    /// Escrow settled (funds released to payee)
    EscrowSettled,
    /// Escrow refunded (funds returned to payer)
    EscrowRefunded,
    /// Dispute raised
    DisputeRaised,
    /// Dispute resolved
    DisputeResolved,
    /// Escrow approval added
    ApprovalAdded,
    /// Fee charged
    FeeCharged,
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
pub struct EscrowEventData {
    /// Order ID
    pub order_id: u64,
    /// Payer address
    pub payer: Option<Address>,
    /// Payee address
    pub payee: Option<Address>,
    /// Amount involved
    pub amount: i128,
    /// Token address
    pub token: Option<Address>,
    /// Current status
    pub status: Option<String>,
    /// Previous status
    pub previous_status: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct EscrowEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: EscrowEventData,
}

// ── Event Emission Functions ───────────────────────────────────────────────

/// Emit Initialized event
pub fn emit_initialized(env: &Env, admin: Address) {
    let event = EscrowEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: admin.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id: 0,
            payer: Some(admin),
            payee: None,
            amount: 0,
            token: None,
            status: Some("initialized".into()),
            previous_status: None,
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("INIT")), event);
}

/// Emit EscrowCreated event
pub fn emit_escrow_created(
    env: &Env,
    caller: Address,
    order_id: u64,
    payer: Address,
    payee: Address,
    amount: i128,
    token: Address,
) {
    let event = EscrowEvent {
        event_type: EventType::EscrowCreated,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id,
            payer: Some(payer),
            payee: Some(payee),
            amount,
            token: Some(token),
            status: Some("pending".into()),
            previous_status: None,
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("CREATE")), event);
}

/// Emit EscrowStatusChanged event
pub fn emit_escrow_status_changed(
    env: &Env,
    caller: Address,
    order_id: u64,
    previous_status: String,
    new_status: String,
    payer: Address,
    payee: Address,
) {
    let event = EscrowEvent {
        event_type: EventType::EscrowStatusChanged,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id,
            payer: Some(payer),
            payee: Some(payee),
            amount: 0,
            token: None,
            status: Some(new_status),
            previous_status: Some(previous_status),
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("STATUS")), event);
}

/// Emit FundsDeposited event
pub fn emit_funds_deposited(
    env: &Env,
    caller: Address,
    order_id: u64,
    amount: i128,
    token: Address,
) {
    let event = EscrowEvent {
        event_type: EventType::FundsDeposited,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id,
            payer: Some(caller),
            payee: None,
            amount,
            token: Some(token),
            status: Some("funded".into()),
            previous_status: None,
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("DEPOSIT")), event);
}

/// Emit EscrowSettled event
pub fn emit_escrow_settled(
    env: &Env,
    caller: Address,
    order_id: u64,
    amount: i128,
    payee: Address,
) {
    let event = EscrowEvent {
        event_type: EventType::EscrowSettled,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id,
            payer: None,
            payee: Some(payee),
            amount,
            token: None,
            status: Some("settled".into()),
            previous_status: Some("active".into()),
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("SETTLE")), event);
}

/// Emit EscrowRefunded event
pub fn emit_escrow_refunded(
    env: &Env,
    caller: Address,
    order_id: u64,
    amount: i128,
    payer: Address,
) {
    let event = EscrowEvent {
        event_type: EventType::EscrowRefunded,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id,
            payer: Some(payer),
            payee: None,
            amount,
            token: None,
            status: Some("refunded".into()),
            previous_status: Some("pending".into()),
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("REFUND")), event);
}

/// Emit DisputeRaised event
pub fn emit_dispute_raised(env: &Env, caller: Address, order_id: u64, reason: Option<String>) {
    let event = EscrowEvent {
        event_type: EventType::DisputeRaised,
        category: OperationCategory::RecordOperations,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id,
            payer: None,
            payee: None,
            amount: 0,
            token: None,
            status: Some("disputed".into()),
            previous_status: None,
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("DISPUTE")), event);
}

/// Emit FeeCharged event
pub fn emit_fee_charged(
    env: &Env,
    caller: Address,
    order_id: u64,
    fee_amount: i128,
    fee_recipient: Address,
) {
    let event = EscrowEvent {
        event_type: EventType::FeeCharged,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller,
        block_height: env.ledger().sequence() as u64,
        data: EscrowEventData {
            order_id,
            payer: None,
            payee: Some(fee_recipient),
            amount: fee_amount,
            token: None,
            status: None,
            previous_status: None,
        },
    };
    env.events()
        .publish((symbol_short!("ESC"), symbol_short!("FEE")), event);
}
