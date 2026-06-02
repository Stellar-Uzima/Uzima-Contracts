//! # HealthcareDataMarketplace Events Module
//!
//! Standardized event emissions for the healthcare_data_marketplace contract.
//! Topic naming convention: (HDMKT, ACTION)

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
pub struct HealthcareDataMarketplaceEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareDataMarketplaceEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcareDataMarketplaceEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("INIT")), event);
}

/// Emitted when register_provider is called.
pub fn emit_register_provider(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_provider"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("REGISTER_")), event);
}

/// Emitted when set_provider_status is called.
pub fn emit_set_provider_status(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_provider_status"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("SET_PROVI")), event);
}

/// Emitted when create_listing is called.
pub fn emit_create_listing(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_listing"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("CREATE_LI")), event);
}

/// Emitted when reserve_purchase is called.
pub fn emit_reserve_purchase(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "reserve_purchase"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("RESERVE_P")), event);
}

/// Emitted when initiate_transaction is called.
pub fn emit_initiate_transaction(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_transaction"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("INITIATE_")), event);
}

/// Emitted when finalize_settlement is called.
pub fn emit_finalize_settlement(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "finalize_settlement"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("FINALIZE_")), event);
}

/// Emitted when cancel_listing is called.
pub fn emit_cancel_listing(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "cancel_listing"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("CANCEL_LI")), event);
}

/// Emitted when purchase_access_tier is called.
pub fn emit_purchase_access_tier(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "purchase_access_tier"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("PURCHASE_")), event);
}

/// Emitted when query_data is called.
pub fn emit_query_data(env: &Env, caller: &Address) {
    let event = HealthcareDataMarketplaceEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareDataMarketplaceEventData {
            user: caller.clone(),
            action: String::from_str(env, "query_data"),
        },
    };
    env.events()
        .publish((symbol_short!("HDMKT"), symbol_short!("QUERY_DAT")), event);
}
