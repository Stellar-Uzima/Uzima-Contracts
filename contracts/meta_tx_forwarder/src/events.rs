//! # MetaTxForwarder Events Module
//!
//! Standardized event emissions for the meta_tx_forwarder contract.
//! Topic naming convention: (META, ACTION)

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
pub struct MetaTxForwarderEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MetaTxForwarderEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MetaTxForwarderEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MetaTxForwarderEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MetaTxForwarderEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("META"), symbol_short!("INIT")), event);
}

/// Emitted when execute is called.
pub fn emit_execute(env: &Env, caller: &Address) {
    let event = MetaTxForwarderEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MetaTxForwarderEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute"),
        },
    };
    env.events()
        .publish((symbol_short!("META"), symbol_short!("EXECUTE")), event);
}

/// Emitted when execute_batch is called.
pub fn emit_execute_batch(env: &Env, caller: &Address) {
    let event = MetaTxForwarderEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MetaTxForwarderEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_batch"),
        },
    };
    env.events()
        .publish((symbol_short!("META"), symbol_short!("EXECUTE_B")), event);
}

/// Emitted when register_relayer is called.
pub fn emit_register_relayer(env: &Env, caller: &Address) {
    let event = MetaTxForwarderEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MetaTxForwarderEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_relayer"),
        },
    };
    env.events()
        .publish((symbol_short!("META"), symbol_short!("REGISTER_")), event);
}

/// Emitted when deactivate_relayer is called.
pub fn emit_deactivate_relayer(env: &Env, caller: &Address) {
    let event = MetaTxForwarderEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MetaTxForwarderEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_relayer"),
        },
    };
    env.events()
        .publish((symbol_short!("META"), symbol_short!("DEACTIVAT")), event);
}
