//! # CrossChainIdentity Events Module
//!
//! Standardized event emissions for the cross_chain_identity contract.
//! Topic naming convention: (XCID, ACTION)

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
pub struct CrossChainIdentityEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainIdentityEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: CrossChainIdentityEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("INIT")), event);
}

/// Emitted when add_validator is called.
pub fn emit_add_validator(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_validator"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("ADD_VALID")), event);
}

/// Emitted when deactivate_validator is called.
pub fn emit_deactivate_validator(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_validator"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when update_trust_score is called.
pub fn emit_update_trust_score(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_trust_score"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("UPDATE_TR")), event);
}

/// Emitted when set_min_attestations is called.
pub fn emit_set_min_attestations(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_min_attestations"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("SET_MIN_A")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when request_verification is called.
pub fn emit_request_verification(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_verification"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("REQUEST_V")), event);
}

/// Emitted when attest_verification is called.
pub fn emit_attest_verification(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "attest_verification"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("ATTEST_VE")), event);
}

/// Emitted when revoke_identity is called.
pub fn emit_revoke_identity(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_identity"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("REVOKE_ID")), event);
}

/// Emitted when initiate_sync is called.
pub fn emit_initiate_sync(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("INITIATE_")), event);
}

/// Emitted when update_sync_status is called.
pub fn emit_update_sync_status(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_sync_status"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("UPDATE_SY")), event);
}

/// Emitted when verify_identity is called.
pub fn emit_verify_identity(env: &Env, caller: &Address) {
    let event = CrossChainIdentityEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainIdentityEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_identity"),
        },
    };
    env.events()
        .publish((symbol_short!("XCID"), symbol_short!("VERIFY_ID")), event);
}
