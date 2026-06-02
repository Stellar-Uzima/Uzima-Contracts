//! # CrossChainBridge Events Module
//!
//! Standardized event emissions for the cross_chain_bridge contract.
//! Topic naming convention: (XCBR, ACTION)

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
pub struct CrossChainBridgeEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct CrossChainBridgeEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: CrossChainBridgeEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("INIT")), event);
}

/// Emitted when add_validator is called.
pub fn emit_add_validator(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_validator"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("ADD_VALID")), event);
}

/// Emitted when deactivate_validator is called.
pub fn emit_deactivate_validator(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_validator"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when add_supported_chain is called.
pub fn emit_add_supported_chain(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_supported_chain"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("ADD_SUPPO")), event);
}

/// Emitted when set_min_confirmations is called.
pub fn emit_set_min_confirmations(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_min_confirmations"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("SET_MIN_C")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when submit_message is called.
pub fn emit_submit_message(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_message"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("SUBMIT_ME")), event);
}

/// Emitted when confirm_message is called.
pub fn emit_confirm_message(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "confirm_message"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("CONFIRM_M")), event);
}

/// Emitted when execute_message is called.
pub fn emit_execute_message(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_message"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("EXECUTE_M")), event);
}

/// Emitted when fail_message is called.
pub fn emit_fail_message(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "fail_message"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("FAIL_MESS")), event);
}

/// Emitted when retry_message is called.
pub fn emit_retry_message(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "retry_message"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("RETRY_MES")), event);
}

/// Emitted when initiate_atomic_tx is called.
pub fn emit_initiate_atomic_tx(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_atomic_tx"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("INITIATE_")), event);
}

/// Emitted when prepare_atomic_tx is called.
pub fn emit_prepare_atomic_tx(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "prepare_atomic_tx"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("PREPARE_A")), event);
}

/// Emitted when commit_atomic_tx is called.
pub fn emit_commit_atomic_tx(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "commit_atomic_tx"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("COMMIT_AT")), event);
}

/// Emitted when abort_atomic_tx is called.
pub fn emit_abort_atomic_tx(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "abort_atomic_tx"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("ABORT_ATO")), event);
}

/// Emitted when register_record_ref is called.
pub fn emit_register_record_ref(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_record_ref"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when update_sync_status is called.
pub fn emit_update_sync_status(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_sync_status"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("UPDATE_SY")), event);
}

/// Emitted when register_oracle is called.
pub fn emit_register_oracle(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_oracle"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("REGISTER_")), event);
}

/// Emitted when deactivate_oracle is called.
pub fn emit_deactivate_oracle(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_oracle"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when submit_oracle_report is called.
pub fn emit_submit_oracle_report(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_oracle_report"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("SUBMIT_OR")), event);
}

/// Emitted when aggregate_oracle_data is called.
pub fn emit_aggregate_oracle_data(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "aggregate_oracle_data"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("AGGREGATE")), event);
}

/// Emitted when submit_proof is called.
pub fn emit_submit_proof(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("SUBMIT_PR")), event);
}

/// Emitted when verify_cross_chain_proof is called.
pub fn emit_verify_cross_chain_proof(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_cross_chain_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("VERIFY_CR")), event);
}

/// Emitted when validate_chain_address is called.
pub fn emit_validate_chain_address(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "validate_chain_address"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("VALIDATE_")), event);
}

/// Emitted when sync_cross_chain_event is called.
pub fn emit_sync_cross_chain_event(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "sync_cross_chain_event"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("SYNC_CROS")), event);
}

/// Emitted when process_sync_event is called.
pub fn emit_process_sync_event(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "process_sync_event"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("PROCESS_S")), event);
}

/// Emitted when create_operation is called.
pub fn emit_create_operation(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_operation"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("CREATE_OP")), event);
}

/// Emitted when extend_timeout is called.
pub fn emit_extend_timeout(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "extend_timeout"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("EXTEND_TI")), event);
}

/// Emitted when update_operation_status is called.
pub fn emit_update_operation_status(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_operation_status"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("UPDATE_OP")), event);
}

/// Emitted when initiate_rollback is called.
pub fn emit_initiate_rollback(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "initiate_rollback"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("INITIATE_")), event);
}

/// Emitted when execute_rollback is called.
pub fn emit_execute_rollback(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_rollback"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("EXECUTE_R")), event);
}

/// Emitted when cancel_rollback is called.
pub fn emit_cancel_rollback(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "cancel_rollback"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("CANCEL_RO")), event);
}

/// Emitted when add_relayer is called.
pub fn emit_add_relayer(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_relayer"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("ADD_RELAY")), event);
}

/// Emitted when remove_relayer is called.
pub fn emit_remove_relayer(env: &Env, caller: &Address) {
    let event = CrossChainBridgeEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: CrossChainBridgeEventData {
            user: caller.clone(),
            action: String::from_str(env, "remove_relayer"),
        },
    };
    env.events()
        .publish((symbol_short!("XCBR"), symbol_short!("REMOVE_RE")), event);
}
