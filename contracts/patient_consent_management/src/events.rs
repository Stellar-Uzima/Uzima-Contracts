//! # PatientConsentManagement Events Module
//!
//! Standardized event emissions for the patient_consent_management contract.
//! Topic naming convention: (CONS, ACTION)

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
pub struct PatientConsentManagementEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientConsentManagementEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PatientConsentManagementEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("INIT")), event);
}

/// Emitted when grant_consent is called.
pub fn emit_grant_consent(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("GRANT_CON")), event);
}

/// Emitted when grant_consent_with_expiry is called.
pub fn emit_grant_consent_with_expiry(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_consent_with_expiry"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("GRANT_CON")), event);
}

/// Emitted when batch_grant_consent is called.
pub fn emit_batch_grant_consent(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "batch_grant_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("BATCH_GRA")), event);
}

/// Emitted when revoke_consent is called.
pub fn emit_revoke_consent(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("REVOKE_CO")), event);
}

/// Emitted when cleanup_expired_consents is called.
pub fn emit_cleanup_expired_consents(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_expired_consents"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("CLEANUP_E")), event);
}

/// Emitted when verify_consent_with_audit is called.
pub fn emit_verify_consent_with_audit(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_consent_with_audit"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("VERIFY_CO")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when health_check is called.
pub fn emit_health_check(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "health_check"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("HEALTH_CH")), event);
}

/// Emitted when designate_proxy is called.
pub fn emit_designate_proxy(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "designate_proxy"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("DESIGNATE")), event);
}

/// Emitted when revoke_proxy is called.
pub fn emit_revoke_proxy(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_proxy"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("REVOKE_PR")), event);
}

/// Emitted when proxy_grant_consent is called.
pub fn emit_proxy_grant_consent(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "proxy_grant_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("PROXY_GRA")), event);
}

/// Emitted when proxy_revoke_consent is called.
pub fn emit_proxy_revoke_consent(env: &Env, caller: &Address) {
    let event = PatientConsentManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientConsentManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "proxy_revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("CONS"), symbol_short!("PROXY_REV")), event);
}
