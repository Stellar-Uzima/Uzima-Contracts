//! # HomomorphicRegistry Events Module
//!
//! Standardized event emissions for the homomorphic_registry contract.
//! Topic naming convention: (HOMO, ACTION)

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
pub struct HomomorphicRegistryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HomomorphicRegistryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HomomorphicRegistryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("INIT")), event);
}

/// Emitted when register_key_bundle is called.
pub fn emit_register_key_bundle(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_key_bundle"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("REGISTER_")), event);
}

/// Emitted when set_performance_profile is called.
pub fn emit_set_performance_profile(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_performance_profile"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("SET_PERFO")), event);
}

/// Emitted when encrypt_ckks_vector is called.
pub fn emit_encrypt_ckks_vector(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "encrypt_ckks_vector"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("ENCRYPT_C")), event);
}

/// Emitted when encrypt_bgv_vector is called.
pub fn emit_encrypt_bgv_vector(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "encrypt_bgv_vector"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("ENCRYPT_B")), event);
}

/// Emitted when fhe_add is called.
pub fn emit_fhe_add(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "fhe_add"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("FHE_ADD")), event);
}

/// Emitted when fhe_multiply is called.
pub fn emit_fhe_multiply(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "fhe_multiply"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("FHE_MULTI")), event);
}

/// Emitted when bootstrap_ciphertext is called.
pub fn emit_bootstrap_ciphertext(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "bootstrap_ciphertext"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("BOOTSTRAP")), event);
}

/// Emitted when encrypted_statistics is called.
pub fn emit_encrypted_statistics(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "encrypted_statistics"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("ENCRYPTED")), event);
}

/// Emitted when encrypted_linear_inference is called.
pub fn emit_encrypted_linear_inference(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "encrypted_linear_inference"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("ENCRYPTED")), event);
}

/// Emitted when estimate_operation_cost is called.
pub fn emit_estimate_operation_cost(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "estimate_operation_cost"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("ESTIMATE_")), event);
}

/// Emitted when register_context is called.
pub fn emit_register_context(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_context"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("REGISTER_")), event);
}

/// Emitted when deactivate_context is called.
pub fn emit_deactivate_context(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_context"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when submit_encrypted_computation is called.
pub fn emit_submit_encrypted_computation(env: &Env, caller: &Address) {
    let event = HomomorphicRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HomomorphicRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_encrypted_computation"),
        },
    };
    env.events()
        .publish((symbol_short!("HOMO"), symbol_short!("SUBMIT_EN")), event);
}
