//! # Mfa Events Module
//!
//! Standardized event emissions for the mfa contract.
//! Topic naming convention: (MFA, ACTION)

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
pub struct MfaEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MfaEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MfaEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MfaEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MfaEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MFA"), symbol_short!("INIT")), event);
}

/// Emitted when add_factor is called.
pub fn emit_add_factor(env: &Env, caller: &Address) {
    let event = MfaEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MfaEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_factor"),
        },
    };
    env.events()
        .publish((symbol_short!("MFA"), symbol_short!("ADD_FACTO")), event);
}

/// Emitted when start_session is called.
pub fn emit_start_session(env: &Env, caller: &Address) {
    let event = MfaEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MfaEventData {
            user: caller.clone(),
            action: String::from_str(env, "start_session"),
        },
    };
    env.events()
        .publish((symbol_short!("MFA"), symbol_short!("START_SES")), event);
}

/// Emitted when verify_mfa_factor is called.
pub fn emit_verify_mfa_factor(env: &Env, caller: &Address) {
    let event = MfaEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MfaEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_mfa_factor"),
        },
    };
    env.events()
        .publish((symbol_short!("MFA"), symbol_short!("VERIFY_MF")), event);
}

/// Emitted when initiate_recovery is called.
pub fn emit_initiate_recovery(env: &Env, caller: &Address) {
    let event = MfaEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MfaEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_recovery"),
        },
    };
    env.events()
        .publish((symbol_short!("MFA"), symbol_short!("INITIATE_")), event);
}

/// Emitted when emergency_override is called.
pub fn emit_emergency_override(env: &Env, caller: &Address) {
    let event = MfaEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MfaEventData {
            user: caller.clone(),
            action: String::from_str(env, "emergency_override"),
        },
    };
    env.events()
        .publish((symbol_short!("MFA"), symbol_short!("EMERGENCY")), event);
}
