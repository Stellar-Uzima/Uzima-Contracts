//! # RegulatoryCompliance Events Module
//!
//! Standardized event emissions for the regulatory_compliance contract.
//! Topic naming convention: (REGCOM, ACTION)

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
pub struct RegulatoryComplianceEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct RegulatoryComplianceEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: RegulatoryComplianceEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = RegulatoryComplianceEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegulatoryComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("REGCOM"), symbol_short!("INIT")), event);
}

/// Emitted when set_rule is called.
pub fn emit_set_rule(env: &Env, caller: &Address) {
    let event = RegulatoryComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegulatoryComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_rule"),
        },
    };
    env.events()
        .publish((symbol_short!("REGCOM"), symbol_short!("SET_RULE")), event);
}

/// Emitted when grant_consent is called.
pub fn emit_grant_consent(env: &Env, caller: &Address) {
    let event = RegulatoryComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegulatoryComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("REGCOM"), symbol_short!("GRANT_CON")), event);
}

/// Emitted when revoke_consent is called.
pub fn emit_revoke_consent(env: &Env, caller: &Address) {
    let event = RegulatoryComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegulatoryComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("REGCOM"), symbol_short!("REVOKE_CO")), event);
}

/// Emitted when log_audit is called.
pub fn emit_log_audit(env: &Env, caller: &Address) {
    let event = RegulatoryComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegulatoryComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_audit"),
        },
    };
    env.events()
        .publish((symbol_short!("REGCOM"), symbol_short!("LOG_AUDIT")), event);
}

/// Emitted when invoke_right_to_be_forgotten is called.
pub fn emit_invoke_right_to_be_forgotten(env: &Env, caller: &Address) {
    let event = RegulatoryComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RegulatoryComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "invoke_right_to_be_forgotten"),
        },
    };
    env.events()
        .publish((symbol_short!("REGCOM"), symbol_short!("INVOKE_RI")), event);
}
