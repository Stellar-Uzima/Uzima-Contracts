//! # ClinicalNlp Events Module
//!
//! Standardized event emissions for the clinical_nlp contract.
//! Topic naming convention: (CN, ACTION)

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
pub struct ClinicalNlpEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ClinicalNlpEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ClinicalNlpEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("INIT")), event);
}

/// Emitted when process_clinical_note is called.
pub fn emit_process_clinical_note(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "process_clinical_note"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("PROCESS_C")), event);
}

/// Emitted when extract_entities is called.
pub fn emit_extract_entities(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "extract_entities"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("EXTRACT_E")), event);
}

/// Emitted when analyze_sentiment is called.
pub fn emit_analyze_sentiment(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "analyze_sentiment"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("ANALYZE_S")), event);
}

/// Emitted when generate_coding_suggestions is called.
pub fn emit_generate_coding_suggestions(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_coding_suggestions"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("GENERATE_")), event);
}

/// Emitted when update_config is called.
pub fn emit_update_config(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_config"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when process_batch is called.
pub fn emit_process_batch(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "process_batch"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("PROCESS_B")), event);
}

/// Emitted when version is called.
pub fn emit_version(env: &Env, caller: &Address) {
    let event = ClinicalNlpEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalNlpEventData {
            user: caller.clone(),
            action: String::from_str(env, "version"),
        },
    };
    env.events()
        .publish((symbol_short!("CN"), symbol_short!("VERSION")), event);
}
