//! # UpgradeManager Events Module
//!
//! Standardized event emissions for the upgrade_manager contract.
//! Topic naming convention: (UPMGR, ACTION)

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
pub struct UpgradeManagerEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct UpgradeManagerEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: UpgradeManagerEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = UpgradeManagerEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("UPMGR"), symbol_short!("INIT")), event);
}

/// Emitted when propose_upgrade is called.
pub fn emit_propose_upgrade(env: &Env, caller: &Address) {
    let event = UpgradeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "propose_upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("UPMGR"), symbol_short!("PROPOSE_U")), event);
}

/// Emitted when approve is called.
pub fn emit_approve(env: &Env, caller: &Address) {
    let event = UpgradeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve"),
        },
    };
    env.events()
        .publish((symbol_short!("UPMGR"), symbol_short!("APPROVE")), event);
}

/// Emitted when execute is called.
pub fn emit_execute(env: &Env, caller: &Address) {
    let event = UpgradeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute"),
        },
    };
    env.events()
        .publish((symbol_short!("UPMGR"), symbol_short!("EXECUTE")), event);
}

/// Emitted when execute_emergency is called.
pub fn emit_execute_emergency(env: &Env, caller: &Address) {
    let event = UpgradeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_emergency"),
        },
    };
    env.events()
        .publish((symbol_short!("UPMGR"), symbol_short!("EXECUTE_E")), event);
}

/// Emitted when validate_proposal is called.
pub fn emit_validate_proposal(env: &Env, caller: &Address) {
    let event = UpgradeManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: UpgradeManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "validate_proposal"),
        },
    };
    env.events()
        .publish((symbol_short!("UPMGR"), symbol_short!("VALIDATE_")), event);
}
