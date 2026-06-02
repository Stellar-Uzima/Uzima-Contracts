//! # SyncManager Events Module
//!
//! Standardized event emissions for the sync_manager contract.
//! Topic naming convention: (SYNCM, ACTION)

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
pub struct SyncManagerEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct SyncManagerEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: SyncManagerEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("INIT")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when initiate_sync is called.
pub fn emit_initiate_sync(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("INITIATE_")), event);
}

/// Emitted when execute_sync is called.
pub fn emit_execute_sync(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("EXECUTE_S")), event);
}

/// Emitted when retry_sync is called.
pub fn emit_retry_sync(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "retry_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("RETRY_SYN")), event);
}

/// Emitted when list_sync_operations is called.
pub fn emit_list_sync_operations(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_sync_operations"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("LIST_SYNC")), event);
}

/// Emitted when record_replication_lag is called.
pub fn emit_record_replication_lag(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_replication_lag"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("RECORD_RE")), event);
}

/// Emitted when detect_sync_conflict is called.
pub fn emit_detect_sync_conflict(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "detect_sync_conflict"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("DETECT_SY")), event);
}

/// Emitted when resolve_conflict is called.
pub fn emit_resolve_conflict(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_conflict"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("RESOLVE_C")), event);
}

/// Emitted when set_sync_policy is called.
pub fn emit_set_sync_policy(env: &Env, caller: &Address) {
    let event = SyncManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SyncManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_sync_policy"),
        },
    };
    env.events()
        .publish((symbol_short!("SYNCM"), symbol_short!("SET_SYNC_")), event);
}
