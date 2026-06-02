//! # DeprecationFramework Events Module
//!
//! Standardized event emissions for the deprecation_framework contract.
//! Topic naming convention: (DEPR, ACTION)

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
pub struct DeprecationFrameworkEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DeprecationFrameworkEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: DeprecationFrameworkEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("INIT")), event);
}

/// Emitted when mark_for_deprecation is called.
pub fn emit_mark_for_deprecation(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_for_deprecation"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("MARK_FOR_")), event);
}

/// Emitted when set_sunset_timeline is called.
pub fn emit_set_sunset_timeline(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_sunset_timeline"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("SET_SUNSE")), event);
}

/// Emitted when add_migration_guide is called.
pub fn emit_add_migration_guide(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_migration_guide"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("ADD_MIGRA")), event);
}

/// Emitted when update_deprecation_phase is called.
pub fn emit_update_deprecation_phase(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_deprecation_phase"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("UPDATE_DE")), event);
}

/// Emitted when publish_user_communication is called.
pub fn emit_publish_user_communication(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "publish_user_communication"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("PUBLISH_U")), event);
}

/// Emitted when create_removal_checklist is called.
pub fn emit_create_removal_checklist(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_removal_checklist"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("CREATE_RE")), event);
}

/// Emitted when mark_checklist_item_complete is called.
pub fn emit_mark_checklist_item_complete(env: &Env, caller: &Address) {
    let event = DeprecationFrameworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DeprecationFrameworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_checklist_item_complete"),
        },
    };
    env.events()
        .publish((symbol_short!("DEPR"), symbol_short!("MARK_CHEC")), event);
}
