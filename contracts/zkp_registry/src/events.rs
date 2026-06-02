//! # ZkpRegistry Events Module
//!
//! Standardized event emissions for the zkp_registry contract.
//! Topic naming convention: (ZKPREG, ACTION)

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
pub struct ZkpRegistryEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct ZkpRegistryEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: ZkpRegistryEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("INIT")), event);
}

/// Emitted when configure_multisig is called.
pub fn emit_configure_multisig(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "configure_multisig"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("CONFIGURE")), event);
}

/// Emitted when create_admin_proposal is called.
pub fn emit_create_admin_proposal(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_admin_proposal"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("CREATE_AD")), event);
}

/// Emitted when approve_admin_proposal is called.
pub fn emit_approve_admin_proposal(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve_admin_proposal"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("APPROVE_A")), event);
}

/// Emitted when execute_admin_proposal is called.
pub fn emit_execute_admin_proposal(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "execute_admin_proposal"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("EXECUTE_A")), event);
}

/// Emitted when emergency_override is called.
pub fn emit_emergency_override(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "emergency_override"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("EMERGENCY")), event);
}

/// Emitted when register_circuit is called.
pub fn emit_register_circuit(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_circuit"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("REGISTER_")), event);
}

/// Emitted when submit_zkp is called.
pub fn emit_submit_zkp(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_zkp"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("SUBMIT_ZK")), event);
}

/// Emitted when submit_zkp_batch is called.
pub fn emit_submit_zkp_batch(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_zkp_batch"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("SUBMIT_ZK")), event);
}

/// Emitted when create_medical_record_proof is called.
pub fn emit_create_medical_record_proof(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_medical_record_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("CREATE_ME")), event);
}

/// Emitted when create_range_proof is called.
pub fn emit_create_range_proof(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_range_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("CREATE_RA")), event);
}

/// Emitted when create_credential_proof is called.
pub fn emit_create_credential_proof(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_credential_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("CREATE_CR")), event);
}

/// Emitted when create_recursive_proof is called.
pub fn emit_create_recursive_proof(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_recursive_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("CREATE_RE")), event);
}

/// Emitted when cleanup_proof is called.
pub fn emit_cleanup_proof(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "cleanup_proof"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("CLEANUP_P")), event);
}

/// Emitted when export_state is called.
pub fn emit_export_state(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "export_state"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("EXPORT_ST")), event);
}

/// Emitted when import_state is called.
pub fn emit_import_state(env: &Env, caller: &Address) {
    let event = ZkpRegistryEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: ZkpRegistryEventData {
            user: caller.clone(),
            action: String::from_str(env, "import_state"),
        },
    };
    env.events()
        .publish((symbol_short!("ZKPREG"), symbol_short!("IMPORT_ST")), event);
}
