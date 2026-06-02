//! # DifferentialPrivacy Events Module
//!
//! Standardized event emissions for the differential_privacy contract.
//! Topic naming convention: (PRIV, ACTION)

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
pub struct DifferentialPrivacyEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DifferentialPrivacyEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: DifferentialPrivacyEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = DifferentialPrivacyEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DifferentialPrivacyEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PRIV"), symbol_short!("INIT")), event);
}

/// Emitted when create_budget is called.
pub fn emit_create_budget(env: &Env, caller: &Address) {
    let event = DifferentialPrivacyEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DifferentialPrivacyEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_budget"),
        },
    };
    env.events()
        .publish((symbol_short!("PRIV"), symbol_short!("CREATE_BU")), event);
}

/// Emitted when add_laplace_noise is called.
pub fn emit_add_laplace_noise(env: &Env, caller: &Address) {
    let event = DifferentialPrivacyEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DifferentialPrivacyEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_laplace_noise"),
        },
    };
    env.events()
        .publish((symbol_short!("PRIV"), symbol_short!("ADD_LAPLA")), event);
}

/// Emitted when add_gaussian_noise is called.
pub fn emit_add_gaussian_noise(env: &Env, caller: &Address) {
    let event = DifferentialPrivacyEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DifferentialPrivacyEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_gaussian_noise"),
        },
    };
    env.events()
        .publish((symbol_short!("PRIV"), symbol_short!("ADD_GAUSS")), event);
}

/// Emitted when deactivate_budget is called.
pub fn emit_deactivate_budget(env: &Env, caller: &Address) {
    let event = DifferentialPrivacyEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DifferentialPrivacyEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_budget"),
        },
    };
    env.events()
        .publish((symbol_short!("PRIV"), symbol_short!("DEACTIVAT")), event);
}
