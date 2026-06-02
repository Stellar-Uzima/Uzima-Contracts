//! # ContractVerification Events Module
//!
//! Standardized event emissions for the contract_verification contract.
//! Topic naming convention: (VERIF, ACTION)

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
pub struct ContractVerificationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ContractVerificationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ContractVerificationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ContractVerificationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractVerificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("VERIF"), symbol_short!("INIT")), event);
}

/// Emitted when publish_metadata is called.
pub fn emit_publish_metadata(env: &Env, caller: &Address) {
    let event = ContractVerificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractVerificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "publish_metadata"),
        },
    };
    env.events()
        .publish((symbol_short!("VERIF"), symbol_short!("PUBLISH_M")), event);
}

/// Emitted when publish_build_info is called.
pub fn emit_publish_build_info(env: &Env, caller: &Address) {
    let event = ContractVerificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractVerificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "publish_build_info"),
        },
    };
    env.events()
        .publish((symbol_short!("VERIF"), symbol_short!("PUBLISH_B")), event);
}

/// Emitted when publish_abi is called.
pub fn emit_publish_abi(env: &Env, caller: &Address) {
    let event = ContractVerificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractVerificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "publish_abi"),
        },
    };
    env.events()
        .publish((symbol_short!("VERIF"), symbol_short!("PUBLISH_A")), event);
}

/// Emitted when mark_verified is called.
pub fn emit_mark_verified(env: &Env, caller: &Address) {
    let event = ContractVerificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractVerificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_verified"),
        },
    };
    env.events()
        .publish((symbol_short!("VERIF"), symbol_short!("MARK_VERI")), event);
}
