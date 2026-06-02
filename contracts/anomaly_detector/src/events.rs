//! # AnomalyDetector Events Module
//!
//! Standardized event emissions for the anomaly_detector contract.
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
pub struct AnomalyDetectorEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AnomalyDetectorEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AnomalyDetectorEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("INIT")), event);
}

/// Emitted when add_validator is called.
pub fn emit_add_validator(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_validator"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("ADD_VALID")), event);
}

/// Emitted when remove_validator is called.
pub fn emit_remove_validator(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_validator"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("REMOVE_VA")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when update_threshold is called.
pub fn emit_update_threshold(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_threshold"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("UPDATE_TH")), event);
}

/// Emitted when clear_alerts is called.
pub fn emit_clear_alerts(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "clear_alerts"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("CLEAR_ALE")), event);
}

/// Emitted when register_model is called.
pub fn emit_register_model(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_model"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("REGISTER_")), event);
}

/// Emitted when update_model_weight is called.
pub fn emit_update_model_weight(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_model_weight"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("UPDATE_MO")), event);
}

/// Emitted when run_inference is called.
pub fn emit_run_inference(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_inference"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("RUN_INFER")), event);
}

/// Emitted when detect_prescription_anomaly is called.
pub fn emit_detect_prescription_anomaly(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "detect_prescription_anomaly"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("DETECT_PR")), event);
}

/// Emitted when detect_access_anomaly is called.
pub fn emit_detect_access_anomaly(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "detect_access_anomaly"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("DETECT_AC")), event);
}

/// Emitted when create_alert is called.
pub fn emit_create_alert(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("CREATE_AL")), event);
}

/// Emitted when acknowledge_alert is called.
pub fn emit_acknowledge_alert(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "acknowledge_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("ACKNOWLED")), event);
}

/// Emitted when resolve_alert is called.
pub fn emit_resolve_alert(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("RESOLVE_A")), event);
}

/// Emitted when mark_false_positive is called.
pub fn emit_mark_false_positive(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_false_positive"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("MARK_FALS")), event);
}

/// Emitted when submit_feedback is called.
pub fn emit_submit_feedback(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_feedback"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("SUBMIT_FE")), event);
}

/// Emitted when submit_federated_update is called.
pub fn emit_submit_federated_update(env: &Env, caller: &Address) {
    let event = AnomalyDetectorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AnomalyDetectorEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_federated_update"),
        },
    };
    env.events()
        .publish((symbol_short!("ANMDT"), symbol_short!("SUBMIT_FE")), event);
}
