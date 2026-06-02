//! # Forensics Events Module
//!
//! Standardized event emissions for the forensics contract.
//! Topic naming convention: (FOREN, ACTION)

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
pub struct ForensicsEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ForensicsEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ForensicsEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ForensicsEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("INIT")), event);
}

/// Emitted when collect_evidence is called.
pub fn emit_collect_evidence(env: &Env, caller: &Address) {
    let event = ForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "collect_evidence"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("COLLECT_E")), event);
}

/// Emitted when analyze_pattern is called.
pub fn emit_analyze_pattern(env: &Env, caller: &Address) {
    let event = ForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "analyze_pattern"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("ANALYZE_P")), event);
}

/// Emitted when detect_suspicious is called.
pub fn emit_detect_suspicious(env: &Env, caller: &Address) {
    let event = ForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "detect_suspicious"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("DETECT_SU")), event);
}

/// Emitted when generate_report is called.
pub fn emit_generate_report(env: &Env, caller: &Address) {
    let event = ForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_report"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("GENERATE_")), event);
}

/// Emitted when update_investigation is called.
pub fn emit_update_investigation(env: &Env, caller: &Address) {
    let event = ForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_investigation"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("UPDATE_IN")), event);
}

/// Emitted when blacklist_actor is called.
pub fn emit_blacklist_actor(env: &Env, caller: &Address) {
    let event = ForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "blacklist_actor"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("BLACKLIST")), event);
}
