//! # DicomwebServices Events Module
//!
//! Standardized event emissions for the dicomweb_services contract.
//! Topic naming convention: (DICOM, ACTION)

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
pub struct DicomwebServicesEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct DicomwebServicesEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: DicomwebServicesEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("INIT")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("SET_PAUSE")), event);
}

/// Emitted when qido_search_studies is called.
pub fn emit_qido_search_studies(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "qido_search_studies"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("QIDO_SEAR")), event);
}

/// Emitted when qido_search_series is called.
pub fn emit_qido_search_series(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "qido_search_series"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("QIDO_SEAR")), event);
}

/// Emitted when qido_search_instances is called.
pub fn emit_qido_search_instances(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "qido_search_instances"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("QIDO_SEAR")), event);
}

/// Emitted when wado_retrieve_study is called.
pub fn emit_wado_retrieve_study(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "wado_retrieve_study"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("WADO_RETR")), event);
}

/// Emitted when wado_retrieve_series is called.
pub fn emit_wado_retrieve_series(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "wado_retrieve_series"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("WADO_RETR")), event);
}

/// Emitted when wado_retrieve_instance is called.
pub fn emit_wado_retrieve_instance(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "wado_retrieve_instance"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("WADO_RETR")), event);
}

/// Emitted when wado_retrieve_bulk_data is called.
pub fn emit_wado_retrieve_bulk_data(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "wado_retrieve_bulk_data"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("WADO_RETR")), event);
}

/// Emitted when wado_retrieve_bulk_data_batch is called.
pub fn emit_wado_retrieve_bulk_data_batch(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "wado_retrieve_bulk_data_batch"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("WADO_RETR")), event);
}

/// Emitted when stow_store_instance is called.
pub fn emit_stow_store_instance(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "stow_store_instance"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("STOW_STOR")), event);
}

/// Emitted when stow_store_batch is called.
pub fn emit_stow_store_batch(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "stow_store_batch"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("STOW_STOR")), event);
}

/// Emitted when cache_set is called.
pub fn emit_cache_set(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "cache_set"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("CACHE_SET")), event);
}

/// Emitted when cache_get is called.
pub fn emit_cache_get(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "cache_get"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("CACHE_GET")), event);
}

/// Emitted when cache_invalidate is called.
pub fn emit_cache_invalidate(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "cache_invalidate"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("CACHE_INV")), event);
}

/// Emitted when list_studies is called.
pub fn emit_list_studies(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_studies"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("LIST_STUD")), event);
}

/// Emitted when placeholder is called.
pub fn emit_placeholder(env: &Env, caller: &Address) {
    let event = DicomwebServicesEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: DicomwebServicesEventData {
            user: caller.clone(),
            action: String::from_str(env, "placeholder"),
        },
    };
    env.events()
        .publish((symbol_short!("DICOM"), symbol_short!("PLACEHOLD")), event);
}
