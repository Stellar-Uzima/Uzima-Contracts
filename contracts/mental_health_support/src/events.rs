//! # MentalHealthSupport Events Module
//!
//! Standardized event emissions for the mental_health_support contract.
//! Topic naming convention: (MHSP, ACTION)

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
pub struct MentalHealthSupportEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MentalHealthSupportEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MentalHealthSupportEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("INIT")), event);
}

/// Emitted when set_integration_contracts is called.
pub fn emit_set_integration_contracts(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_integration_contracts"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("SET_INTEG")), event);
}

/// Emitted when set_emergency_routing_commitment is called.
pub fn emit_set_emergency_routing_commitment(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_emergency_routing_commitment"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("SET_EMERG")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when enroll is called.
pub fn emit_enroll(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "enroll"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("ENROLL")), event);
}

/// Emitted when log_mood is called.
pub fn emit_log_mood(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_mood"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("LOG_MOOD")), event);
}

/// Emitted when book_teletherapy is called.
pub fn emit_book_teletherapy(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "book_teletherapy"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("BOOK_TELE")), event);
}

/// Emitted when report_crisis is called.
pub fn emit_report_crisis(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_crisis"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("REPORT_CR")), event);
}

/// Emitted when create_peer_community is called.
pub fn emit_create_peer_community(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_peer_community"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("CREATE_PE")), event);
}

/// Emitted when join_peer_community is called.
pub fn emit_join_peer_community(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "join_peer_community"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("JOIN_PEER")), event);
}

/// Emitted when list_community_members is called.
pub fn emit_list_community_members(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_community_members"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("LIST_COMM")), event);
}

/// Emitted when open_crisis_queue is called.
pub fn emit_open_crisis_queue(env: &Env, caller: &Address) {
    let event = MentalHealthSupportEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MentalHealthSupportEventData {
            user: caller.clone(),
            action: String::from_str(env, "open_crisis_queue"),
        },
    };
    env.events()
        .publish((symbol_short!("MHSP"), symbol_short!("OPEN_CRIS")), event);
}
