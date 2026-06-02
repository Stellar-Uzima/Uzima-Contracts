//! # AppointmentBookingEscrow Events Module
//!
//! Standardized event emissions for the appointment_booking_escrow contract.
//! Topic naming convention: (ABESC, ACTION)

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
pub struct AppointmentBookingEscrowEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AppointmentBookingEscrowEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AppointmentBookingEscrowEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("INIT")), event);
}

/// Emitted when book_appointment is called.
pub fn emit_book_appointment(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "book_appointment"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("BOOK_APPO")), event);
}

/// Emitted when confirm_appointment is called.
pub fn emit_confirm_appointment(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "confirm_appointment"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("CONFIRM_A")), event);
}

/// Emitted when refund_appointment is called.
pub fn emit_refund_appointment(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "refund_appointment"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("REFUND_AP")), event);
}

/// Emitted when mark_no_show is called.
pub fn emit_mark_no_show(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "mark_no_show"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("MARK_NO_S")), event);
}

/// Emitted when send_reminder is called.
pub fn emit_send_reminder(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "send_reminder"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("SEND_REMI")), event);
}

/// Emitted when health_check is called.
pub fn emit_health_check(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "health_check"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("HEALTH_CH")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = AppointmentBookingEscrowEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AppointmentBookingEscrowEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("ABESC"), symbol_short!("SET_PAUSE")), event);
}
