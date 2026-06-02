//! # CrossChainAccess Events Module
//!
//! Standardized event emissions for the cross_chain_access contract.
//! Topic naming convention: (XCA, ACTION)

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
pub struct CrossChainAccessEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainAccessEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: CrossChainAccessEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("INIT")), event);
}

/// Emitted when grant_access is called.
pub fn emit_grant_access(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_access"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("GRANT_ACC")), event);
}

/// Emitted when revoke_access is called.
pub fn emit_revoke_access(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_access"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("REVOKE_AC")), event);
}

/// Emitted when update_grant_conditions is called.
pub fn emit_update_grant_conditions(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_grant_conditions"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("UPDATE_GR")), event);
}

/// Emitted when extend_grant is called.
pub fn emit_extend_grant(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "extend_grant"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("EXTEND_GR")), event);
}

/// Emitted when request_access is called.
pub fn emit_request_access(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_access"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("REQUEST_A")), event);
}

/// Emitted when process_request is called.
pub fn emit_process_request(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "process_request"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("PROCESS_R")), event);
}

/// Emitted when create_delegation is called.
pub fn emit_create_delegation(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_delegation"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("CREATE_DE")), event);
}

/// Emitted when revoke_delegation is called.
pub fn emit_revoke_delegation(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_delegation"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("REVOKE_DE")), event);
}

/// Emitted when configure_emergency is called.
pub fn emit_configure_emergency(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_emergency"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when log_access is called.
pub fn emit_log_access(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_access"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("LOG_ACCES")), event);
}

/// Emitted when initiate_access_swap is called.
pub fn emit_initiate_access_swap(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_access_swap"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("INITIATE_")), event);
}

/// Emitted when accept_access_swap is called.
pub fn emit_accept_access_swap(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "accept_access_swap"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("ACCEPT_AC")), event);
}

/// Emitted when finalize_access_swap is called.
pub fn emit_finalize_access_swap(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "finalize_access_swap"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("FINALIZE_")), event);
}

/// Emitted when cancel_access_swap is called.
pub fn emit_cancel_access_swap(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "cancel_access_swap"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("CANCEL_AC")), event);
}

/// Emitted when verify_access is called.
pub fn emit_verify_access(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_access"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("VERIFY_AC")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = CrossChainAccessEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainAccessEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("XCA"), symbol_short!("UNPAUSE")), event);
}
