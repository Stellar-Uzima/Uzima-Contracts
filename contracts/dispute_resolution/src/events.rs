//! # DisputeResolution Events Module
//!
//! Standardized event emissions for the dispute_resolution contract.
//! Topic naming convention: (DISP, ACTION)

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
pub struct DisputeResolutionEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DisputeResolutionEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: DisputeResolutionEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = DisputeResolutionEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DisputeResolutionEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("DISP"), symbol_short!("INIT")), event);
}

/// Emitted when dispute is called.
pub fn emit_dispute(env: &Env, caller: &Address) {
    let event = DisputeResolutionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DisputeResolutionEventData {
            user: caller.clone(),
            action: String::from_str(env, "dispute"),
        },
    };
    env.events()
        .publish((symbol_short!("DISP"), symbol_short!("DISPUTE")), event);
}

/// Emitted when resolve is called.
pub fn emit_resolve(env: &Env, caller: &Address) {
    let event = DisputeResolutionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DisputeResolutionEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve"),
        },
    };
    env.events()
        .publish((symbol_short!("DISP"), symbol_short!("RESOLVE")), event);
}
