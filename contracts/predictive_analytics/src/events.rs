//! # PredictiveAnalytics Events Module
//!
//! Standardized event emissions for the predictive_analytics contract.
//! Topic naming convention: (PREDAI, ACTION)

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
pub struct PredictiveAnalyticsEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PredictiveAnalyticsEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PredictiveAnalyticsEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PredictiveAnalyticsEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PredictiveAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PREDAI"), symbol_short!("INIT")), event);
}

/// Emitted when update_config is called.
pub fn emit_update_config(env: &Env, caller: &Address) {
    let event = PredictiveAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PredictiveAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_config"),
        },
    };
    env.events()
        .publish((symbol_short!("PREDAI"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when make_prediction is called.
pub fn emit_make_prediction(env: &Env, caller: &Address) {
    let event = PredictiveAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PredictiveAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "make_prediction"),
        },
    };
    env.events()
        .publish((symbol_short!("PREDAI"), symbol_short!("MAKE_PRED")), event);
}

/// Emitted when update_model_metrics is called.
pub fn emit_update_model_metrics(env: &Env, caller: &Address) {
    let event = PredictiveAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PredictiveAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_model_metrics"),
        },
    };
    env.events()
        .publish((symbol_short!("PREDAI"), symbol_short!("UPDATE_MO")), event);
}

/// Emitted when whitelist_predictor is called.
pub fn emit_whitelist_predictor(env: &Env, caller: &Address) {
    let event = PredictiveAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PredictiveAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "whitelist_predictor"),
        },
    };
    env.events()
        .publish((symbol_short!("PREDAI"), symbol_short!("WHITELIST")), event);
}
