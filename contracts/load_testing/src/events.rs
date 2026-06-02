//! # LoadTesting Events Module
//!
//! Standardized event emissions for the load_testing contract.
//! Topic naming convention: (LOAD, ACTION)

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
pub struct LoadTestingEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct LoadTestingEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: LoadTestingEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = LoadTestingEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: LoadTestingEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("LOAD"), symbol_short!("INIT")), event);
}

/// Emitted when run is called.
pub fn emit_run(env: &Env, caller: &Address) {
    let event = LoadTestingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: LoadTestingEventData {
            user: caller.clone(),
            action: String::from_str(env, "run"),
        },
    };
    env.events()
        .publish((symbol_short!("LOAD"), symbol_short!("RUN")), event);
}

/// Emitted when last_result is called.
pub fn emit_last_result(env: &Env, caller: &Address) {
    let event = LoadTestingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: LoadTestingEventData {
            user: caller.clone(),
            action: String::from_str(env, "last_result"),
        },
    };
    env.events()
        .publish((symbol_short!("LOAD"), symbol_short!("LAST_RESU")), event);
}

/// Emitted when run_count is called.
pub fn emit_run_count(env: &Env, caller: &Address) {
    let event = LoadTestingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: LoadTestingEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_count"),
        },
    };
    env.events()
        .publish((symbol_short!("LOAD"), symbol_short!("RUN_COUNT")), event);
}
