//! # ZkVerifier Events Module
//!
//! Standardized event emissions for the zk_verifier contract.
//! Topic naming convention: (ZKVER, ACTION)

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
pub struct ZkVerifierEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ZkVerifierEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ZkVerifierEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("INIT")), event);
}

/// Emitted when set_default_ttl is called.
pub fn emit_set_default_ttl(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_default_ttl"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("SET_DEFAU")), event);
}

/// Emitted when register_verifying_key is called.
pub fn emit_register_verifying_key(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_verifying_key"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("REGISTER_")), event);
}

/// Emitted when deactivate_verifying_key is called.
pub fn emit_deactivate_verifying_key(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_verifying_key"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when submit_attestation is called.
pub fn emit_submit_attestation(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_attestation"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("SUBMIT_AT")), event);
}

/// Emitted when verify_proof is called.
pub fn emit_verify_proof(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("VERIFY_PR")), event);
}

/// Emitted when compute_proof_hash is called.
pub fn emit_compute_proof_hash(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "compute_proof_hash"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("COMPUTE_P")), event);
}

/// Emitted when mark_nullifier_used is called.
pub fn emit_mark_nullifier_used(env: &Env, caller: &Address) {
    let event = ZkVerifierEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkVerifierEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_nullifier_used"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKVER"), symbol_short!("MARK_NULL")), event);
}
