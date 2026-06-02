//! # DrugDiscovery Events Module
//!
//! Standardized event emissions for the drug_discovery contract.
//! Topic naming convention: (DRUGD, ACTION)

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
pub struct DrugDiscoveryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DrugDiscoveryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: DrugDiscoveryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("INIT")), event);
}

/// Emitted when configure_integrations is called.
pub fn emit_configure_integrations(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_integrations"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when register_molecule is called.
pub fn emit_register_molecule(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_molecule"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("REGISTER_")), event);
}

/// Emitted when analyze_molecular_structure is called.
pub fn emit_analyze_molecular_structure(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "analyze_molecular_structure"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("ANALYZE_M")), event);
}

/// Emitted when predict_drug_target_interaction is called.
pub fn emit_predict_drug_target_interaction(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "predict_drug_target_interaction"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("PREDICT_D")), event);
}

/// Emitted when predict_adverse_effects is called.
pub fn emit_predict_adverse_effects(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "predict_adverse_effects"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("PREDICT_A")), event);
}

/// Emitted when optimize_clinical_trial_matching is called.
pub fn emit_optimize_clinical_trial_matching(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "optimize_clinical_trial_matching"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("OPTIMIZE_")), event);
}

/// Emitted when request_quantum_simulation is called.
pub fn emit_request_quantum_simulation(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_quantum_simulation"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("REQUEST_Q")), event);
}

/// Emitted when run_screening_campaign is called.
pub fn emit_run_screening_campaign(env: &Env, caller: &Address) {
    let event = DrugDiscoveryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DrugDiscoveryEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_screening_campaign"),
        },
    };
    env.events()
        .publish((symbol_short!("DRUGD"), symbol_short!("RUN_SCREE")), event);
}
