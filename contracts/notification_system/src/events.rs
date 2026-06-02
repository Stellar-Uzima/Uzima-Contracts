//! # NotificationSystem Events Module
//!
//! Standardized event emissions for the notification_system contract.
//! Topic naming convention: (NOTIF, ACTION)

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
pub struct NotificationSystemEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct NotificationSystemEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: NotificationSystemEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("INIT")), event);
}

/// Emitted when add_authorized_sender is called.
pub fn emit_add_authorized_sender(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_authorized_sender"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("ADD_AUTHO")), event);
}

/// Emitted when remove_authorized_sender is called.
pub fn emit_remove_authorized_sender(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_authorized_sender"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("REMOVE_AU")), event);
}

/// Emitted when set_preferences is called.
pub fn emit_set_preferences(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_preferences"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("SET_PREFE")), event);
}

/// Emitted when create_notification is called.
pub fn emit_create_notification(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_notification"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("CREATE_NO")), event);
}

/// Emitted when create_bulk_notifications is called.
pub fn emit_create_bulk_notifications(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_bulk_notifications"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("CREATE_BU")), event);
}

/// Emitted when mark_read is called.
pub fn emit_mark_read(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_read"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("MARK_READ")), event);
}

/// Emitted when mark_all_read is called.
pub fn emit_mark_all_read(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_all_read"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("MARK_ALL_")), event);
}

/// Emitted when archive_notification is called.
pub fn emit_archive_notification(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "archive_notification"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("ARCHIVE_N")), event);
}

/// Emitted when create_alert_rule is called.
pub fn emit_create_alert_rule(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_alert_rule"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("CREATE_AL")), event);
}

/// Emitted when update_alert_rule is called.
pub fn emit_update_alert_rule(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_alert_rule"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("UPDATE_AL")), event);
}

/// Emitted when delete_alert_rule is called.
pub fn emit_delete_alert_rule(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "delete_alert_rule"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("DELETE_AL")), event);
}

/// Emitted when trigger_alert is called.
pub fn emit_trigger_alert(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "trigger_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("TRIGGER_A")), event);
}

/// Emitted when set_template is called.
pub fn emit_set_template(env: &Env, caller: &Address) {
    let event = NotificationSystemEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: NotificationSystemEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_template"),
        },
    };
    env.events()
        .publish((symbol_short!("NOTIF"), symbol_short!("SET_TEMPL")), event);
}
