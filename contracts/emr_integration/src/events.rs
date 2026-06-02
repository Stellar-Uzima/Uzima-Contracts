//! # EmrIntegration Events Module
//!
//! Standardized event emissions for the emr_integration contract.
//! Topic naming convention: (EMR, ACTION)

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
pub struct EmrIntegrationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct EmrIntegrationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: EmrIntegrationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("INIT")), event);
}

/// Emitted when register_emr_system is called.
pub fn emit_register_emr_system(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_emr_system"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when initiate_onboarding is called.
pub fn emit_initiate_onboarding(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_onboarding"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("INITIATE_")), event);
}

/// Emitted when complete_onboarding is called.
pub fn emit_complete_onboarding(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "complete_onboarding"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("COMPLETE_")), event);
}

/// Emitted when register_network_node is called.
pub fn emit_register_network_node(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_network_node"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_interop_agreement is called.
pub fn emit_register_interop_agreement(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_interop_agreement"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when record_interop_test is called.
pub fn emit_record_interop_test(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_interop_test"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("RECORD_IN")), event);
}

/// Emitted when parse_message is called.
pub fn emit_parse_message(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "parse_message"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("PARSE_MES")), event);
}

/// Emitted when generate_message is called.
pub fn emit_generate_message(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_message"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("GENERATE_")), event);
}

/// Emitted when transform_message is called.
pub fn emit_transform_message(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "transform_message"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("TRANSFORM")), event);
}

/// Emitted when validate_message is called.
pub fn emit_validate_message(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "validate_message"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("VALIDATE_")), event);
}

/// Emitted when wrap_transport_payload is called.
pub fn emit_wrap_transport_payload(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "wrap_transport_payload"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("WRAP_TRAN")), event);
}

/// Emitted when benchmark_message_processing is called.
pub fn emit_benchmark_message_processing(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "benchmark_message_processing"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("BENCHMARK")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("PAUSE")), event);
}

/// Emitted when resume is called.
pub fn emit_resume(env: &Env, caller: &Address) {
    let event = EmrIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: EmrIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "resume"),
        },
    };
    env.events()
        .publish((symbol_short!("EMR"), symbol_short!("RESUME")), event);
}
