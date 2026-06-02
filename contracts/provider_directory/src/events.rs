//! # ProviderDirectory Events Module
//!
//! Standardized event emissions for the provider_directory contract.
//! Topic naming convention: (PRVDIR, ACTION)

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
pub struct ProviderDirectoryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ProviderDirectoryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ProviderDirectoryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ProviderDirectoryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ProviderDirectoryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PRVDIR"), symbol_short!("INIT")), event);
}

/// Emitted when set_rate_limit_config is called.
pub fn emit_set_rate_limit_config(env: &Env, caller: &Address) {
    let event = ProviderDirectoryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ProviderDirectoryEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_rate_limit_config"),
        },
    };
    env.events()
        .publish((symbol_short!("PRVDIR"), symbol_short!("SET_RATE_")), event);
}

/// Emitted when set_institution_exemption is called.
pub fn emit_set_institution_exemption(env: &Env, caller: &Address) {
    let event = ProviderDirectoryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ProviderDirectoryEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_institution_exemption"),
        },
    };
    env.events()
        .publish((symbol_short!("PRVDIR"), symbol_short!("SET_INSTI")), event);
}

/// Emitted when search_providers is called.
pub fn emit_search_providers(env: &Env, caller: &Address) {
    let event = ProviderDirectoryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ProviderDirectoryEventData {
            user: caller.clone(),
            action: String::from_str(env, "search_providers"),
        },
    };
    env.events()
        .publish((symbol_short!("PRVDIR"), symbol_short!("SEARCH_PR")), event);
}
