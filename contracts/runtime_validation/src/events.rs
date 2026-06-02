//! # RuntimeValidation Events Module
//!
//! Standardized event emissions for the runtime_validation contract.
//! Topic naming convention: (RVAL, ACTION)

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
pub struct RuntimeValidationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct RuntimeValidationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: RuntimeValidationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("INIT")), event);
}

/// Emitted when register_invariant is called.
pub fn emit_register_invariant(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_invariant"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_state_check is called.
pub fn emit_register_state_check(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_state_check"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_permission_check is called.
pub fn emit_register_permission_check(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_permission_check"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_resource_tracker is called.
pub fn emit_register_resource_tracker(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_resource_tracker"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("REGISTER_")), event);
}

/// Emitted when report_violation is called.
pub fn emit_report_violation(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_violation"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("REPORT_VI")), event);
}

/// Emitted when verify_invariant is called.
pub fn emit_verify_invariant(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_invariant"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("VERIFY_IN")), event);
}

/// Emitted when verify_state_consistency is called.
pub fn emit_verify_state_consistency(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_state_consistency"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("VERIFY_ST")), event);
}

/// Emitted when verify_permission is called.
pub fn emit_verify_permission(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_permission"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("VERIFY_PE")), event);
}

/// Emitted when update_resource_usage is called.
pub fn emit_update_resource_usage(env: &Env, caller: &Address) {
    let event = RuntimeValidationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RuntimeValidationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_resource_usage"),
        },
    };
    env.events()
        .publish((symbol_short!("RVAL"), symbol_short!("UPDATE_RE")), event);
}
