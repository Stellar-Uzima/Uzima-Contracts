//! # MedicalConsentNft Events Module
//!
//! Standardized event emissions for the medical_consent_nft contract.
//! Topic naming convention: (MCNFT, ACTION)

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
pub struct MedicalConsentNftEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicalConsentNftEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MedicalConsentNftEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("INIT")), event);
}

/// Emitted when add_issuer is called.
pub fn emit_add_issuer(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_issuer"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("ADD_ISSUE")), event);
}

/// Emitted when remove_issuer is called.
pub fn emit_remove_issuer(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_issuer"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("REMOVE_IS")), event);
}

/// Emitted when mint_consent is called.
pub fn emit_mint_consent(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "mint_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("MINT_CONS")), event);
}

/// Emitted when update_consent is called.
pub fn emit_update_consent(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when revoke_consent is called.
pub fn emit_revoke_consent(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("REVOKE_CO")), event);
}

/// Emitted when transfer is called.
pub fn emit_transfer(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "transfer"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("TRANSFER")), event);
}

/// Emitted when owner_of is called.
pub fn emit_owner_of(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "owner_of"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("OWNER_OF")), event);
}

/// Emitted when tokens_of_owner is called.
pub fn emit_tokens_of_owner(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "tokens_of_owner"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("TOKENS_OF")), event);
}

/// Emitted when set_granular_permissions is called.
pub fn emit_set_granular_permissions(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_granular_permissions"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("SET_GRANU")), event);
}

/// Emitted when set_access_controls is called.
pub fn emit_set_access_controls(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_access_controls"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("SET_ACCES")), event);
}

/// Emitted when record_access is called.
pub fn emit_record_access(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_access"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("RECORD_AC")), event);
}

/// Emitted when delegate_consent is called.
pub fn emit_delegate_consent(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "delegate_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("DELEGATE_")), event);
}

/// Emitted when revoke_delegation is called.
pub fn emit_revoke_delegation(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_delegation"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("REVOKE_DE")), event);
}

/// Emitted when set_inheritance is called.
pub fn emit_set_inheritance(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_inheritance"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("SET_INHER")), event);
}

/// Emitted when add_emergency_authority is called.
pub fn emit_add_emergency_authority(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_emergency_authority"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("ADD_EMERG")), event);
}

/// Emitted when emergency_override is called.
pub fn emit_emergency_override(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "emergency_override"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("EMERGENCY")), event);
}

/// Emitted when set_marketplace_enabled is called.
pub fn emit_set_marketplace_enabled(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_marketplace_enabled"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("SET_MARKE")), event);
}

/// Emitted when list_on_marketplace is called.
pub fn emit_list_on_marketplace(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_on_marketplace"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("LIST_ON_M")), event);
}

/// Emitted when purchase_marketplace_listing is called.
pub fn emit_purchase_marketplace_listing(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "purchase_marketplace_listing"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("PURCHASE_")), event);
}

/// Emitted when update_consent_dynamic is called.
pub fn emit_update_consent_dynamic(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_consent_dynamic"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("UPDATE_CO")), event);
}

/// Emitted when enable_dynamic_updates is called.
pub fn emit_enable_dynamic_updates(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "enable_dynamic_updates"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("ENABLE_DY")), event);
}

/// Emitted when generate_consent_report is called.
pub fn emit_generate_consent_report(env: &Env, caller: &Address) {
    let event = MedicalConsentNftEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalConsentNftEventData {
            user: caller.clone(),
            action: String::from_str(env, "generate_consent_report"),
        },
    };
    env.events()
        .publish((symbol_short!("MCNFT"), symbol_short!("GENERATE_")), event);
}
