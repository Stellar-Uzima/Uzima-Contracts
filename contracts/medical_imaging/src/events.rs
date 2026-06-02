//! # MedicalImaging Events Module
//!
//! Standardized event emissions for the medical_imaging contract.
//! Topic naming convention: (MEDIMG, ACTION)

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
pub struct MedicalImagingEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct MedicalImagingEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: MedicalImagingEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("INIT")), event);
}

/// Emitted when set_paused is called.
pub fn emit_set_paused(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_paused"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("SET_PAUSE")), event);
}

/// Emitted when assign_role is called.
pub fn emit_assign_role(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_role"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("ASSIGN_RO")), event);
}

/// Emitted when set_safety_threshold is called.
pub fn emit_set_safety_threshold(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_safety_threshold"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("SET_SAFET")), event);
}

/// Emitted when upload_image is called.
pub fn emit_upload_image(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "upload_image"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("UPLOAD_IM")), event);
}

/// Emitted when extract_and_index_metadata is called.
pub fn emit_extract_and_index_metadata(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "extract_and_index_metadata"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("EXTRACT_A")), event);
}

/// Emitted when run_edge_detection is called.
pub fn emit_run_edge_detection(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_edge_detection"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("RUN_EDGE_")), event);
}

/// Emitted when run_segmentation is called.
pub fn emit_run_segmentation(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "run_segmentation"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("RUN_SEGME")), event);
}

/// Emitted when register_ai_model is called.
pub fn emit_register_ai_model(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_ai_model"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("REGISTER_")), event);
}

/// Emitted when submit_diagnostic_assistance is called.
pub fn emit_submit_diagnostic_assistance(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_diagnostic_assistance"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("SUBMIT_DI")), event);
}

/// Emitted when grant_image_access is called.
pub fn emit_grant_image_access(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "grant_image_access"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("GRANT_IMA")), event);
}

/// Emitted when revoke_image_access is called.
pub fn emit_revoke_image_access(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "revoke_image_access"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("REVOKE_IM")), event);
}

/// Emitted when verify_share_access is called.
pub fn emit_verify_share_access(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_share_access"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("VERIFY_SH")), event);
}

/// Emitted when verify_image_integrity is called.
pub fn emit_verify_image_integrity(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "verify_image_integrity"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("VERIFY_IM")), event);
}

/// Emitted when add_annotation is called.
pub fn emit_add_annotation(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_annotation"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("ADD_ANNOT")), event);
}

/// Emitted when add_annotation_reply is called.
pub fn emit_add_annotation_reply(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "add_annotation_reply"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("ADD_ANNOT")), event);
}

/// Emitted when resolve_annotation is called.
pub fn emit_resolve_annotation(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "resolve_annotation"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("RESOLVE_A")), event);
}

/// Emitted when link_image_to_record is called.
pub fn emit_link_image_to_record(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "link_image_to_record"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("LINK_IMAG")), event);
}

/// Emitted when record_radiation_dose is called.
pub fn emit_record_radiation_dose(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "record_radiation_dose"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("RECORD_RA")), event);
}

/// Emitted when list_images_by_patient is called.
pub fn emit_list_images_by_patient(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_images_by_patient"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("LIST_IMAG")), event);
}

/// Emitted when list_images_by_modality_hash is called.
pub fn emit_list_images_by_modality_hash(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_images_by_modality_hash"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("LIST_IMAG")), event);
}

/// Emitted when list_images_by_body_part_hash is called.
pub fn emit_list_images_by_body_part_hash(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_images_by_body_part_hash"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("LIST_IMAG")), event);
}

/// Emitted when list_annotations_for_image is called.
pub fn emit_list_annotations_for_image(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "list_annotations_for_image"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("LIST_ANNO")), event);
}

/// Emitted when create_study is called.
pub fn emit_create_study(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_study"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("CREATE_ST")), event);
}

/// Emitted when assign_reader is called.
pub fn emit_assign_reader(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_reader"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("ASSIGN_RE")), event);
}

/// Emitted when assign_arbitrator is called.
pub fn emit_assign_arbitrator(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "assign_arbitrator"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("ASSIGN_AR")), event);
}

/// Emitted when link_ai_results is called.
pub fn emit_link_ai_results(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "link_ai_results"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("LINK_AI_R")), event);
}

/// Emitted when submit_reader_report is called.
pub fn emit_submit_reader_report(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_reader_report"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("SUBMIT_RE")), event);
}

/// Emitted when finalize_study is called.
pub fn emit_finalize_study(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "finalize_study"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("FINALIZE_")), event);
}

/// Emitted when amend_study is called.
pub fn emit_amend_study(env: &Env, caller: &Address) {
    let event = MedicalImagingEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: MedicalImagingEventData {
            user: caller.clone(),
            action: String::from_str(env, "amend_study"),
        },
    };
    env.events()
        .publish((symbol_short!("MEDIMG"), symbol_short!("AMEND_STU")), event);
}
