//! # RemotePatientMonitoring Events Module
//!
//! Standardized event emissions for the remote_patient_monitoring contract.
//! Topic naming convention: (RPM, ACTION)

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
pub struct RemotePatientMonitoringEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct RemotePatientMonitoringEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: RemotePatientMonitoringEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = RemotePatientMonitoringEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RemotePatientMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("RPM"), symbol_short!("INIT")), event);
}

/// Emitted when register_device is called.
pub fn emit_register_device(env: &Env, caller: &Address) {
    let event = RemotePatientMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RemotePatientMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_device"),
        },
    };
    env.events()
        .publish((symbol_short!("RPM"), symbol_short!("REGISTER_")), event);
}

/// Emitted when add_caregiver is called.
pub fn emit_add_caregiver(env: &Env, caller: &Address) {
    let event = RemotePatientMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RemotePatientMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_caregiver"),
        },
    };
    env.events()
        .publish((symbol_short!("RPM"), symbol_short!("ADD_CAREG")), event);
}

/// Emitted when submit_vital_sign is called.
pub fn emit_submit_vital_sign(env: &Env, caller: &Address) {
    let event = RemotePatientMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RemotePatientMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_vital_sign"),
        },
    };
    env.events()
        .publish((symbol_short!("RPM"), symbol_short!("SUBMIT_VI")), event);
}

/// Emitted when set_threshold is called.
pub fn emit_set_threshold(env: &Env, caller: &Address) {
    let event = RemotePatientMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RemotePatientMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_threshold"),
        },
    };
    env.events()
        .publish((symbol_short!("RPM"), symbol_short!("SET_THRES")), event);
}

/// Emitted when update_battery_level is called.
pub fn emit_update_battery_level(env: &Env, caller: &Address) {
    let event = RemotePatientMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: RemotePatientMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_battery_level"),
        },
    };
    env.events()
        .publish((symbol_short!("RPM"), symbol_short!("UPDATE_BA")), event);
}
