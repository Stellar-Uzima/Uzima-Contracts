//! # Audit Events Module
//!
//! Standardized event emissions for the audit contract.
//! Topic naming convention: (AUDIT, ACTION)

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
pub struct AuditEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AuditEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("INIT")), event);
}

/// Emitted when log_event is called.
pub fn emit_log_event(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_event"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("LOG_EVENT")), event);
}

/// Emitted when log_data_access is called.
pub fn emit_log_data_access(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_data_access"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("LOG_DATA_")), event);
}

/// Emitted when log_permission_change is called.
pub fn emit_log_permission_change(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_permission_change"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("LOG_PERMI")), event);
}

/// Emitted when log_auth_attempt is called.
pub fn emit_log_auth_attempt(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_auth_attempt"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("LOG_AUTH_")), event);
}

/// Emitted when log_cross_chain_transfer is called.
pub fn emit_log_cross_chain_transfer(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_cross_chain_transfer"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("LOG_CROSS")), event);
}

/// Emitted when grant_log_access is called.
pub fn emit_grant_log_access(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_log_access"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("GRANT_LOG")), event);
}

/// Emitted when revoke_log_access is called.
pub fn emit_revoke_log_access(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_log_access"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("REVOKE_LO")), event);
}

/// Emitted when set_retention_policy is called.
pub fn emit_set_retention_policy(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_retention_policy"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("SET_RETEN")), event);
}

/// Emitted when verify_retention is called.
pub fn emit_verify_retention(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_retention"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("VERIFY_RE")), event);
}

/// Emitted when export_logs is called.
pub fn emit_export_logs(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "export_logs"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("EXPORT_LO")), event);
}

/// Emitted when verify_log_integrity is called.
pub fn emit_verify_log_integrity(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_log_integrity"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("VERIFY_LO")), event);
}

/// Emitted when verify_integrity is called.
pub fn emit_verify_integrity(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_integrity"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("VERIFY_IN")), event);
}

/// Emitted when generate_summary is called.
pub fn emit_generate_summary(env: &Env, caller: &Address) {
    let event = AuditEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_summary"),
        },
    };
    env.events()
        .publish((symbol_short!("AUDIT"), symbol_short!("GENERATE_")), event);
}
