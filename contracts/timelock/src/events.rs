//! # Timelock Events Module
//!
//! Standardized event emissions for the timelock contract.
//! Topic naming convention: (TLOCK, ACTION)

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
pub struct TimelockEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct TimelockEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: TimelockEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = TimelockEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TimelockEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("TLOCK"), symbol_short!("INIT")), event);
}

/// Emitted when queue is called.
pub fn emit_queue(env: &Env, caller: &Address) {
    let event = TimelockEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TimelockEventData {
            user: caller.clone(),
            action: String::from_str(env, "queue"),
        },
    };
    env.events()
        .publish((symbol_short!("TLOCK"), symbol_short!("QUEUE")), event);
}

/// Emitted when execute is called.
pub fn emit_execute(env: &Env, caller: &Address) {
    let event = TimelockEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TimelockEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute"),
        },
    };
    env.events()
        .publish((symbol_short!("TLOCK"), symbol_short!("EXECUTE")), event);
}
