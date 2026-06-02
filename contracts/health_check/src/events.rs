//! # HealthCheck Events Module
//!
//! Standardized event emissions for the health_check contract.
//! Topic naming convention: (HCHK, ACTION)

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
pub struct HealthCheckEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthCheckEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthCheckEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("INIT")), event);
}

/// Emitted when health_check is called.
pub fn emit_health_check(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "health_check"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("HEALTH_CH")), event);
}

/// Emitted when record_operation is called.
pub fn emit_record_operation(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_operation"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("RECORD_OP")), event);
}

/// Emitted when record_error is called.
pub fn emit_record_error(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_error"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("RECORD_ER")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("SET_PAUSE")), event);
}

/// Emitted when storage_usage is called.
pub fn emit_storage_usage(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "storage_usage"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("STORAGE_U")), event);
}

/// Emitted when last_activity is called.
pub fn emit_last_activity(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "last_activity"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("LAST_ACTI")), event);
}

/// Emitted when reset_recent_errors is called.
pub fn emit_reset_recent_errors(env: &Env, caller: &Address) {
    let event = HealthCheckEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthCheckEventData {
            user: caller.clone(),
            action: String::from_str(env, "reset_recent_errors"),
        },
    };
    env.events()
        .publish((symbol_short!("HCHK"), symbol_short!("RESET_REC")), event);
}
