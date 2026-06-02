//! # FailoverDetector Events Module
//!
//! Standardized event emissions for the failover_detector contract.
//! Topic naming convention: (FAIL, ACTION)

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
pub struct FailoverDetectorEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct FailoverDetectorEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: FailoverDetectorEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = FailoverDetectorEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FailoverDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FAIL"), symbol_short!("INIT")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = FailoverDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FailoverDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("FAIL"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when detect_node_failure is called.
pub fn emit_detect_node_failure(env: &Env, caller: &Address) {
    let event = FailoverDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FailoverDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "detect_node_failure"),
        },
    };
    env.events()
        .publish((symbol_short!("FAIL"), symbol_short!("DETECT_NO")), event);
}

/// Emitted when create_failover_plan is called.
pub fn emit_create_failover_plan(env: &Env, caller: &Address) {
    let event = FailoverDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FailoverDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_failover_plan"),
        },
    };
    env.events()
        .publish((symbol_short!("FAIL"), symbol_short!("CREATE_FA")), event);
}

/// Emitted when execute_failover is called.
pub fn emit_execute_failover(env: &Env, caller: &Address) {
    let event = FailoverDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FailoverDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_failover"),
        },
    };
    env.events()
        .publish((symbol_short!("FAIL"), symbol_short!("EXECUTE_F")), event);
}

/// Emitted when mark_recovery_success is called.
pub fn emit_mark_recovery_success(env: &Env, caller: &Address) {
    let event = FailoverDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FailoverDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_recovery_success"),
        },
    };
    env.events()
        .publish((symbol_short!("FAIL"), symbol_short!("MARK_RECO")), event);
}

/// Emitted when deactivate_failover_plan is called.
pub fn emit_deactivate_failover_plan(env: &Env, caller: &Address) {
    let event = FailoverDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FailoverDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_failover_plan"),
        },
    };
    env.events()
        .publish((symbol_short!("FAIL"), symbol_short!("DEACTIVAT")), event);
}
