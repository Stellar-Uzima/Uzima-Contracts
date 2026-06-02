//! # SutToken Events Module
//!
//! Standardized event emissions for the sut_token contract.
//! Topic naming convention: (SUTTKN, ACTION)

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
pub struct SutTokenEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct SutTokenEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: SutTokenEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("INIT")), event);
}

/// Emitted when name is called.
pub fn emit_name(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "name"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("NAME")), event);
}

/// Emitted when symbol is called.
pub fn emit_symbol(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "symbol"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("SYMBOL")), event);
}

/// Emitted when decimals is called.
pub fn emit_decimals(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "decimals"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("DECIMALS")), event);
}

/// Emitted when total_supply is called.
pub fn emit_total_supply(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "total_supply"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("TOTAL_SUP")), event);
}

/// Emitted when supply_cap is called.
pub fn emit_supply_cap(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "supply_cap"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("SUPPLY_CA")), event);
}

/// Emitted when allowance is called.
pub fn emit_allowance(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "allowance"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("ALLOWANCE")), event);
}

/// Emitted when transfer is called.
pub fn emit_transfer(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "transfer"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("TRANSFER")), event);
}

/// Emitted when transfer_from is called.
pub fn emit_transfer_from(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "transfer_from"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("TRANSFER_")), event);
}

/// Emitted when approve is called.
pub fn emit_approve(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("APPROVE")), event);
}

/// Emitted when mint is called.
pub fn emit_mint(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "mint"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("MINT")), event);
}

/// Emitted when burn is called.
pub fn emit_burn(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "burn"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("BURN")), event);
}

/// Emitted when add_minter is called.
pub fn emit_add_minter(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_minter"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("ADD_MINTE")), event);
}

/// Emitted when remove_minter is called.
pub fn emit_remove_minter(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_minter"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("REMOVE_MI")), event);
}

/// Emitted when snapshot is called.
pub fn emit_snapshot(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "snapshot"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("SNAPSHOT")), event);
}

/// Emitted when total_supply_at is called.
pub fn emit_total_supply_at(env: &Env, caller: &Address) {
    let event = SutTokenEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: SutTokenEventData {
            user: caller.clone(),
            action: String::from_str(env, "total_supply_at"),
        },
    };
    env.events()
        .publish((symbol_short!("SUTTKN"), symbol_short!("TOTAL_SUP")), event);
}
