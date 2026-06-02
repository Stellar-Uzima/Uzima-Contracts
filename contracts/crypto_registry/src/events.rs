//! # CryptoRegistry Events Module
//!
//! Standardized event emissions for the crypto_registry contract.
//! Topic naming convention: (CRYREG, ACTION)

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
pub struct CryptoRegistryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CryptoRegistryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: CryptoRegistryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = CryptoRegistryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CryptoRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CRYREG"), symbol_short!("INIT")), event);
}

/// Emitted when register_key_bundle is called.
pub fn emit_register_key_bundle(env: &Env, caller: &Address) {
    let event = CryptoRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CryptoRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_key_bundle"),
        },
    };
    env.events()
        .publish((symbol_short!("CRYREG"), symbol_short!("REGISTER_")), event);
}

/// Emitted when revoke_key_bundle is called.
pub fn emit_revoke_key_bundle(env: &Env, caller: &Address) {
    let event = CryptoRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CryptoRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_key_bundle"),
        },
    };
    env.events()
        .publish((symbol_short!("CRYREG"), symbol_short!("REVOKE_KE")), event);
}

/// Emitted when rotate_key is called.
pub fn emit_rotate_key(env: &Env, caller: &Address) {
    let event = CryptoRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CryptoRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "rotate_key"),
        },
    };
    env.events()
        .publish((symbol_short!("CRYREG"), symbol_short!("ROTATE_KE")), event);
}
