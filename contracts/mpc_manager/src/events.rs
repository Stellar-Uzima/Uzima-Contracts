//! # MpcManager Events Module
//!
//! Standardized event emissions for the mpc_manager contract.
//! Topic naming convention: (MPC, ACTION)

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
pub struct MpcManagerEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MpcManagerEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MpcManagerEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("INIT")), event);
}

/// Emitted when start_session is called.
pub fn emit_start_session(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "start_session"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("START_SES")), event);
}

/// Emitted when commit_share is called.
pub fn emit_commit_share(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "commit_share"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("COMMIT_SH")), event);
}

/// Emitted when reveal_share is called.
pub fn emit_reveal_share(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "reveal_share"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("REVEAL_SH")), event);
}

/// Emitted when finalize_session is called.
pub fn emit_finalize_session(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "finalize_session"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("FINALIZE_")), event);
}

/// Emitted when create_secret_shares is called.
pub fn emit_create_secret_shares(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_secret_shares"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("CREATE_SE")), event);
}

/// Emitted when submit_computation_proof is called.
pub fn emit_submit_computation_proof(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_computation_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("SUBMIT_CO")), event);
}

/// Emitted when perform_statistical_analysis is called.
pub fn emit_perform_statistical_analysis(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "perform_statistical_analysis"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("PERFORM_S")), event);
}

/// Emitted when train_secure_ml_model is called.
pub fn emit_train_secure_ml_model(env: &Env, caller: &Address) {
    let event = MpcManagerEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MpcManagerEventData {
            user: caller.clone(),
            action: String::from_str(env, "train_secure_ml_model"),
        },
    };
    env.events()
        .publish((symbol_short!("MPC"), symbol_short!("TRAIN_SEC")), event);
}
