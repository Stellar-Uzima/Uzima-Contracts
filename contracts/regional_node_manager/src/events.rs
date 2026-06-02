//! # RegionalNodeManager Events Module
//!
//! Standardized event emissions for the regional_node_manager contract.
//! Topic naming convention: (RGNOD, ACTION)

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
pub struct RegionalNodeManagerEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct RegionalNodeManagerEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: RegionalNodeManagerEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("INIT")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when register_node is called.
pub fn emit_register_node(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_node"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("REGISTER_")), event);
}

/// Emitted when list_nodes is called.
pub fn emit_list_nodes(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_nodes"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("LIST_NODE")), event);
}

/// Emitted when update_node_metrics is called.
pub fn emit_update_node_metrics(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_node_metrics"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("UPDATE_NO")), event);
}

/// Emitted when perform_health_check is called.
pub fn emit_perform_health_check(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "perform_health_check"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("PERFORM_H")), event);
}

/// Emitted when register_replica is called.
pub fn emit_register_replica(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_replica"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("REGISTER_")), event);
}

/// Emitted when update_replica_sync is called.
pub fn emit_update_replica_sync(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_replica_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("UPDATE_RE")), event);
}

/// Emitted when set_configuration is called.
pub fn emit_set_configuration(env: &Env, caller: &Address) {
    let event = RegionalNodeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegionalNodeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_configuration"),
        },
    };
    env.events()
        .publish((symbol_short!("RGNOD"), symbol_short!("SET_CONFI")), event);
}
