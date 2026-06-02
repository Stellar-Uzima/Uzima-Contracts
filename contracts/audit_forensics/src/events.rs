//! # AuditForensics Events Module
//!
//! Standardized event emissions for the audit_forensics contract.
//! Topic naming convention: (FOREN, ACTION)

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
pub struct AuditForensicsEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditForensicsEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: AuditForensicsEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("INIT")), event);
}

/// Emitted when configure_audit_rule is called.
pub fn emit_configure_audit_rule(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_audit_rule"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when log_event is called.
pub fn emit_log_event(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "log_event"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("LOG_EVENT")), event);
}

/// Emitted when run_automated_audit is called.
pub fn emit_run_automated_audit(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_automated_audit"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("RUN_AUTOM")), event);
}

/// Emitted when record_formal_verification is called.
pub fn emit_record_formal_verification(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_formal_verification"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("RECORD_FO")), event);
}

/// Emitted when generate_remediation_plan is called.
pub fn emit_generate_remediation_plan(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_remediation_plan"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("GENERATE_")), event);
}

/// Emitted when analyze_timeline is called.
pub fn emit_analyze_timeline(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "analyze_timeline"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("ANALYZE_T")), event);
}

/// Emitted when investigate_user is called.
pub fn emit_investigate_user(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "investigate_user"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("INVESTIGA")), event);
}

/// Emitted when generate_compliance_report is called.
pub fn emit_generate_compliance_report(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_compliance_report"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("GENERATE_")), event);
}

/// Emitted when set_alert_threshold is called.
pub fn emit_set_alert_threshold(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_alert_threshold"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("SET_ALERT")), event);
}

/// Emitted when compress_logs is called.
pub fn emit_compress_logs(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "compress_logs"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("COMPRESS_")), event);
}

/// Emitted when archive_logs is called.
pub fn emit_archive_logs(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "archive_logs"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("ARCHIVE_L")), event);
}

/// Emitted when sync_audit_cross_chain is called.
pub fn emit_sync_audit_cross_chain(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_audit_cross_chain"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("SYNC_AUDI")), event);
}

/// Emitted when share_audit_with_regulator is called.
pub fn emit_share_audit_with_regulator(env: &Env, caller: &Address) {
    let event = AuditForensicsEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: AuditForensicsEventData {
            user: caller.clone(),
            action: String::from_str(env, "share_audit_with_regulator"),
        },
    };
    env.events()
        .publish((symbol_short!("FOREN"), symbol_short!("SHARE_AUD")), event);
}
