//! # ExplainableAi Events Module
//!
//! Standardized event emissions for the explainable_ai contract.
//! Topic naming convention: (XAI, ACTION)

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
pub struct ExplainableAiEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ExplainableAiEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ExplainableAiEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ExplainableAiEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ExplainableAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("XAI"), symbol_short!("INIT")), event);
}

/// Emitted when request_explanation is called.
pub fn emit_request_explanation(env: &Env, caller: &Address) {
    let event = ExplainableAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ExplainableAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_explanation"),
        },
    };
    env.events()
        .publish((symbol_short!("XAI"), symbol_short!("REQUEST_E")), event);
}

/// Emitted when fulfill_explanation_request is called.
pub fn emit_fulfill_explanation_request(env: &Env, caller: &Address) {
    let event = ExplainableAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ExplainableAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "fulfill_explanation_request"),
        },
    };
    env.events()
        .publish((symbol_short!("XAI"), symbol_short!("FULFILL_E")), event);
}

/// Emitted when submit_bias_audit is called.
pub fn emit_submit_bias_audit(env: &Env, caller: &Address) {
    let event = ExplainableAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ExplainableAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_bias_audit"),
        },
    };
    env.events()
        .publish((symbol_short!("XAI"), symbol_short!("SUBMIT_BI")), event);
}

/// Emitted when run_fairness_metrics is called.
pub fn emit_run_fairness_metrics(env: &Env, caller: &Address) {
    let event = ExplainableAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ExplainableAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_fairness_metrics"),
        },
    };
    env.events()
        .publish((symbol_short!("XAI"), symbol_short!("RUN_FAIRN")), event);
}

/// Emitted when generate_shap_explanation is called.
pub fn emit_generate_shap_explanation(env: &Env, caller: &Address) {
    let event = ExplainableAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ExplainableAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_shap_explanation"),
        },
    };
    env.events()
        .publish((symbol_short!("XAI"), symbol_short!("GENERATE_")), event);
}

/// Emitted when generate_counterfactual is called.
pub fn emit_generate_counterfactual(env: &Env, caller: &Address) {
    let event = ExplainableAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ExplainableAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_counterfactual"),
        },
    };
    env.events()
        .publish((symbol_short!("XAI"), symbol_short!("GENERATE_")), event);
}
