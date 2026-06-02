//! # DigitalTwin Events Module
//!
//! Standardized event emissions for the digital_twin contract.
//! Topic naming convention: (DTWIN, ACTION)

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
pub struct DigitalTwinEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DigitalTwinEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: DigitalTwinEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("INIT")), event);
}

/// Emitted when set_medical_records_contract is called.
pub fn emit_set_medical_records_contract(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_medical_records_contract"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("SET_MEDIC")), event);
}

/// Emitted when set_genomic_data_contract is called.
pub fn emit_set_genomic_data_contract(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_genomic_data_contract"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("SET_GENOM")), event);
}

/// Emitted when create_digital_twin is called.
pub fn emit_create_digital_twin(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_digital_twin"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("CREATE_DI")), event);
}

/// Emitted when update_digital_twin_status is called.
pub fn emit_update_digital_twin_status(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_digital_twin_status"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("UPDATE_DI")), event);
}

/// Emitted when add_data_stream is called.
pub fn emit_add_data_stream(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_data_stream"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("ADD_DATA_")), event);
}

/// Emitted when add_data_point is called.
pub fn emit_add_data_point(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_data_point"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("ADD_DATA_")), event);
}

/// Emitted when add_predictive_model is called.
pub fn emit_add_predictive_model(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_predictive_model"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("ADD_PREDI")), event);
}

/// Emitted when generate_prediction is called.
pub fn emit_generate_prediction(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_prediction"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("GENERATE_")), event);
}

/// Emitted when create_simulation is called.
pub fn emit_create_simulation(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_simulation"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("CREATE_SI")), event);
}

/// Emitted when complete_simulation is called.
pub fn emit_complete_simulation(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "complete_simulation"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("COMPLETE_")), event);
}

/// Emitted when create_research_snapshot is called.
pub fn emit_create_research_snapshot(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_research_snapshot"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("CREATE_RE")), event);
}

/// Emitted when sync_with_medical_records is called.
pub fn emit_sync_with_medical_records(env: &Env, caller: &Address) {
    let event = DigitalTwinEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DigitalTwinEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_with_medical_records"),
        },
    };
    env.events()
        .publish((symbol_short!("DTWIN"), symbol_short!("SYNC_WITH")), event);
}
