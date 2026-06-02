//! # EmergencyAccessOverride Events Module
//!
//! Standardized event emissions for the emergency_access_override contract.
//! Topic naming convention: (EMERG, ACTION)

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
pub struct EmergencyAccessOverrideEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct EmergencyAccessOverrideEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: EmergencyAccessOverrideEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("INIT")), event);
}

/// Emitted when grant_emergency_access is called.
pub fn emit_grant_emergency_access(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_emergency_access"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("GRANT_EME")), event);
}

/// Emitted when reset_circuit_breaker is called.
pub fn emit_reset_circuit_breaker(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "reset_circuit_breaker"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("RESET_CIR")), event);
}

/// Emitted when update_cooldown_period is called.
pub fn emit_update_cooldown_period(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_cooldown_period"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when revoke_emergency_access is called.
pub fn emit_revoke_emergency_access(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_emergency_access"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("REVOKE_EM")), event);
}

/// Emitted when health_check is called.
pub fn emit_health_check(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "health_check"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("HEALTH_CH")), event);
}

/// Emitted when configure_multisig is called.
pub fn emit_configure_multisig(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_multisig"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when request_emergency_access is called.
pub fn emit_request_emergency_access(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_emergency_access"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("REQUEST_E")), event);
}

/// Emitted when approve_emergency_access is called.
pub fn emit_approve_emergency_access(env: &Env, caller: &Address) {
    let event = EmergencyAccessOverrideEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmergencyAccessOverrideEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve_emergency_access"),
        },
    };
    env.events()
        .publish((symbol_short!("EMERG"), symbol_short!("APPROVE_E")), event);
}
