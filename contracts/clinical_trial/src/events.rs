//! # ClinicalTrial Events Module
//!
//! Standardized event emissions for the clinical_trial contract.
//! Topic naming convention: (CLTRL, ACTION)

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
pub struct ClinicalTrialEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ClinicalTrialEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ClinicalTrialEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ClinicalTrialEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalTrialEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CLTRL"), symbol_short!("INIT")), event);
}

/// Emitted when create_protocol is called.
pub fn emit_create_protocol(env: &Env, caller: &Address) {
    let event = ClinicalTrialEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalTrialEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_protocol"),
        },
    };
    env.events()
        .publish((symbol_short!("CLTRL"), symbol_short!("CREATE_PR")), event);
}

/// Emitted when register_site is called.
pub fn emit_register_site(env: &Env, caller: &Address) {
    let event = ClinicalTrialEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalTrialEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_site"),
        },
    };
    env.events()
        .publish((symbol_short!("CLTRL"), symbol_short!("REGISTER_")), event);
}

/// Emitted when recruit_patient is called.
pub fn emit_recruit_patient(env: &Env, caller: &Address) {
    let event = ClinicalTrialEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalTrialEventData {
            user: caller.clone(),
            action: String::from_str(env, "recruit_patient"),
        },
    };
    env.events()
        .publish((symbol_short!("CLTRL"), symbol_short!("RECRUIT_P")), event);
}

/// Emitted when record_consent is called.
pub fn emit_record_consent(env: &Env, caller: &Address) {
    let event = ClinicalTrialEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalTrialEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("CLTRL"), symbol_short!("RECORD_CO")), event);
}

/// Emitted when report_adverse_event is called.
pub fn emit_report_adverse_event(env: &Env, caller: &Address) {
    let event = ClinicalTrialEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ClinicalTrialEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_adverse_event"),
        },
    };
    env.events()
        .publish((symbol_short!("CLTRL"), symbol_short!("REPORT_AD")), event);
}
