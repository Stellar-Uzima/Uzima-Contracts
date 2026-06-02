//! # ReputationIntegration Events Module
//!
//! Standardized event emissions for the reputation_integration contract.
//! Topic naming convention: (REPINT, ACTION)

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
pub struct ReputationIntegrationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ReputationIntegrationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ReputationIntegrationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("INIT")), event);
}

/// Emitted when sync_provider_reputation is called.
pub fn emit_sync_provider_reputation(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_provider_reputation"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("SYNC_PROV")), event);
}

/// Emitted when batch_sync_providers is called.
pub fn emit_batch_sync_providers(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "batch_sync_providers"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("BATCH_SYN")), event);
}

/// Emitted when auto_sync_all_providers is called.
pub fn emit_auto_sync_all_providers(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "auto_sync_all_providers"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("AUTO_SYNC")), event);
}

/// Emitted when update_score_mapping is called.
pub fn emit_update_score_mapping(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_score_mapping"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("UPDATE_SC")), event);
}

/// Emitted when update_sync_settings is called.
pub fn emit_update_sync_settings(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_sync_settings"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("UPDATE_SY")), event);
}

/// Emitted when trigger_credential_sync is called.
pub fn emit_trigger_credential_sync(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "trigger_credential_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("TRIGGER_C")), event);
}

/// Emitted when trigger_feedback_sync is called.
pub fn emit_trigger_feedback_sync(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "trigger_feedback_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("TRIGGER_F")), event);
}

/// Emitted when trigger_conduct_sync is called.
pub fn emit_trigger_conduct_sync(env: &Env, caller: &Address) {
    let event = ReputationIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ReputationIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "trigger_conduct_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("REPINT"), symbol_short!("TRIGGER_C")), event);
}
