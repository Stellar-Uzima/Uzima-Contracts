//! # IdentityRegistry Events Module
//!
//! Standardized event emissions for the identity_registry contract.
//! Topic naming convention: (IDR, ACTION)

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
pub struct IdentityRegistryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct IdentityRegistryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: IdentityRegistryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("INIT")), event);
}

/// Emitted when health_check is called.
pub fn emit_health_check(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "health_check"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("HEALTH_CH")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when initialize_legacy is called.
pub fn emit_initialize_legacy(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize_legacy"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("INITIALIZ")), event);
}

/// Emitted when create_did is called.
pub fn emit_create_did(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_did"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("CREATE_DI")), event);
}

/// Emitted when resolve_did is called.
pub fn emit_resolve_did(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_did"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("RESOLVE_D")), event);
}

/// Emitted when resolve_did_by_string is called.
pub fn emit_resolve_did_by_string(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_did_by_string"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("RESOLVE_D")), event);
}

/// Emitted when update_did is called.
pub fn emit_update_did(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_did"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("UPDATE_DI")), event);
}

/// Emitted when deactivate_did is called.
pub fn emit_deactivate_did(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_did"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when add_verification_method is called.
pub fn emit_add_verification_method(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_verification_method"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ADD_VERIF")), event);
}

/// Emitted when rotate_key is called.
pub fn emit_rotate_key(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "rotate_key"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ROTATE_KE")), event);
}

/// Emitted when revoke_verification_method is called.
pub fn emit_revoke_verification_method(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_verification_method"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REVOKE_VE")), event);
}

/// Emitted when issue_credential is called.
pub fn emit_issue_credential(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "issue_credential"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ISSUE_CRE")), event);
}

/// Emitted when verify_credential is called.
pub fn emit_verify_credential(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_credential"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("VERIFY_CR")), event);
}

/// Emitted when revoke_credential is called.
pub fn emit_revoke_credential(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_credential"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REVOKE_CR")), event);
}

/// Emitted when add_recovery_guardian is called.
pub fn emit_add_recovery_guardian(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_recovery_guardian"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ADD_RECOV")), event);
}

/// Emitted when remove_recovery_guardian is called.
pub fn emit_remove_recovery_guardian(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_recovery_guardian"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REMOVE_RE")), event);
}

/// Emitted when set_recovery_threshold is called.
pub fn emit_set_recovery_threshold(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_recovery_threshold"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("SET_RECOV")), event);
}

/// Emitted when initiate_recovery is called.
pub fn emit_initiate_recovery(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_recovery"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("INITIATE_")), event);
}

/// Emitted when approve_recovery is called.
pub fn emit_approve_recovery(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve_recovery"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("APPROVE_R")), event);
}

/// Emitted when execute_recovery is called.
pub fn emit_execute_recovery(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_recovery"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("EXECUTE_R")), event);
}

/// Emitted when cancel_recovery is called.
pub fn emit_cancel_recovery(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "cancel_recovery"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("CANCEL_RE")), event);
}

/// Emitted when add_service is called.
pub fn emit_add_service(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_service"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ADD_SERVI")), event);
}

/// Emitted when remove_service is called.
pub fn emit_remove_service(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_service"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REMOVE_SE")), event);
}

/// Emitted when add_verifier is called.
pub fn emit_add_verifier(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_verifier"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ADD_VERIF")), event);
}

/// Emitted when remove_verifier is called.
pub fn emit_remove_verifier(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_verifier"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REMOVE_VE")), event);
}

/// Emitted when register_identity_hash is called.
pub fn emit_register_identity_hash(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_identity_hash"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when attest is called.
pub fn emit_attest(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "attest"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ATTEST")), event);
}

/// Emitted when revoke_attestation is called.
pub fn emit_revoke_attestation(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_attestation"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REVOKE_AT")), event);
}

/// Emitted when verify_did_authorization is called.
pub fn emit_verify_did_authorization(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_did_authorization"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("VERIFY_DI")), event);
}

/// Emitted when add_fido2_device is called.
pub fn emit_add_fido2_device(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_fido2_device"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ADD_FIDO2")), event);
}

/// Emitted when deposit_stake is called.
pub fn emit_deposit_stake(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "deposit_stake"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("DEPOSIT_S")), event);
}

/// Emitted when withdraw_stake is called.
pub fn emit_withdraw_stake(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "withdraw_stake"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("WITHDRAW_")), event);
}

/// Emitted when slash_stake is called.
pub fn emit_slash_stake(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "slash_stake"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("SLASH_STA")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when remove_role is called.
pub fn emit_remove_role(env: &Env, caller: &Address) {
    let event = IdentityRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IdentityRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_role"),
        },
    };
    env.events()
        .publish((symbol_short!("IDR"), symbol_short!("REMOVE_RO")), event);
}
