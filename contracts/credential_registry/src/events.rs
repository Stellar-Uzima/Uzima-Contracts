//! # CredentialRegistry Events Module
//!
//! Standardized event emissions for the credential_registry contract.
//! Topic naming convention: (CR, ACTION)

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
pub struct CredentialRegistryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CredentialRegistryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: CredentialRegistryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = CredentialRegistryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CR"), symbol_short!("INIT")), event);
}

/// Emitted when set_issuer_admin is called.
pub fn emit_set_issuer_admin(env: &Env, caller: &Address) {
    let event = CredentialRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_issuer_admin"),
        },
    };
    env.events()
        .publish((symbol_short!("CR"), symbol_short!("SET_ISSUE")), event);
}

/// Emitted when set_credential_root is called.
pub fn emit_set_credential_root(env: &Env, caller: &Address) {
    let event = CredentialRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_credential_root"),
        },
    };
    env.events()
        .publish((symbol_short!("CR"), symbol_short!("SET_CREDE")), event);
}

/// Emitted when revoke_root is called.
pub fn emit_revoke_root(env: &Env, caller: &Address) {
    let event = CredentialRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_root"),
        },
    };
    env.events()
        .publish((symbol_short!("CR"), symbol_short!("REVOKE_RO")), event);
}

/// Emitted when set_revocation_root is called.
pub fn emit_set_revocation_root(env: &Env, caller: &Address) {
    let event = CredentialRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_revocation_root"),
        },
    };
    env.events()
        .publish((symbol_short!("CR"), symbol_short!("SET_REVOC")), event);
}

/// Emitted when batch_set_credential_roots is called.
pub fn emit_batch_set_credential_roots(env: &Env, caller: &Address) {
    let event = CredentialRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "batch_set_credential_roots"),
        },
    };
    env.events()
        .publish((symbol_short!("CR"), symbol_short!("BATCH_SET")), event);
}
