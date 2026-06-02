//! # HealthcareAnalyticsDashboard Events Module
//!
//! Standardized event emissions for the healthcare_analytics_dashboard contract.
//! Topic naming convention: (HCANA, ACTION)

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
pub struct HealthcareAnalyticsDashboardEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthcareAnalyticsDashboardEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: HealthcareAnalyticsDashboardEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("INIT")), event);
}

/// Emitted when set_collector is called.
pub fn emit_set_collector(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_collector"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("SET_COLLE")), event);
}

/// Emitted when configure_ai_analytics is called.
pub fn emit_configure_ai_analytics(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_ai_analytics"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when register_data_lake_connection is called.
pub fn emit_register_data_lake_connection(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_data_lake_connection"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("REGISTER_")), event);
}

/// Emitted when sync_export_to_data_lake is called.
pub fn emit_sync_export_to_data_lake(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_export_to_data_lake"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("SYNC_EXPO")), event);
}

/// Emitted when optimize_query_profile is called.
pub fn emit_optimize_query_profile(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "optimize_query_profile"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("OPTIMIZE_")), event);
}

/// Emitted when record_medical_metric is called.
pub fn emit_record_medical_metric(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_medical_metric"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("RECORD_ME")), event);
}

/// Emitted when record_system_snapshot is called.
pub fn emit_record_system_snapshot(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_system_snapshot"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("RECORD_SY")), event);
}

/// Emitted when create_report_template is called.
pub fn emit_create_report_template(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_report_template"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("CREATE_RE")), event);
}

/// Emitted when schedule_report is called.
pub fn emit_schedule_report(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "schedule_report"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("SCHEDULE_")), event);
}

/// Emitted when run_scheduled_report is called.
pub fn emit_run_scheduled_report(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_scheduled_report"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("RUN_SCHED")), event);
}

/// Emitted when upsert_compliance_summary is called.
pub fn emit_upsert_compliance_summary(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "upsert_compliance_summary"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("UPSERT_CO")), event);
}

/// Emitted when sync_ai_round is called.
pub fn emit_sync_ai_round(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_ai_round"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("SYNC_AI_R")), event);
}

/// Emitted when set_differential_privacy_contract is called.
pub fn emit_set_differential_privacy_contract(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_differential_privacy_contract"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("SET_DIFFE")), event);
}

/// Emitted when apply_differential_privacy_noise is called.
pub fn emit_apply_differential_privacy_noise(env: &Env, caller: &Address) {
    let event = HealthcareAnalyticsDashboardEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: HealthcareAnalyticsDashboardEventData {
            user: caller.clone(),
            action: String::from_str(env, "apply_differential_privacy_noise"),
        },
    };
    env.events()
        .publish((symbol_short!("HCANA"), symbol_short!("APPLY_DIF")), event);
}
