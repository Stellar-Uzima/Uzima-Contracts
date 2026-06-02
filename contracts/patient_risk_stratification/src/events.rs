//! # PatientRiskStratification Events Module
//!
//! Standardized event emissions for the patient_risk_stratification contract.
//! Topic naming convention: (PRISK, ACTION)

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
pub struct PatientRiskStratificationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientRiskStratificationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PatientRiskStratificationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PatientRiskStratificationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientRiskStratificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PRISK"), symbol_short!("INIT")), event);
}

/// Emitted when register_risk_model is called.
pub fn emit_register_risk_model(env: &Env, caller: &Address) {
    let event = PatientRiskStratificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientRiskStratificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_risk_model"),
        },
    };
    env.events()
        .publish((symbol_short!("PRISK"), symbol_short!("REGISTER_")), event);
}

/// Emitted when perform_risk_assessment is called.
pub fn emit_perform_risk_assessment(env: &Env, caller: &Address) {
    let event = PatientRiskStratificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientRiskStratificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "perform_risk_assessment"),
        },
    };
    env.events()
        .publish((symbol_short!("PRISK"), symbol_short!("PERFORM_R")), event);
}

/// Emitted when update_model_status is called.
pub fn emit_update_model_status(env: &Env, caller: &Address) {
    let event = PatientRiskStratificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientRiskStratificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_model_status"),
        },
    };
    env.events()
        .publish((symbol_short!("PRISK"), symbol_short!("UPDATE_MO")), event);
}
