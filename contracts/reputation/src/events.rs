//! # Reputation Events Module
//!
//! Standardized event emissions for the reputation contract.
//! Topic naming convention: (REP, ACTION)

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
pub struct ReputationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ReputationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ReputationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ReputationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("REP"), symbol_short!("INIT")), event);
}

/// Emitted when mint is called.
pub fn emit_mint(env: &Env, caller: &Address) {
    let event = ReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "mint"),
        },
    };
    env.events()
        .publish((symbol_short!("REP"), symbol_short!("MINT")), event);
}

/// Emitted when slash is called.
pub fn emit_slash(env: &Env, caller: &Address) {
    let event = ReputationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationEventData {
            user: caller.clone(),
            action: String::from_str(env, "slash"),
        },
    };
    env.events()
        .publish((symbol_short!("REP"), symbol_short!("SLASH")), event);
}
