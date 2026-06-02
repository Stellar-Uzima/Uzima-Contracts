//! # MedicalRecordBackup Events Module
//!
//! Standardized event emissions for the medical_record_backup contract.
//! Topic naming convention: (MRBAK, ACTION)

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
pub struct MedicalRecordBackupEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicalRecordBackupEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MedicalRecordBackupEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("INIT")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("SET_PAUSE")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when set_policy is called.
pub fn emit_set_policy(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_policy"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("SET_POLIC")), event);
}

/// Emitted when register_target is called.
pub fn emit_register_target(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_target"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("REGISTER_")), event);
}

/// Emitted when set_target_active is called.
pub fn emit_set_target_active(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_target_active"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("SET_TARGE")), event);
}

/// Emitted when list_targets is called.
pub fn emit_list_targets(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_targets"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("LIST_TARG")), event);
}

/// Emitted when run_scheduled_backup is called.
pub fn emit_run_scheduled_backup(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_scheduled_backup"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("RUN_SCHED")), event);
}

/// Emitted when run_backup_now is called.
pub fn emit_run_backup_now(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_backup_now"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("RUN_BACKU")), event);
}

/// Emitted when verify_backup_integrity is called.
pub fn emit_verify_backup_integrity(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_backup_integrity"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("VERIFY_BA")), event);
}

/// Emitted when request_restore is called.
pub fn emit_request_restore(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "request_restore"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("REQUEST_R")), event);
}

/// Emitted when approve_restore is called.
pub fn emit_approve_restore(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve_restore"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("APPROVE_R")), event);
}

/// Emitted when execute_restore is called.
pub fn emit_execute_restore(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_restore"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("EXECUTE_R")), event);
}

/// Emitted when run_recovery_test is called.
pub fn emit_run_recovery_test(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_recovery_test"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("RUN_RECOV")), event);
}

/// Emitted when optimize_and_cleanup is called.
pub fn emit_optimize_and_cleanup(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "optimize_and_cleanup"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("OPTIMIZE_")), event);
}

/// Emitted when report_target_failure is called.
pub fn emit_report_target_failure(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_target_failure"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("REPORT_TA")), event);
}

/// Emitted when resolve_alert is called.
pub fn emit_resolve_alert(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_alert"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("RESOLVE_A")), event);
}

/// Emitted when list_alerts is called.
pub fn emit_list_alerts(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_alerts"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("LIST_ALER")), event);
}

/// Emitted when list_artifacts is called.
pub fn emit_list_artifacts(env: &Env, caller: &Address) {
    let event = MedicalRecordBackupEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalRecordBackupEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_artifacts"),
        },
    };
    env.events()
        .publish((symbol_short!("MRBAK"), symbol_short!("LIST_ARTI")), event);
}
