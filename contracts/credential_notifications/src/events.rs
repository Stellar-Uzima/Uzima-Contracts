//! # CredentialNotifications Events Module
//!
//! Standardized event emissions for the credential_notifications contract.
//! Topic naming convention: (CREDNT, ACTION)

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
pub struct CredentialNotificationsEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CredentialNotificationsEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: CredentialNotificationsEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = CredentialNotificationsEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialNotificationsEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("CREDNT"), symbol_short!("INIT")), event);
}

/// Emitted when add_notifier is called.
pub fn emit_add_notifier(env: &Env, caller: &Address) {
    let event = CredentialNotificationsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialNotificationsEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_notifier"),
        },
    };
    env.events()
        .publish((symbol_short!("CREDNT"), symbol_short!("ADD_NOTIF")), event);
}

/// Emitted when remove_notifier is called.
pub fn emit_remove_notifier(env: &Env, caller: &Address) {
    let event = CredentialNotificationsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialNotificationsEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_notifier"),
        },
    };
    env.events()
        .publish((symbol_short!("CREDNT"), symbol_short!("REMOVE_NO")), event);
}

/// Emitted when send_notification is called.
pub fn emit_send_notification(env: &Env, caller: &Address) {
    let event = CredentialNotificationsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CredentialNotificationsEventData {
            user: caller.clone(),
            action: String::from_str(env, "send_notification"),
        },
    };
    env.events()
        .publish((symbol_short!("CREDNT"), symbol_short!("SEND_NOTI")), event);
}
