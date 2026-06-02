//! # MedicationManagement Events Module
//!
//! Standardized event emissions for the medication_management contract.
//! Topic naming convention: (MEDMGM, ACTION)

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
pub struct MedicationManagementEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicationManagementEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MedicationManagementEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("INIT")), event);
}

/// Emitted when upsert_fda_medication is called.
pub fn emit_upsert_fda_medication(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "upsert_fda_medication"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("UPSERT_FD")), event);
}

/// Emitted when sync_fda_catalog is called.
pub fn emit_sync_fda_catalog(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_fda_catalog"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("SYNC_FDA_")), event);
}

/// Emitted when create_schedule is called.
pub fn emit_create_schedule(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_schedule"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("CREATE_SC")), event);
}

/// Emitted when update_schedule_status is called.
pub fn emit_update_schedule_status(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_schedule_status"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("UPDATE_SC")), event);
}

/// Emitted when register_interaction is called.
pub fn emit_register_interaction(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_interaction"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("REGISTER_")), event);
}

/// Emitted when update_interaction is called.
pub fn emit_update_interaction(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_interaction"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("UPDATE_IN")), event);
}

/// Emitted when resolve_interaction is called.
pub fn emit_resolve_interaction(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_interaction"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("RESOLVE_I")), event);
}

/// Emitted when record_dose is called.
pub fn emit_record_dose(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_dose"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("RECORD_DO")), event);
}

/// Emitted when process_refill is called.
pub fn emit_process_refill(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "process_refill"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("PROCESS_R")), event);
}

/// Emitted when trigger_auto_refill is called.
pub fn emit_trigger_auto_refill(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "trigger_auto_refill"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("TRIGGER_A")), event);
}

/// Emitted when generate_adherence_report is called.
pub fn emit_generate_adherence_report(env: &Env, caller: &Address) {
    let event = MedicationManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicationManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_adherence_report"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDMGM"), symbol_short!("GENERATE_")), event);
}
