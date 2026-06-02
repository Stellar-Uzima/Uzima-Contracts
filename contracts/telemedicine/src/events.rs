//! # Telemedicine Events Module
//!
//! Standardized event emissions for the telemedicine contract.
//! Topic naming convention: (TEMED, ACTION)

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
pub struct TelemedicineEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct TelemedicineEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: TelemedicineEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("INIT")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when register_provider is called.
pub fn emit_register_provider(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_provider"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("REGISTER_")), event);
}

/// Emitted when deactivate_provider is called.
pub fn emit_deactivate_provider(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_provider"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when register_patient is called.
pub fn emit_register_patient(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_patient"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("REGISTER_")), event);
}

/// Emitted when grant_consent is called.
pub fn emit_grant_consent(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("GRANT_CON")), event);
}

/// Emitted when revoke_consent is called.
pub fn emit_revoke_consent(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("REVOKE_CO")), event);
}

/// Emitted when schedule_consultation is called.
pub fn emit_schedule_consultation(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "schedule_consultation"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("SCHEDULE_")), event);
}

/// Emitted when start_consultation is called.
pub fn emit_start_consultation(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "start_consultation"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("START_CON")), event);
}

/// Emitted when complete_consultation is called.
pub fn emit_complete_consultation(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "complete_consultation"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("COMPLETE_")), event);
}

/// Emitted when issue_prescription is called.
pub fn emit_issue_prescription(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "issue_prescription"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("ISSUE_PRE")), event);
}

/// Emitted when start_monitoring_session is called.
pub fn emit_start_monitoring_session(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "start_monitoring_session"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("START_MON")), event);
}

/// Emitted when end_monitoring_session is called.
pub fn emit_end_monitoring_session(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "end_monitoring_session"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("END_MONIT")), event);
}

/// Emitted when upsert_knowledge_entry is called.
pub fn emit_upsert_knowledge_entry(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "upsert_knowledge_entry"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("UPSERT_KN")), event);
}

/// Emitted when configure_emergency_protocol is called.
pub fn emit_configure_emergency_protocol(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_emergency_protocol"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when submit_chatbot_inquiry is called.
pub fn emit_submit_chatbot_inquiry(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_chatbot_inquiry"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("SUBMIT_CH")), event);
}

/// Emitted when resolve_emergency_case is called.
pub fn emit_resolve_emergency_case(env: &Env, caller: &Address) {
    let event = TelemedicineEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: TelemedicineEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_emergency_case"),
        },
    };
    env.events()
        .publish((symbol_short!("TEMED"), symbol_short!("RESOLVE_E")), event);
}
