//! # AnomalyDetection Events Module
//!
//! Standardized event emissions for the anomaly_detection contract.
//! Topic naming convention: (ANMDT, ACTION)

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
pub struct AnomalyDetectionEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AnomalyDetectionEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AnomalyDetectionEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("INIT")), event);
}

/// Emitted when update_config is called.
pub fn emit_update_config(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_config"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when set_audit_forensics is called.
pub fn emit_set_audit_forensics(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_audit_forensics"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("SET_AUDIT")), event);
}

/// Emitted when detect_anomaly is called.
pub fn emit_detect_anomaly(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "detect_anomaly"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("DETECT_AN")), event);
}

/// Emitted when whitelist_detector is called.
pub fn emit_whitelist_detector(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "whitelist_detector"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("WHITELIST")), event);
}

/// Emitted when create_alert is called.
pub fn emit_create_alert(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("CREATE_AL")), event);
}

/// Emitted when acknowledge_alert is called.
pub fn emit_acknowledge_alert(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "acknowledge_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("ACKNOWLED")), event);
}

/// Emitted when resolve_alert is called.
pub fn emit_resolve_alert(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("RESOLVE_A")), event);
}

/// Emitted when mark_false_positive is called.
pub fn emit_mark_false_positive(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_false_positive"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("MARK_FALS")), event);
}

/// Emitted when submit_feedback is called.
pub fn emit_submit_feedback(env: &Env, caller: &Address) {
    let event = AnomalyDetectionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectionEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_feedback"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("SUBMIT_FE")), event);
}
