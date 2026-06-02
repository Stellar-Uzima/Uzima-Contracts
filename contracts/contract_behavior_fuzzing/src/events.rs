//! # ContractBehaviorFuzzing Events Module
//!
//! Standardized event emissions for the contract_behavior_fuzzing contract.
//! Topic naming convention: (FUZZ, ACTION)

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
pub struct ContractBehaviorFuzzingEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ContractBehaviorFuzzingEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ContractBehaviorFuzzingEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ContractBehaviorFuzzingEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractBehaviorFuzzingEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FUZZ"), symbol_short!("INIT")), event);
}

/// Emitted when execute_sequence is called.
pub fn emit_execute_sequence(env: &Env, caller: &Address) {
    let event = ContractBehaviorFuzzingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractBehaviorFuzzingEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_sequence"),
        },
    };
    env.events()
        .publish((symbol_short!("FUZZ"), symbol_short!("EXECUTE_S")), event);
}

/// Emitted when run_regressions is called.
pub fn emit_run_regressions(env: &Env, caller: &Address) {
    let event = ContractBehaviorFuzzingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractBehaviorFuzzingEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_regressions"),
        },
    };
    env.events()
        .publish((symbol_short!("FUZZ"), symbol_short!("RUN_REGRE")), event);
}
