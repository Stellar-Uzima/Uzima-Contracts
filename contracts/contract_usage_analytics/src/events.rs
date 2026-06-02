//! # ContractUsageAnalytics Events Module
//!
//! Standardized event emissions for the contract_usage_analytics contract.
//! Topic naming convention: (ANALY, ACTION)

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
pub struct ContractUsageAnalyticsEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ContractUsageAnalyticsEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ContractUsageAnalyticsEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ContractUsageAnalyticsEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractUsageAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ANALY"), symbol_short!("INIT")), event);
}

/// Emitted when record_call is called.
pub fn emit_record_call(env: &Env, caller: &Address) {
    let event = ContractUsageAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractUsageAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_call"),
        },
    };
    env.events()
        .publish((symbol_short!("ANALY"), symbol_short!("RECORD_CA")), event);
}

/// Emitted when take_snapshot is called.
pub fn emit_take_snapshot(env: &Env, caller: &Address) {
    let event = ContractUsageAnalyticsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ContractUsageAnalyticsEventData {
            user: caller.clone(),
            action: String::from_str(env, "take_snapshot"),
        },
    };
    env.events()
        .publish((symbol_short!("ANALY"), symbol_short!("TAKE_SNAP")), event);
}
