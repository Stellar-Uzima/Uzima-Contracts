//! # GenomicData Events Module
//!
//! Standardized event emissions for the genomic_data contract.
//! Topic naming convention: (GENO, ACTION)

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
pub struct GenomicDataEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct GenomicDataEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: GenomicDataEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("INIT")), event);
}

/// Emitted when set_zk_verifier is called.
pub fn emit_set_zk_verifier(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_zk_verifier"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("SET_ZK_VE")), event);
}

/// Emitted when add_record is called.
pub fn emit_add_record(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_record"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("ADD_RECOR")), event);
}

/// Emitted when grant_consent is called.
pub fn emit_grant_consent(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("GRANT_CON")), event);
}

/// Emitted when revoke_consent is called.
pub fn emit_revoke_consent(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("REVOKE_CO")), event);
}

/// Emitted when grant_research_consent is called.
pub fn emit_grant_research_consent(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_research_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("GRANT_RES")), event);
}

/// Emitted when revoke_research_consent is called.
pub fn emit_revoke_research_consent(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_research_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("REVOKE_RE")), event);
}

/// Emitted when verify_and_grant_access is called.
pub fn emit_verify_and_grant_access(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_and_grant_access"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("VERIFY_AN")), event);
}

/// Emitted when add_gene_disease_assoc is called.
pub fn emit_add_gene_disease_assoc(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_gene_disease_assoc"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("ADD_GENE_")), event);
}

/// Emitted when add_drug_response is called.
pub fn emit_add_drug_response(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_drug_response"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("ADD_DRUG_")), event);
}

/// Emitted when set_ancestry_profile is called.
pub fn emit_set_ancestry_profile(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_ancestry_profile"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("SET_ANCES")), event);
}

/// Emitted when create_listing is called.
pub fn emit_create_listing(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_listing"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("CREATE_LI")), event);
}

/// Emitted when purchase_listing is called.
pub fn emit_purchase_listing(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "purchase_listing"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("PURCHASE_")), event);
}

/// Emitted when report_breach is called.
pub fn emit_report_breach(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "report_breach"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("REPORT_BR")), event);
}

/// Emitted when upgrade is called.
pub fn emit_upgrade(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("UPGRADE")), event);
}

/// Emitted when validate_upgrade is called.
pub fn emit_validate_upgrade(env: &Env, caller: &Address) {
    let event = GenomicDataEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: GenomicDataEventData {
            user: caller.clone(),
            action: String::from_str(env, "validate_upgrade"),
        },
    };
    env.events()
        .publish((symbol_short!("GENO"), symbol_short!("VALIDATE_")), event);
}
