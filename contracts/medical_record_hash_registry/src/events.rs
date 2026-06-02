//! # MedicalRecordHashRegistry Events Module
//!
//! Standardized event emissions for the medical_record_hash_registry contract.
//! Topic naming convention: (MHR, ACTION)

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
pub struct MedicalRecordHashRegistryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicalRecordHashRegistryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MedicalRecordHashRegistryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MedicalRecordHashRegistryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordHashRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MHR"), symbol_short!("INIT")), event);
}

/// Emitted when store_record is called.
pub fn emit_store_record(env: &Env, caller: &Address) {
    let event = MedicalRecordHashRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordHashRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "store_record"),
        },
    };
    env.events()
        .publish((symbol_short!("MHR"), symbol_short!("STORE_REC")), event);
}

/// Emitted when verify_record is called.
pub fn emit_verify_record(env: &Env, caller: &Address) {
    let event = MedicalRecordHashRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordHashRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_record"),
        },
    };
    env.events()
        .publish((symbol_short!("MHR"), symbol_short!("VERIFY_RE")), event);
}
