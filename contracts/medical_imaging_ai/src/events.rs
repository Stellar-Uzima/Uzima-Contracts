//! # MedicalImagingAi Events Module
//!
//! Standardized event emissions for the medical_imaging_ai contract.
//! Topic naming convention: (IMGAI, ACTION)

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
pub struct MedicalImagingAiEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicalImagingAiEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MedicalImagingAiEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("INIT")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when register_evaluator is called.
pub fn emit_register_evaluator(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_evaluator"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("REGISTER_")), event);
}

/// Emitted when revoke_evaluator is called.
pub fn emit_revoke_evaluator(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_evaluator"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("REVOKE_EV")), event);
}

/// Emitted when register_cnn_model is called.
pub fn emit_register_cnn_model(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_cnn_model"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("REGISTER_")), event);
}

/// Emitted when update_model_status is called.
pub fn emit_update_model_status(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_model_status"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("UPDATE_MO")), event);
}

/// Emitted when submit_analysis is called.
pub fn emit_submit_analysis(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_analysis"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("SUBMIT_AN")), event);
}

/// Emitted when submit_segmentation is called.
pub fn emit_submit_segmentation(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_segmentation"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("SUBMIT_SE")), event);
}

/// Emitted when record_evaluation is called.
pub fn emit_record_evaluation(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_evaluation"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("RECORD_EV")), event);
}

/// Emitted when configure_thresholds is called.
pub fn emit_configure_thresholds(env: &Env, caller: &Address) {
    let event = MedicalImagingAiEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingAiEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_thresholds"),
        },
    };
    env.events()
        .publish((symbol_short!("IMGAI"), symbol_short!("CONFIGURE")), event);
}
