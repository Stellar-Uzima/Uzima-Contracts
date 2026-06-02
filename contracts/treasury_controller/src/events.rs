//! # TreasuryController Events Module
//!
//! Standardized event emissions for the treasury_controller contract.
//! Topic naming convention: (TREAS, ACTION)

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
pub struct TreasuryControllerEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct TreasuryControllerEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: TreasuryControllerEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("INIT")), event);
}

/// Emitted when add_supported_token is called.
pub fn emit_add_supported_token(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_supported_token"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("ADD_SUPPO")), event);
}

/// Emitted when create_proposal is called.
pub fn emit_create_proposal(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_proposal"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("CREATE_PR")), event);
}

/// Emitted when approve_proposal is called.
pub fn emit_approve_proposal(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve_proposal"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("APPROVE_P")), event);
}

/// Emitted when execute_proposal is called.
pub fn emit_execute_proposal(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_proposal"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("EXECUTE_P")), event);
}

/// Emitted when emergency_halt is called.
pub fn emit_emergency_halt(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "emergency_halt"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("EMERGENCY")), event);
}

/// Emitted when resume_operations is called.
pub fn emit_resume_operations(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "resume_operations"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("RESUME_OP")), event);
}

/// Emitted when gnosis_get_threshold is called.
pub fn emit_gnosis_get_threshold(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "gnosis_get_threshold"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("GNOSIS_GE")), event);
}

/// Emitted when gnosis_get_owners is called.
pub fn emit_gnosis_get_owners(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "gnosis_get_owners"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("GNOSIS_GE")), event);
}

/// Emitted when governance_execute is called.
pub fn emit_governance_execute(env: &Env, caller: &Address) {
    let event = TreasuryControllerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TreasuryControllerEventData {
            user: caller.clone(),
            action: String::from_str(env, "governance_execute"),
        },
    };
    env.events()
        .publish((symbol_short!("TREAS"), symbol_short!("GOVERNANC")), event);
}
