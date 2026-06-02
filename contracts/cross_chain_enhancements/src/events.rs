//! # CrossChainEnhancements Events Module
//!
//! Standardized event emissions for the cross_chain_enhancements contract.
//! Topic naming convention: (XCEN, ACTION)

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
pub struct CrossChainEnhancementsEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainEnhancementsEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: CrossChainEnhancementsEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = CrossChainEnhancementsEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainEnhancementsEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("XCEN"), symbol_short!("INIT")), event);
}

/// Emitted when submit_zk_ownership_proof is called.
pub fn emit_submit_zk_ownership_proof(env: &Env, caller: &Address) {
    let event = CrossChainEnhancementsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainEnhancementsEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_zk_ownership_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("XCEN"), symbol_short!("SUBMIT_ZK")), event);
}

/// Emitted when verify_zk_ownership_proof is called.
pub fn emit_verify_zk_ownership_proof(env: &Env, caller: &Address) {
    let event = CrossChainEnhancementsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainEnhancementsEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_zk_ownership_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("XCEN"), symbol_short!("VERIFY_ZK")), event);
}

/// Emitted when create_data_integrity_proof is called.
pub fn emit_create_data_integrity_proof(env: &Env, caller: &Address) {
    let event = CrossChainEnhancementsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainEnhancementsEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_data_integrity_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("XCEN"), symbol_short!("CREATE_DA")), event);
}

/// Emitted when set_rate_limit is called.
pub fn emit_set_rate_limit(env: &Env, caller: &Address) {
    let event = CrossChainEnhancementsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainEnhancementsEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_rate_limit"),
        },
    };
    env.events()
        .publish((symbol_short!("XCEN"), symbol_short!("SET_RATE_")), event);
}
