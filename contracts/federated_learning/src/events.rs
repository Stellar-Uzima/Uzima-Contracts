//! # FederatedLearning Events Module
//!
//! Standardized event emissions for the federated_learning contract.
//! Topic naming convention: (FEDL, ACTION)

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
pub struct FederatedLearningEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct FederatedLearningEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: FederatedLearningEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("INIT")), event);
}

/// Emitted when register_institution is called.
pub fn emit_register_institution(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_institution"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("REGISTER_")), event);
}

/// Emitted when start_round is called.
pub fn emit_start_round(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "start_round"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("START_ROU")), event);
}

/// Emitted when submit_update is called.
pub fn emit_submit_update(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_update"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("SUBMIT_UP")), event);
}

/// Emitted when begin_aggregation is called.
pub fn emit_begin_aggregation(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "begin_aggregation"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("BEGIN_AGG")), event);
}

/// Emitted when finalize_round is called.
pub fn emit_finalize_round(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "finalize_round"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("FINALIZE_")), event);
}

/// Emitted when update_communication_metrics is called.
pub fn emit_update_communication_metrics(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_communication_metrics"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when blacklist_institution is called.
pub fn emit_blacklist_institution(env: &Env, caller: &Address) {
    let event = FederatedLearningEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FederatedLearningEventData {
            user: caller.clone(),
            action: String::from_str(env, "blacklist_institution"),
        },
    };
    env.events()
        .publish((symbol_short!("FEDL"), symbol_short!("BLACKLIST")), event);
}
