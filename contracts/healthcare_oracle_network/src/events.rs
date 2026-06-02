//! # HealthcareOracleNetwork Events Module
//!
//! Standardized event emissions for the healthcare_oracle_network contract.
//! Topic naming convention: (HCORA, ACTION)

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
pub struct HealthcareOracleNetworkEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareOracleNetworkEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcareOracleNetworkEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("INIT")), event);
}

/// Emitted when register_oracle is called.
pub fn emit_register_oracle(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_oracle"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("REGISTER_")), event);
}

/// Emitted when verify_oracle is called.
pub fn emit_verify_oracle(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_oracle"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("VERIFY_OR")), event);
}

/// Emitted when update_oracle_endpoint is called.
pub fn emit_update_oracle_endpoint(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_oracle_endpoint"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("UPDATE_OR")), event);
}

/// Emitted when update_config is called.
pub fn emit_update_config(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_config"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when add_arbiter is called.
pub fn emit_add_arbiter(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_arbiter"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("ADD_ARBIT")), event);
}

/// Emitted when submit_drug_price is called.
pub fn emit_submit_drug_price(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_drug_price"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("SUBMIT_DR")), event);
}

/// Emitted when submit_clinical_trial is called.
pub fn emit_submit_clinical_trial(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_clinical_trial"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("SUBMIT_CL")), event);
}

/// Emitted when submit_regulatory_update is called.
pub fn emit_submit_regulatory_update(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_regulatory_update"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("SUBMIT_RE")), event);
}

/// Emitted when submit_treatment_outcome is called.
pub fn emit_submit_treatment_outcome(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_treatment_outcome"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("SUBMIT_TR")), event);
}

/// Emitted when finalize_feed is called.
pub fn emit_finalize_feed(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "finalize_feed"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("FINALIZE_")), event);
}

/// Emitted when raise_dispute is called.
pub fn emit_raise_dispute(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "raise_dispute"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("RAISE_DIS")), event);
}

/// Emitted when resolve_dispute is called.
pub fn emit_resolve_dispute(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_dispute"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("RESOLVE_D")), event);
}

/// Emitted when fetch_external_payload is called.
pub fn emit_fetch_external_payload(env: &Env, caller: &Address) {
    let event = HealthcareOracleNetworkEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareOracleNetworkEventData {
            user: caller.clone(),
            action: String::from_str(env, "fetch_external_payload"),
        },
    };
    env.events()
        .publish((symbol_short!("HCORA"), symbol_short!("FETCH_EXT")), event);
}
