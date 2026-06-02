//! # IheIntegration Events Module
//!
//! Standardized event emissions for the ihe_integration contract.
//! Topic naming convention: (IHE, ACTION)

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
pub struct IheIntegrationEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct IheIntegrationEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: IheIntegrationEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("INIT")), event);
}

/// Emitted when xds_register_document is called.
pub fn emit_xds_register_document(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xds_register_document"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XDS_REGIS")), event);
}

/// Emitted when xds_deprecate_document is called.
pub fn emit_xds_deprecate_document(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xds_deprecate_document"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XDS_DEPRE")), event);
}

/// Emitted when xds_query_documents is called.
pub fn emit_xds_query_documents(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xds_query_documents"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XDS_QUERY")), event);
}

/// Emitted when xds_retrieve_document is called.
pub fn emit_xds_retrieve_document(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xds_retrieve_document"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XDS_RETRI")), event);
}

/// Emitted when xds_submit_document_set is called.
pub fn emit_xds_submit_document_set(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xds_submit_document_set"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XDS_SUBMI")), event);
}

/// Emitted when pix_register_patient is called.
pub fn emit_pix_register_patient(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pix_register_patient"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("PIX_REGIS")), event);
}

/// Emitted when pix_query_identifiers is called.
pub fn emit_pix_query_identifiers(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pix_query_identifiers"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("PIX_QUERY")), event);
}

/// Emitted when pix_merge_patients is called.
pub fn emit_pix_merge_patients(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pix_merge_patients"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("PIX_MERGE")), event);
}

/// Emitted when pdq_register_demographics is called.
pub fn emit_pdq_register_demographics(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pdq_register_demographics"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("PDQ_REGIS")), event);
}

/// Emitted when pdq_query is called.
pub fn emit_pdq_query(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pdq_query"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("PDQ_QUERY")), event);
}

/// Emitted when pdq_get_demographics is called.
pub fn emit_pdq_get_demographics(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "pdq_get_demographics"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("PDQ_GET_D")), event);
}

/// Emitted when atna_log_event is called.
pub fn emit_atna_log_event(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "atna_log_event"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("ATNA_LOG_")), event);
}

/// Emitted when atna_get_event is called.
pub fn emit_atna_get_event(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "atna_get_event"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("ATNA_GET_")), event);
}

/// Emitted when atna_authenticate_node is called.
pub fn emit_atna_authenticate_node(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "atna_authenticate_node"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("ATNA_AUTH")), event);
}

/// Emitted when xca_register_gateway is called.
pub fn emit_xca_register_gateway(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xca_register_gateway"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XCA_REGIS")), event);
}

/// Emitted when xca_initiate_query is called.
pub fn emit_xca_initiate_query(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xca_initiate_query"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XCA_INITI")), event);
}

/// Emitted when mpi_register_master_patient is called.
pub fn emit_mpi_register_master_patient(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "mpi_register_master_patient"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("MPI_REGIS")), event);
}

/// Emitted when mpi_find_patient is called.
pub fn emit_mpi_find_patient(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "mpi_find_patient"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("MPI_FIND_")), event);
}

/// Emitted when xdr_send_document is called.
pub fn emit_xdr_send_document(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xdr_send_document"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XDR_SEND_")), event);
}

/// Emitted when xdm_record_media_package is called.
pub fn emit_xdm_record_media_package(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "xdm_record_media_package"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("XDM_RECOR")), event);
}

/// Emitted when ct_record_time_sync is called.
pub fn emit_ct_record_time_sync(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "ct_record_time_sync"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("CT_RECORD")), event);
}

/// Emitted when bppc_register_consent is called.
pub fn emit_bppc_register_consent(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "bppc_register_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("BPPC_REGI")), event);
}

/// Emitted when bppc_revoke_consent is called.
pub fn emit_bppc_revoke_consent(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "bppc_revoke_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("BPPC_REVO")), event);
}

/// Emitted when bppc_verify_consent is called.
pub fn emit_bppc_verify_consent(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "bppc_verify_consent"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("BPPC_VERI")), event);
}

/// Emitted when dsg_sign_document is called.
pub fn emit_dsg_sign_document(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "dsg_sign_document"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("DSG_SIGN_")), event);
}

/// Emitted when dsg_verify_signature is called.
pub fn emit_dsg_verify_signature(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "dsg_verify_signature"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("DSG_VERIF")), event);
}

/// Emitted when dsg_get_document_signatures is called.
pub fn emit_dsg_get_document_signatures(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "dsg_get_document_signatures"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("DSG_GET_D")), event);
}

/// Emitted when hpd_register_provider is called.
pub fn emit_hpd_register_provider(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "hpd_register_provider"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("HPD_REGIS")), event);
}

/// Emitted when hpd_get_provider is called.
pub fn emit_hpd_get_provider(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "hpd_get_provider"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("HPD_GET_P")), event);
}

/// Emitted when svs_register_value_set is called.
pub fn emit_svs_register_value_set(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "svs_register_value_set"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("SVS_REGIS")), event);
}

/// Emitted when svs_get_value_set_by_oid is called.
pub fn emit_svs_get_value_set_by_oid(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "svs_get_value_set_by_oid"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("SVS_GET_V")), event);
}

/// Emitted when connectathon_record_test is called.
pub fn emit_connectathon_record_test(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "connectathon_record_test"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("CONNECTAT")), event);
}

/// Emitted when connectathon_get_profile_results is called.
pub fn emit_connectathon_get_profile_results(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "connectathon_get_profile_results"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("CONNECTAT")), event);
}

/// Emitted when connectathon_is_compliant is called.
pub fn emit_connectathon_is_compliant(env: &Env, caller: &Address) {
    let event = IheIntegrationEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IheIntegrationEventData {
            user: caller.clone(),
            action: String::from_str(env, "connectathon_is_compliant"),
        },
    };
    env.events()
        .publish((symbol_short!("IHE"), symbol_short!("CONNECTAT")), event);
}
