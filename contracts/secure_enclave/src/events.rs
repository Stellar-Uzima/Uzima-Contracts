//! # SecureEnclave Events Module
//!
//! Standardized event emissions for the secure_enclave contract.
//! Topic naming convention: (ENCL, ACTION)

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
pub struct SecureEnclaveEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct SecureEnclaveEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: SecureEnclaveEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = SecureEnclaveEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SecureEnclaveEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ENCL"), symbol_short!("INIT")), event);
}

/// Emitted when register_enclave is called.
pub fn emit_register_enclave(env: &Env, caller: &Address) {
    let event = SecureEnclaveEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SecureEnclaveEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_enclave"),
        },
    };
    env.events()
        .publish((symbol_short!("ENCL"), symbol_short!("REGISTER_")), event);
}

/// Emitted when verify_attestation is called.
pub fn emit_verify_attestation(env: &Env, caller: &Address) {
    let event = SecureEnclaveEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SecureEnclaveEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_attestation"),
        },
    };
    env.events()
        .publish((symbol_short!("ENCL"), symbol_short!("VERIFY_AT")), event);
}

/// Emitted when submit_task is called.
pub fn emit_submit_task(env: &Env, caller: &Address) {
    let event = SecureEnclaveEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SecureEnclaveEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_task"),
        },
    };
    env.events()
        .publish((symbol_short!("ENCL"), symbol_short!("SUBMIT_TA")), event);
}

/// Emitted when assign_task is called.
pub fn emit_assign_task(env: &Env, caller: &Address) {
    let event = SecureEnclaveEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SecureEnclaveEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_task"),
        },
    };
    env.events()
        .publish((symbol_short!("ENCL"), symbol_short!("ASSIGN_TA")), event);
}

/// Emitted when complete_task is called.
pub fn emit_complete_task(env: &Env, caller: &Address) {
    let event = SecureEnclaveEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SecureEnclaveEventData {
            user: caller.clone(),
            action: String::from_str(env, "complete_task"),
        },
    };
    env.events()
        .publish((symbol_short!("ENCL"), symbol_short!("COMPLETE_")), event);
}

/// Emitted when fallback_to_mpc is called.
pub fn emit_fallback_to_mpc(env: &Env, caller: &Address) {
    let event = SecureEnclaveEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SecureEnclaveEventData {
            user: caller.clone(),
            action: String::from_str(env, "fallback_to_mpc"),
        },
    };
    env.events()
        .publish((symbol_short!("ENCL"), symbol_short!("FALLBACK_")), event);
}
