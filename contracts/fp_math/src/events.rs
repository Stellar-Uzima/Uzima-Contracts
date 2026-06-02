//! # FpMath Events Module
//!
//! Standardized event emissions for the fp_math contract.
//! Topic naming convention: (FPM, ACTION)

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
pub struct FpMathEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct FpMathEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: FpMathEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = FpMathEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FpMathEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FPM"), symbol_short!("INIT")), event);
}

/// Emitted when mul_bps is called.
pub fn emit_mul_bps(env: &Env, caller: &Address) {
    let event = FpMathEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FpMathEventData {
            user: caller.clone(),
            action: String::from_str(env, "mul_bps"),
        },
    };
    env.events()
        .publish((symbol_short!("FPM"), symbol_short!("MUL_BPS")), event);
}

/// Emitted when mul_bps_round_half_up is called.
pub fn emit_mul_bps_round_half_up(env: &Env, caller: &Address) {
    let event = FpMathEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FpMathEventData {
            user: caller.clone(),
            action: String::from_str(env, "mul_bps_round_half_up"),
        },
    };
    env.events()
        .publish((symbol_short!("FPM"), symbol_short!("MUL_BPS_R")), event);
}

/// Emitted when tokens_for_payment is called.
pub fn emit_tokens_for_payment(env: &Env, caller: &Address) {
    let event = FpMathEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: FpMathEventData {
            user: caller.clone(),
            action: String::from_str(env, "tokens_for_payment"),
        },
    };
    env.events()
        .publish((symbol_short!("FPM"), symbol_short!("TOKENS_FO")), event);
}
