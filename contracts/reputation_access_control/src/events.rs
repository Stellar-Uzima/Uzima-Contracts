//! # ReputationAccessControl Events Module
//!
//! Standardized event emissions for the reputation_access_control contract.
//! Topic naming convention: (REPAC, ACTION)

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
pub struct ReputationAccessControlEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ReputationAccessControlEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ReputationAccessControlEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("INIT")), event);
}

/// Emitted when set_access_policy is called.
pub fn emit_set_access_policy(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_access_policy"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("SET_ACCES")), event);
}

/// Emitted when request_access is called.
pub fn emit_request_access(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_access"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("REQUEST_A")), event);
}

/// Emitted when approve_request is called.
pub fn emit_approve_request(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve_request"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("APPROVE_R")), event);
}

/// Emitted when deny_request is called.
pub fn emit_deny_request(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "deny_request"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("DENY_REQU")), event);
}

/// Emitted when grant_emergency_access is called.
pub fn emit_grant_emergency_access(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_emergency_access"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("GRANT_EME")), event);
}

/// Emitted when revoke_emergency_access is called.
pub fn emit_revoke_emergency_access(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_emergency_access"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("REVOKE_EM")), event);
}

/// Emitted when set_reputation_threshold is called.
pub fn emit_set_reputation_threshold(env: &Env, caller: &Address) {
    let event = ReputationAccessControlEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationAccessControlEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_reputation_threshold"),
        },
    };
    env.events()
        .publish((symbol_short!("REPAC"), symbol_short!("SET_REPUT")), event);
}
