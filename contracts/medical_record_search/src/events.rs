//! # MedicalRecordSearch Events Module
//!
//! Standardized event emissions for the medical_record_search contract.
//! Topic naming convention: (MRSRC, ACTION)

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
pub struct MedicalRecordSearchEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicalRecordSearchEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MedicalRecordSearchEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("INIT")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("SET_PAUSE")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when set_cache_policy is called.
pub fn emit_set_cache_policy(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_cache_policy"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("SET_CACHE")), event);
}

/// Emitted when set_ranking is called.
pub fn emit_set_ranking(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_ranking"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("SET_RANKI")), event);
}

/// Emitted when index_record is called.
pub fn emit_index_record(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "index_record"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("INDEX_REC")), event);
}

/// Emitted when batch_index_records is called.
pub fn emit_batch_index_records(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "batch_index_records"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("BATCH_IND")), event);
}

/// Emitted when search is called.
pub fn emit_search(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "search"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("SEARCH")), event);
}

/// Emitted when invalidate_cache is called.
pub fn emit_invalidate_cache(env: &Env, caller: &Address) {
    let event = MedicalRecordSearchEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordSearchEventData {
            user: caller.clone(),
            action: String::from_str(env, "invalidate_cache"),
        },
    };
    env.events()
        .publish((symbol_short!("MRSRC"), symbol_short!("INVALIDAT")), event);
}
