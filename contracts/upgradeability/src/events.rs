//! # Upgradeability Events Module
//!
//! Standardized event emissions for the upgradeability contract.
//! Topic naming convention: (UPGRD, ACTION)

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
pub struct UpgradeabilityEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct UpgradeabilityEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: UpgradeabilityEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("INIT")), event);
}

/// Emitted when set_version is called.
pub fn emit_set_version(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_version"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("SET_VERSI")), event);
}

/// Emitted when set_admin is called.
pub fn emit_set_admin(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_admin"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("SET_ADMIN")), event);
}

/// Emitted when freeze is called.
pub fn emit_freeze(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "freeze"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("FREEZE")), event);
}

/// Emitted when add_history is called.
pub fn emit_add_history(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_history"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("ADD_HISTO")), event);
}

/// Emitted when set_deprecated_functions is called.
pub fn emit_set_deprecated_functions(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_deprecated_functions"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("SET_DEPRE")), event);
}

/// Emitted when authorize_upgrade is called.
pub fn emit_authorize_upgrade(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "authorize_upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("AUTHORIZE")), event);
}

/// Emitted when execute_upgrade is called.
pub fn emit_execute_upgrade(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("EXECUTE_U")), event);
}

/// Emitted when execute_upgrade_with_deprecations is called.
pub fn emit_execute_upgrade_with_deprecations(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_upgrade_with_deprecations"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("EXECUTE_U")), event);
}

/// Emitted when validate_upgrade is called.
pub fn emit_validate_upgrade(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "validate_upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("VALIDATE_")), event);
}

/// Emitted when rollback is called.
pub fn emit_rollback(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "rollback"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("ROLLBACK")), event);
}

/// Emitted when emit_deprecation_warning is called.
pub fn emit_emit_deprecation_warning(env: &Env, caller: &Address) {
    let event = UpgradeabilityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeabilityEventData {
            user: caller.clone(),
            action: String::from_str(env, "emit_deprecation_warning"),
        },
    };
    env.events()
        .publish((symbol_short!("UPGRD"), symbol_short!("DEPRECATI")), event);
}
