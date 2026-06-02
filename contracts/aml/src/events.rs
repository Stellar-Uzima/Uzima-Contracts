//! # Aml Events Module
//!
//! Standardized event emissions for the aml contract.
//! Topic naming convention: (AML, ACTION)

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
pub struct AmlEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AmlEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AmlEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("INIT")), event);
}

/// Emitted when configure_rule is called.
pub fn emit_configure_rule(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_rule"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when monitor_transaction is called.
pub fn emit_monitor_transaction(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "monitor_transaction"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("MONITOR_T")), event);
}

/// Emitted when update_user_status is called.
pub fn emit_update_user_status(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_user_status"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("UPDATE_US")), event);
}

/// Emitted when set_user_status is called.
pub fn emit_set_user_status(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_user_status"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("SET_USER_")), event);
}

/// Emitted when report_incident is called.
pub fn emit_report_incident(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_incident"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("REPORT_IN")), event);
}

/// Emitted when register_deprecated_functions is called.
pub fn emit_register_deprecated_functions(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_deprecated_functions"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("REGISTER_")), event);
}

/// Emitted when upgrade is called.
pub fn emit_upgrade(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("UPGRADE")), event);
}

/// Emitted when validate_upgrade is called.
pub fn emit_validate_upgrade(env: &Env, caller: &Address) {
    let event = AmlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AmlEventData {
            user: caller.clone(),
            action: String::from_str(env, "validate_upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("AML"), symbol_short!("VALIDATE_")), event);
}
