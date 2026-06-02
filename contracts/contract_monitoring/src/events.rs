//! # ContractMonitoring Events Module
//!
//! Standardized event emissions for the contract_monitoring contract.
//! Topic naming convention: (MONIT, ACTION)

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
pub struct ContractMonitoringEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ContractMonitoringEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ContractMonitoringEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ContractMonitoringEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MONIT"), symbol_short!("INIT")), event);
}

/// Emitted when record_call is called.
pub fn emit_record_call(env: &Env, caller: &Address) {
    let event = ContractMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_call"),
        },
    };
    env.events()
        .publish((symbol_short!("MONIT"), symbol_short!("RECORD_CA")), event);
}

/// Emitted when record_error is called.
pub fn emit_record_error(env: &Env, caller: &Address) {
    let event = ContractMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_error"),
        },
    };
    env.events()
        .publish((symbol_short!("MONIT"), symbol_short!("RECORD_ER")), event);
}

/// Emitted when update_storage_count is called.
pub fn emit_update_storage_count(env: &Env, caller: &Address) {
    let event = ContractMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_storage_count"),
        },
    };
    env.events()
        .publish((symbol_short!("MONIT"), symbol_short!("UPDATE_ST")), event);
}

/// Emitted when update_alert_config is called.
pub fn emit_update_alert_config(env: &Env, caller: &Address) {
    let event = ContractMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_alert_config"),
        },
    };
    env.events()
        .publish((symbol_short!("MONIT"), symbol_short!("UPDATE_AL")), event);
}

/// Emitted when version is called.
pub fn emit_version(env: &Env, caller: &Address) {
    let event = ContractMonitoringEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractMonitoringEventData {
            user: caller.clone(),
            action: String::from_str(env, "version"),
        },
    };
    env.events()
        .publish((symbol_short!("MONIT"), symbol_short!("VERSION")), event);
}
