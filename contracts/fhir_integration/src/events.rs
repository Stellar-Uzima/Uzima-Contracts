//! # FhirIntegration Events Module
//!
//! Standardized event emissions for the fhir_integration contract.
//! Topic naming convention: (FHIR, ACTION)

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
pub struct FhirIntegrationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct FhirIntegrationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: FhirIntegrationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("INIT")), event);
}

/// Emitted when register_provider is called.
pub fn emit_register_provider(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_provider"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when verify_provider is called.
pub fn emit_verify_provider(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_provider"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("VERIFY_PR")), event);
}

/// Emitted when configure_emr is called.
pub fn emit_configure_emr(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_emr"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when store_observation is called.
pub fn emit_store_observation(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "store_observation"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("STORE_OBS")), event);
}

/// Emitted when store_condition is called.
pub fn emit_store_condition(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "store_condition"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("STORE_CON")), event);
}

/// Emitted when store_medication is called.
pub fn emit_store_medication(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "store_medication"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("STORE_MED")), event);
}

/// Emitted when store_procedure is called.
pub fn emit_store_procedure(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "store_procedure"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("STORE_PRO")), event);
}

/// Emitted when store_allergy is called.
pub fn emit_store_allergy(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "store_allergy"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("STORE_ALL")), event);
}

/// Emitted when register_data_mapping is called.
pub fn emit_register_data_mapping(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_data_mapping"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("PAUSE")), event);
}

/// Emitted when resume is called.
pub fn emit_resume(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "resume"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("RESUME")), event);
}

/// Emitted when export_patient_data is called.
pub fn emit_export_patient_data(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "export_patient_data"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("EXPORT_PA")), event);
}

/// Emitted when configure_export is called.
pub fn emit_configure_export(env: &Env, caller: &Address) {
    let event = FhirIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FhirIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_export"),
        },
    };
    env.events()
        .publish((symbol_short!("FHIR"), symbol_short!("CONFIGURE")), event);
}
