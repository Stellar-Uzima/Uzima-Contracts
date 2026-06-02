//! # Sanitization Events Module
//!
//! Standardized event emissions for the sanitization contract.
//! Topic naming convention: (SAN, ACTION)

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
pub struct SanitizationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct SanitizationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: SanitizationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = SanitizationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SanitizationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("SAN"), symbol_short!("INIT")), event);
}

/// Emitted when sanitize_string is called.
pub fn emit_sanitize_string(env: &Env, caller: &Address) {
    let event = SanitizationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SanitizationEventData {
            user: caller.clone(),
            action: String::from_str(env, "sanitize_string"),
        },
    };
    env.events()
        .publish((symbol_short!("SAN"), symbol_short!("SANITIZE_")), event);
}

/// Emitted when sanitize_name is called.
pub fn emit_sanitize_name(env: &Env, caller: &Address) {
    let event = SanitizationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SanitizationEventData {
            user: caller.clone(),
            action: String::from_str(env, "sanitize_name"),
        },
    };
    env.events()
        .publish((symbol_short!("SAN"), symbol_short!("SANITIZE_")), event);
}

/// Emitted when sanitize_email is called.
pub fn emit_sanitize_email(env: &Env, caller: &Address) {
    let event = SanitizationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SanitizationEventData {
            user: caller.clone(),
            action: String::from_str(env, "sanitize_email"),
        },
    };
    env.events()
        .publish((symbol_short!("SAN"), symbol_short!("SANITIZE_")), event);
}

/// Emitted when sanitize_id is called.
pub fn emit_sanitize_id(env: &Env, caller: &Address) {
    let event = SanitizationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SanitizationEventData {
            user: caller.clone(),
            action: String::from_str(env, "sanitize_id"),
        },
    };
    env.events()
        .publish((symbol_short!("SAN"), symbol_short!("SANITIZE_")), event);
}

/// Emitted when sanitize_url is called.
pub fn emit_sanitize_url(env: &Env, caller: &Address) {
    let event = SanitizationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SanitizationEventData {
            user: caller.clone(),
            action: String::from_str(env, "sanitize_url"),
        },
    };
    env.events()
        .publish((symbol_short!("SAN"), symbol_short!("SANITIZE_")), event);
}
