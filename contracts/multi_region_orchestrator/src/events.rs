//! # MultiRegionOrchestrator Events Module
//!
//! Standardized event emissions for the multi_region_orchestrator contract.
//! Topic naming convention: (MRORC, ACTION)

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
pub struct MultiRegionOrchestratorEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MultiRegionOrchestratorEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MultiRegionOrchestratorEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("INIT")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("SET_PAUSE")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when register_region is called.
pub fn emit_register_region(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_region"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("REGISTER_")), event);
}

/// Emitted when list_regions is called.
pub fn emit_list_regions(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_regions"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("LIST_REGI")), event);
}

/// Emitted when update_region_status is called.
pub fn emit_update_region_status(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_region_status"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("UPDATE_RE")), event);
}

/// Emitted when trigger_failover is called.
pub fn emit_trigger_failover(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "trigger_failover"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("TRIGGER_F")), event);
}

/// Emitted when sync_data is called.
pub fn emit_sync_data(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_data"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("SYNC_DATA")), event);
}

/// Emitted when record_uptime_metric is called.
pub fn emit_record_uptime_metric(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_uptime_metric"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("RECORD_UP")), event);
}

/// Emitted when set_policy is called.
pub fn emit_set_policy(env: &Env, caller: &Address) {
    let event = MultiRegionOrchestratorEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MultiRegionOrchestratorEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_policy"),
        },
    };
    env.events()
        .publish((symbol_short!("MRORC"), symbol_short!("SET_POLIC")), event);
}
