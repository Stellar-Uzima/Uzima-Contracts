//! # AiAnalytics Events Module
//!
//! Standardized event emissions for the ai_analytics contract.
//! Topic naming convention: (AIANA, ACTION)

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
pub struct AiAnalyticsEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AiAnalyticsEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AiAnalyticsEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = AiAnalyticsEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AiAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("AIANA"), symbol_short!("INIT")), event);
}

/// Emitted when start_round is called.
pub fn emit_start_round(env: &Env, caller: &Address) {
    let event = AiAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AiAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "start_round"),
        },
    };
    env.events()
        .publish((symbol_short!("AIANA"), symbol_short!("START_ROU")), event);
}

/// Emitted when submit_update is called.
pub fn emit_submit_update(env: &Env, caller: &Address) {
    let event = AiAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AiAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_update"),
        },
    };
    env.events()
        .publish((symbol_short!("AIANA"), symbol_short!("SUBMIT_UP")), event);
}

/// Emitted when finalize_round is called.
pub fn emit_finalize_round(env: &Env, caller: &Address) {
    let event = AiAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AiAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "finalize_round"),
        },
    };
    env.events()
        .publish((symbol_short!("AIANA"), symbol_short!("FINALIZE_")), event);
}
