//! # PatientPortal Events Module
//!
//! Standardized event emissions for the patient_portal contract.
//! Topic naming convention: (PPORT, ACTION)

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
pub struct PatientPortalEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientPortalEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PatientPortalEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("INIT")), event);
}

/// Emitted when set_integration_contracts is called.
pub fn emit_set_integration_contracts(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_integration_contracts"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("SET_INTEG")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when register is called.
pub fn emit_register(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "register"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("REGISTER")), event);
}

/// Emitted when request_phr_export is called.
pub fn emit_request_phr_export(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_phr_export"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("REQUEST_P")), event);
}

/// Emitted when schedule_appointment is called.
pub fn emit_schedule_appointment(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "schedule_appointment"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("SCHEDULE_")), event);
}

/// Emitted when set_appointment_status is called.
pub fn emit_set_appointment_status(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_appointment_status"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("SET_APPOI")), event);
}

/// Emitted when list_my_appointment_ids is called.
pub fn emit_list_my_appointment_ids(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_my_appointment_ids"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("LIST_MY_A")), event);
}

/// Emitted when log_medication_event is called.
pub fn emit_log_medication_event(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_medication_event"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("LOG_MEDIC")), event);
}

/// Emitted when list_my_adherence_ids is called.
pub fn emit_list_my_adherence_ids(env: &Env, caller: &Address) {
    let event = PatientPortalEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientPortalEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_my_adherence_ids"),
        },
    };
    env.events()
        .publish((symbol_short!("PPORT"), symbol_short!("LIST_MY_A")), event);
}
