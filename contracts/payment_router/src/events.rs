//! # PaymentRouter Events Module
//!
//! Standardized event emissions for the payment_router contract.
//! Topic naming convention: (ROUTE, ACTION)

#![allow(dead_code)]

use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    Initialized,
    Action,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    Administrative,
    Operations,
}

#[derive(Clone)]
#[contracttype]
pub struct PaymentRouterEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PaymentRouterEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PaymentRouterEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PaymentRouterEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PaymentRouterEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ROUTE"), symbol_short!("INIT")), event);
}

/// Emitted when set_fee_config is called.
pub fn emit_set_fee_config(env: &Env, caller: &Address) {
    let event = PaymentRouterEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PaymentRouterEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_fee_config"),
        },
    };
    env.events()
        .publish((symbol_short!("ROUTE"), symbol_short!("SET_FEE_C")), event);
}

/// Emitted when compute_split is called.
pub fn emit_compute_split(env: &Env, caller: &Address) {
    let event = PaymentRouterEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PaymentRouterEventData {
            user: caller.clone(),
            action: String::from_str(env, "compute_split"),
        },
    };
    env.events()
        .publish((symbol_short!("ROUTE"), symbol_short!("COMPUTE_S")), event);
}
