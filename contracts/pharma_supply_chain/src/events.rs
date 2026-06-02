//! # PharmaSupplyChain Events Module
//!
//! Standardized event emissions for the pharma_supply_chain contract.
//! Topic naming convention: (PHARMA, ACTION)

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
pub struct PharmaSupplyChainEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PharmaSupplyChainEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PharmaSupplyChainEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("INIT")), event);
}

/// Emitted when register_manufacturer is called.
pub fn emit_register_manufacturer(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_manufacturer"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("REGISTER_")), event);
}

/// Emitted when register_medication is called.
pub fn emit_register_medication(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_medication"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("REGISTER_")), event);
}

/// Emitted when create_batch is called.
pub fn emit_create_batch(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_batch"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("CREATE_BA")), event);
}

/// Emitted when verify_batch_authenticity is called.
pub fn emit_verify_batch_authenticity(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_batch_authenticity"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("VERIFY_BA")), event);
}

/// Emitted when create_shipment is called.
pub fn emit_create_shipment(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_shipment"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("CREATE_SH")), event);
}

/// Emitted when log_condition_data is called.
pub fn emit_log_condition_data(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_condition_data"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("LOG_CONDI")), event);
}

/// Emitted when complete_shipment is called.
pub fn emit_complete_shipment(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "complete_shipment"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("COMPLETE_")), event);
}

/// Emitted when run_compliance_check is called.
pub fn emit_run_compliance_check(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_compliance_check"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("RUN_COMPL")), event);
}

/// Emitted when optimize_inventory is called.
pub fn emit_optimize_inventory(env: &Env, caller: &Address) {
    let event = PharmaSupplyChainEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PharmaSupplyChainEventData {
            user: caller.clone(),
            action: String::from_str(env, "optimize_inventory"),
        },
    };
    env.events()
        .publish((symbol_short!("PHARMA"), symbol_short!("OPTIMIZE_")), event);
}
