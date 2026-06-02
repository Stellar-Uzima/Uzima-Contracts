//! # ContractTemplate Events Module
//!
//! Standardized event emissions for the contract_template contract.
//! Topic naming convention: (CT, ACTION)

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
pub struct ContractTemplateEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ContractTemplateEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ContractTemplateEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ContractTemplateEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractTemplateEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CT"), symbol_short!("INIT")), event);
}

/// Emitted when transfer_admin is called.
pub fn emit_transfer_admin(env: &Env, caller: &Address) {
    let event = ContractTemplateEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractTemplateEventData {
            user: caller.clone(),
            action: String::from_str(env, "transfer_admin"),
        },
    };
    env.events()
        .publish((symbol_short!("CT"), symbol_short!("TRANSFER_")), event);
}

/// Emitted when update_data is called.
pub fn emit_update_data(env: &Env, caller: &Address) {
    let event = ContractTemplateEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractTemplateEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_data"),
        },
    };
    env.events()
        .publish((symbol_short!("CT"), symbol_short!("UPDATE_DA")), event);
}
