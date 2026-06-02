//! # Fido2Authenticator Events Module
//!
//! Standardized event emissions for the fido2_authenticator contract.
//! Topic naming convention: (FIDO2, ACTION)

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
pub struct Fido2AuthenticatorEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct Fido2AuthenticatorEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: Fido2AuthenticatorEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("INIT")), event);
}

/// Emitted when set_identity_registry is called.
pub fn emit_set_identity_registry(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_identity_registry"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("SET_IDENT")), event);
}

/// Emitted when set_zk_verifier is called.
pub fn emit_set_zk_verifier(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_zk_verifier"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("SET_ZK_VE")), event);
}

/// Emitted when issue_registration_challenge is called.
pub fn emit_issue_registration_challenge(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "issue_registration_challenge"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("ISSUE_REG")), event);
}

/// Emitted when register_device is called.
pub fn emit_register_device(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_device"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("REGISTER_")), event);
}

/// Emitted when issue_auth_challenge is called.
pub fn emit_issue_auth_challenge(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "issue_auth_challenge"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("ISSUE_AUT")), event);
}

/// Emitted when verify_ed25519_assertion is called.
pub fn emit_verify_ed25519_assertion(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_ed25519_assertion"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("VERIFY_ED")), event);
}

/// Emitted when verify_zk_assertion is called.
pub fn emit_verify_zk_assertion(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_zk_assertion"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("VERIFY_ZK")), event);
}

/// Emitted when revoke_device is called.
pub fn emit_revoke_device(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_device"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("REVOKE_DE")), event);
}

/// Emitted when update_device_name is called.
pub fn emit_update_device_name(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_device_name"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("UPDATE_DE")), event);
}

/// Emitted when list_devices is called.
pub fn emit_list_devices(env: &Env, caller: &Address) {
    let event = Fido2AuthenticatorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: Fido2AuthenticatorEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_devices"),
        },
    };
    env.events()
        .publish((symbol_short!("FIDO2"), symbol_short!("LIST_DEVI")), event);
}
