//! # HealthcareCompliance Events Module
//!
//! Standardized event emissions for the healthcare_compliance contract.
//! Topic naming convention: (HCCOMP, ACTION)

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
pub struct HealthcareComplianceEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareComplianceEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcareComplianceEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("INIT")), event);
}

/// Emitted when health_check is called.
pub fn emit_health_check(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "health_check"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("HEALTH_CH")), event);
}

/// Emitted when update_config is called.
pub fn emit_update_config(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_config"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when grant_consent is called.
pub fn emit_grant_consent(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("GRANT_CON")), event);
}

/// Emitted when revoke_consent is called.
pub fn emit_revoke_consent(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("REVOKE_CO")), event);
}

/// Emitted when log_audit_event is called.
pub fn emit_log_audit_event(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_audit_event"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("LOG_AUDIT")), event);
}

/// Emitted when report_breach is called.
pub fn emit_report_breach(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_breach"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("REPORT_BR")), event);
}

/// Emitted when register_retention_record is called.
pub fn emit_register_retention_record(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_retention_record"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("REGISTER_")), event);
}

/// Emitted when set_retention_policy is called.
pub fn emit_set_retention_policy(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_retention_policy"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("SET_RETEN")), event);
}

/// Emitted when request_data_deletion is called.
pub fn emit_request_data_deletion(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_data_deletion"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("REQUEST_D")), event);
}

/// Emitted when enforce_retention is called.
pub fn emit_enforce_retention(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "enforce_retention"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("ENFORCE_R")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("PAUSE")), event);
}

/// Emitted when resume is called.
pub fn emit_resume(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "resume"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("RESUME")), event);
}

/// Emitted when submit_compliance_report is called.
pub fn emit_submit_compliance_report(env: &Env, caller: &Address) {
    let event = HealthcareComplianceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareComplianceEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_compliance_report"),
        },
    };
    env.events()
        .publish((symbol_short!("HCCOMP"), symbol_short!("SUBMIT_CO")), event);
}
