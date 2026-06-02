//! # StorageCleanup Events Module
//!
//! Standardized event emissions for the storage_cleanup contract.
//! Topic naming convention: (CLEAN, ACTION)

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
pub struct StorageCleanupEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct StorageCleanupEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: StorageCleanupEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("INIT")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("SET_PAUSE")), event);
}

/// Emitted when set_retention_config is called.
pub fn emit_set_retention_config(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_retention_config"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("SET_RETEN")), event);
}

/// Emitted when register_credential is called.
pub fn emit_register_credential(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_credential"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_audit_log is called.
pub fn emit_register_audit_log(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_audit_log"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_escrow is called.
pub fn emit_register_escrow(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_escrow"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_consent is called.
pub fn emit_register_consent(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_schedule is called.
pub fn emit_register_schedule(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_schedule"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("REGISTER_")), event);
}

/// Emitted when cleanup_expired is called.
pub fn emit_cleanup_expired(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_expired"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("CLEANUP_E")), event);
}

/// Emitted when cleanup_credentials is called.
pub fn emit_cleanup_credentials(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_credentials"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("CLEANUP_C")), event);
}

/// Emitted when cleanup_audit_logs is called.
pub fn emit_cleanup_audit_logs(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_audit_logs"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("CLEANUP_A")), event);
}

/// Emitted when cleanup_escrows is called.
pub fn emit_cleanup_escrows(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_escrows"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("CLEANUP_E")), event);
}

/// Emitted when cleanup_consents is called.
pub fn emit_cleanup_consents(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_consents"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("CLEANUP_C")), event);
}

/// Emitted when cleanup_schedules is called.
pub fn emit_cleanup_schedules(env: &Env, caller: &Address) {
    let event = StorageCleanupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: StorageCleanupEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_schedules"),
        },
    };
    env.events()
        .publish((symbol_short!("CLEAN"), symbol_short!("CLEANUP_S")), event);
}
