//! # PatientGamification Events Module
//!
//! Standardized event emissions for the patient_gamification contract.
//! Topic naming convention: (PGAME, ACTION)

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
pub struct PatientGamificationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientGamificationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: PatientGamificationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("INIT")), event);
}

/// Emitted when create_achievement is called.
pub fn emit_create_achievement(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_achievement"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("CREATE_AC")), event);
}

/// Emitted when update_achievement_progress is called.
pub fn emit_update_achievement_progress(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_achievement_progress"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("UPDATE_AC")), event);
}

/// Emitted when create_challenge is called.
pub fn emit_create_challenge(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_challenge"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("CREATE_CH")), event);
}

/// Emitted when join_challenge is called.
pub fn emit_join_challenge(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "join_challenge"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("JOIN_CHAL")), event);
}

/// Emitted when update_challenge_progress is called.
pub fn emit_update_challenge_progress(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_challenge_progress"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("UPDATE_CH")), event);
}

/// Emitted when redeem_points is called.
pub fn emit_redeem_points(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "redeem_points"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("REDEEM_PO")), event);
}

/// Emitted when commit_random_bonus is called.
pub fn emit_commit_random_bonus(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "commit_random_bonus"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("COMMIT_RA")), event);
}

/// Emitted when reveal_random_bonus is called.
pub fn emit_reveal_random_bonus(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "reveal_random_bonus"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("REVEAL_RA")), event);
}

/// Emitted when create_social_profile is called.
pub fn emit_create_social_profile(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_social_profile"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("CREATE_SO")), event);
}

/// Emitted when update_social_profile is called.
pub fn emit_update_social_profile(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_social_profile"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("UPDATE_SO")), event);
}

/// Emitted when record_health_metric is called.
pub fn emit_record_health_metric(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_health_metric"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("RECORD_HE")), event);
}

/// Emitted when update_config is called.
pub fn emit_update_config(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_config"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when deactivate_achievement is called.
pub fn emit_deactivate_achievement(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_achievement"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when deactivate_challenge is called.
pub fn emit_deactivate_challenge(env: &Env, caller: &Address) {
    let event = PatientGamificationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: PatientGamificationEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_challenge"),
        },
    };
    env.events()
        .publish((symbol_short!("PGAME"), symbol_short!("DEACTIVAT")), event);
}
