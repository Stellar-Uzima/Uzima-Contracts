//! # HealthcareDataConversion Events Module
//!
//! Standardized event emissions for the healthcare_data_conversion contract.
//! Topic naming convention: (HDCON, ACTION)

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
pub struct HealthcareDataConversionEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareDataConversionEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcareDataConversionEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("INIT")), event);
}

/// Emitted when register_conversion_rule is called.
pub fn emit_register_conversion_rule(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_conversion_rule"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_coding_mapping is called.
pub fn emit_register_coding_mapping(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_coding_mapping"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("REGISTER_")), event);
}

/// Emitted when find_coding_mapping is called.
pub fn emit_find_coding_mapping(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "find_coding_mapping"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("FIND_CODI")), event);
}

/// Emitted when register_format_specification is called.
pub fn emit_register_format_specification(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_format_specification"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("REGISTER_")), event);
}

/// Emitted when validate_conversion is called.
pub fn emit_validate_conversion(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "validate_conversion"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("VALIDATE_")), event);
}

/// Emitted when record_conversion is called.
pub fn emit_record_conversion(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_conversion"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("RECORD_CO")), event);
}

/// Emitted when record_lossy_conversion_warning is called.
pub fn emit_record_lossy_conversion_warning(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_lossy_conversion_warning"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("RECORD_LO")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("PAUSE")), event);
}

/// Emitted when resume is called.
pub fn emit_resume(env: &Env, caller: &Address) {
    let event = HealthcareDataConversionEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataConversionEventData {
            user: caller.clone(),
            action: String::from_str(env, "resume"),
        },
    };
    env.events()
        .publish((symbol_short!("HDCON"), symbol_short!("RESUME")), event);
}
