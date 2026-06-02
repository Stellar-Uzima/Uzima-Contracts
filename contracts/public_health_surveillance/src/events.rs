//! # PublicHealthSurveillance Events Module
//!
//! Standardized event emissions for the public_health_surveillance contract.
//! Topic naming convention: (PHS, ACTION)

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
pub struct PublicHealthSurveillanceEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PublicHealthSurveillanceEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PublicHealthSurveillanceEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("INIT")), event);
}

/// Emitted when report_outbreak_data is called.
pub fn emit_report_outbreak_data(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_outbreak_data"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("REPORT_OU")), event);
}

/// Emitted when create_epidemic_model is called.
pub fn emit_create_epidemic_model(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_epidemic_model"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("CREATE_EP")), event);
}

/// Emitted when create_public_health_alert is called.
pub fn emit_create_public_health_alert(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_public_health_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("CREATE_PU")), event);
}

/// Emitted when report_vaccination_coverage is called.
pub fn emit_report_vaccination_coverage(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_vaccination_coverage"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("REPORT_VA")), event);
}

/// Emitted when report_environmental_health is called.
pub fn emit_report_environmental_health(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_environmental_health"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("REPORT_EN")), event);
}

/// Emitted when report_antimicrobial_resistance is called.
pub fn emit_report_antimicrobial_resistance(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_antimicrobial_resistance"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("REPORT_AN")), event);
}

/// Emitted when report_social_determinants is called.
pub fn emit_report_social_determinants(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_social_determinants"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("REPORT_SO")), event);
}

/// Emitted when create_intervention is called.
pub fn emit_create_intervention(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_intervention"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("CREATE_IN")), event);
}

/// Emitted when create_global_collaboration is called.
pub fn emit_create_global_collaboration(env: &Env, caller: &Address) {
    let event = PublicHealthSurveillanceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PublicHealthSurveillanceEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_global_collaboration"),
        },
    };
    env.events()
        .publish((symbol_short!("PHS"), symbol_short!("CREATE_GL")), event);
}
