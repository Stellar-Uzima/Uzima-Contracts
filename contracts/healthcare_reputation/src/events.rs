//! # HealthcareReputation Events Module
//!
//! Standardized event emissions for the healthcare_reputation contract.
//! Topic naming convention: (HREP, ACTION)

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
pub struct HealthcareReputationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareReputationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcareReputationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthcareReputationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HREP"), symbol_short!("INIT")), event);
}

/// Emitted when add_credential is called.
pub fn emit_add_credential(env: &Env, caller: &Address) {
    let event = HealthcareReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_credential"),
        },
    };
    env.events()
        .publish((symbol_short!("HREP"), symbol_short!("ADD_CREDE")), event);
}

/// Emitted when verify_credential is called.
pub fn emit_verify_credential(env: &Env, caller: &Address) {
    let event = HealthcareReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_credential"),
        },
    };
    env.events()
        .publish((symbol_short!("HREP"), symbol_short!("VERIFY_CR")), event);
}

/// Emitted when add_feedback is called.
pub fn emit_add_feedback(env: &Env, caller: &Address) {
    let event = HealthcareReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_feedback"),
        },
    };
    env.events()
        .publish((symbol_short!("HREP"), symbol_short!("ADD_FEEDB")), event);
}

/// Emitted when add_conduct_entry is called.
pub fn emit_add_conduct_entry(env: &Env, caller: &Address) {
    let event = HealthcareReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_conduct_entry"),
        },
    };
    env.events()
        .publish((symbol_short!("HREP"), symbol_short!("ADD_CONDU")), event);
}

/// Emitted when create_dispute is called.
pub fn emit_create_dispute(env: &Env, caller: &Address) {
    let event = HealthcareReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_dispute"),
        },
    };
    env.events()
        .publish((symbol_short!("HREP"), symbol_short!("CREATE_DI")), event);
}

/// Emitted when resolve_dispute is called.
pub fn emit_resolve_dispute(env: &Env, caller: &Address) {
    let event = HealthcareReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_dispute"),
        },
    };
    env.events()
        .publish((symbol_short!("HREP"), symbol_short!("RESOLVE_D")), event);
}
