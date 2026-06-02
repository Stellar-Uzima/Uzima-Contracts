//! # HealthDataAccessLogging Events Module
//!
//! Standardized event emissions for the health_data_access_logging contract.
//! Topic naming convention: (HLOG, ACTION)

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
pub struct HealthDataAccessLoggingEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthDataAccessLoggingEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthDataAccessLoggingEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthDataAccessLoggingEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthDataAccessLoggingEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HLOG"), symbol_short!("INIT")), event);
}

/// Emitted when log_access is called.
pub fn emit_log_access(env: &Env, caller: &Address) {
    let event = HealthDataAccessLoggingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthDataAccessLoggingEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_access"),
        },
    };
    env.events()
        .publish((symbol_short!("HLOG"), symbol_short!("LOG_ACCES")), event);
}

/// Emitted when verify_logs_integrity is called.
pub fn emit_verify_logs_integrity(env: &Env, caller: &Address) {
    let event = HealthDataAccessLoggingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthDataAccessLoggingEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_logs_integrity"),
        },
    };
    env.events()
        .publish((symbol_short!("HLOG"), symbol_short!("VERIFY_LO")), event);
}

/// Emitted when update_config is called.
pub fn emit_update_config(env: &Env, caller: &Address) {
    let event = HealthDataAccessLoggingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthDataAccessLoggingEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_config"),
        },
    };
    env.events()
        .publish((symbol_short!("HLOG"), symbol_short!("UPDATE_CO")), event);
}
