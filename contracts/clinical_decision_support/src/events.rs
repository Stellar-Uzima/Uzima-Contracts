//! # ClinicalDecisionSupport Events Module
//!
//! Standardized event emissions for the clinical_decision_support contract.
//! Topic naming convention: (CDSS, ACTION)

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
pub struct ClinicalDecisionSupportEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ClinicalDecisionSupportEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ClinicalDecisionSupportEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ClinicalDecisionSupportEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalDecisionSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CDSS"), symbol_short!("INIT")), event);
}

/// Emitted when optimize_pathway is called.
pub fn emit_optimize_pathway(env: &Env, caller: &Address) {
    let event = ClinicalDecisionSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalDecisionSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "optimize_pathway"),
        },
    };
    env.events()
        .publish((symbol_short!("CDSS"), symbol_short!("OPTIMIZE_")), event);
}

/// Emitted when record_outcome is called.
pub fn emit_record_outcome(env: &Env, caller: &Address) {
    let event = ClinicalDecisionSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalDecisionSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_outcome"),
        },
    };
    env.events()
        .publish((symbol_short!("CDSS"), symbol_short!("RECORD_OU")), event);
}

/// Emitted when update_guideline is called.
pub fn emit_update_guideline(env: &Env, caller: &Address) {
    let event = ClinicalDecisionSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalDecisionSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_guideline"),
        },
    };
    env.events()
        .publish((symbol_short!("CDSS"), symbol_short!("UPDATE_GU")), event);
}

/// Emitted when set_interaction is called.
pub fn emit_set_interaction(env: &Env, caller: &Address) {
    let event = ClinicalDecisionSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalDecisionSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_interaction"),
        },
    };
    env.events()
        .publish((symbol_short!("CDSS"), symbol_short!("SET_INTER")), event);
}
