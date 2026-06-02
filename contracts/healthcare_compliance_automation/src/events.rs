//! # HealthcareComplianceAutomation Events Module
//!
//! Standardized event emissions for the healthcare_compliance_automation contract.
//! Topic naming convention: (HCAUTO, ACTION)

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
pub struct HealthcareComplianceAutomationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareComplianceAutomationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcareComplianceAutomationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthcareComplianceAutomationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceAutomationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HCAUTO"), symbol_short!("INIT")), event);
}

/// Emitted when add_framework is called.
pub fn emit_add_framework(env: &Env, caller: &Address) {
    let event = HealthcareComplianceAutomationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceAutomationEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_framework"),
        },
    };
    env.events()
        .publish((symbol_short!("HCAUTO"), symbol_short!("ADD_FRAME")), event);
}
