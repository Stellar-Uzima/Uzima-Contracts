use super::event_schema::{
    AccessGrantedEvent, AccessRequestedEvent, AiAnalysisTriggeredEvent, AiConfigUpdatedEvent,
    AnomalyScoreSubmittedEvent, AuditContext, ContractPausedEvent, ContractUnpausedEvent,
    EmergencyAccessGrantedEvent, EventEnvelope, HealthCheckEvent, MetadataUpdatedEvent,
    RecordAccessedEvent, RecordCreatedEvent, RecordRolledBackEvent, RecoveryApprovedEvent,
    RecoveryExecutedEvent, RecoveryProposedEvent, RiskScoreSubmittedEvent,
    TraditionalRecordAddedEvent, UserCreatedEvent, UserDeactivatedEvent, UserRoleUpdatedEvent,
};
use soroban_sdk::{symbol_short, Address, BytesN, Env, String, Vec};

// ==================== Event Publishing Functions ====================

pub fn emit_user_created(
    env: &Env,
    admin: Address,
    new_user: Address,
    role: &str,
    _did_ref: Option<String>,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "user_created"),
        version: 1,
        body: UserCreatedEvent {
            audit: AuditContext {
                actor: admin.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            user: new_user.clone(),
            role: String::from_str(env, role),
        },
    };
    env.events()
        .publish((symbol_short!("USER_ADD"), admin, new_user), event);
}

pub fn emit_user_role_updated(
    env: &Env,
    admin: Address,
    target_user: Address,
    new_role: &str,
    previous_role: Option<&str>,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "user_role_updated"),
        version: 1,
        body: UserRoleUpdatedEvent {
            audit: AuditContext {
                actor: admin.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            user: target_user.clone(),
            new_role: String::from_str(env, new_role),
            previous_role: previous_role.map(|r| String::from_str(env, r)),
        },
    };
    env.events()
        .publish((symbol_short!("ROLE_UPD"), admin, target_user), event);
}

pub fn emit_user_deactivated(env: &Env, admin: Address, target_user: Address) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "user_deactivated"),
        version: 1,
        body: UserDeactivatedEvent {
            audit: AuditContext {
                actor: admin.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            user: target_user.clone(),
        },
    };
    env.events()
        .publish((symbol_short!("USR_DEACT"), admin, target_user), event);
}

pub fn emit_record_created(
    env: &Env,
    doctor: Address,
    record_id: u64,
    patient: Address,
    is_confidential: bool,
    category: String,
    tags: Vec<String>,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "record_created"),
        version: 1,
        body: RecordCreatedEvent {
            audit: AuditContext {
                actor: doctor.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            record_id,
            patient: patient.clone(),
            doctor: doctor.clone(),
            is_confidential,
            category,
            tags,
        },
    };
    env.events()
        .publish((symbol_short!("REC_NEW"), doctor, patient), event);
}

pub fn emit_record_accessed(env: &Env, accessor: Address, record_id: u64, patient: Address) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "record_accessed"),
        version: 1,
        body: RecordAccessedEvent {
            audit: AuditContext {
                actor: accessor.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            record_id,
            patient: patient.clone(),
        },
    };
    env.events()
        .publish((symbol_short!("REC_ACC"), accessor, patient), event);
}

pub fn emit_access_requested(
    env: &Env,
    requester: Address,
    patient: Address,
    record_id: u64,
    purpose: String,
    credential_hash: Option<String>,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "access_requested"),
        version: 1,
        body: AccessRequestedEvent {
            audit: AuditContext {
                actor: requester.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            requester: requester.clone(),
            patient: patient.clone(),
            record_id,
            purpose,
            credential_hash,
        },
    };
    env.events()
        .publish((symbol_short!("ACC_REQ"), requester, patient), event);
}

pub fn emit_access_granted(
    env: &Env,
    granter: Address,
    requester: Address,
    patient: Address,
    record_id: u64,
    purpose: String,
    credential_hash: Option<String>,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "access_granted"),
        version: 1,
        body: AccessGrantedEvent {
            audit: AuditContext {
                actor: granter.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            granter: granter.clone(),
            requester: requester.clone(),
            patient: patient.clone(),
            record_id,
            purpose,
            credential_hash,
        },
    };
    env.events()
        .publish((symbol_short!("ACC_GRANT"), granter, requester), event);
}

pub fn emit_emergency_access_granted(
    env: &Env,
    granter: Address,
    grantee: Address,
    patient: Address,
    record_scope: Vec<u64>,
    expires_at: u64,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "emergency_access_granted"),
        version: 1,
        body: EmergencyAccessGrantedEvent {
            audit: AuditContext {
                actor: granter.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            granter: granter.clone(),
            grantee: grantee.clone(),
            patient: patient.clone(),
            record_scope,
            expires_at,
        },
    };
    env.events()
        .publish((symbol_short!("EM_GRANT"), granter, grantee), event);
}

pub fn emit_contract_paused(env: &Env, admin: Address) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "contract_paused"),
        version: 1,
        body: ContractPausedEvent {
            audit: AuditContext {
                actor: admin.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
        },
    };
    env.events().publish((symbol_short!("PAUSED"), admin), event);
}

pub fn emit_contract_unpaused(env: &Env, admin: Address) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "contract_unpaused"),
        version: 1,
        body: ContractUnpausedEvent {
            audit: AuditContext {
                actor: admin.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
        },
    };
    env.events()
        .publish((symbol_short!("UNPAUSED"), admin), event);
}

pub fn emit_recovery_proposed(
    env: &Env,
    proposer: Address,
    proposal_id: u64,
    token_contract: Address,
    recipient: Address,
    amount: i128,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "recovery_proposed"),
        version: 1,
        body: RecoveryProposedEvent {
            audit: AuditContext {
                actor: proposer.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            proposal_id,
            token_contract,
            recipient,
            amount,
        },
    };
    env.events()
        .publish((symbol_short!("REC_PROP"), proposer), event);
}

pub fn emit_recovery_approved(env: &Env, approver: Address, proposal_id: u64) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "recovery_approved"),
        version: 1,
        body: RecoveryApprovedEvent {
            audit: AuditContext {
                actor: approver.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            proposal_id,
        },
    };
    env.events()
        .publish((symbol_short!("REC_APPR"), approver), event);
}

pub fn emit_recovery_executed(
    env: &Env,
    executor: Address,
    proposal_id: u64,
    token_contract: Address,
    recipient: Address,
    amount: i128,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "recovery_executed"),
        version: 1,
        body: RecoveryExecutedEvent {
            audit: AuditContext {
                actor: executor.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            proposal_id,
            token_contract,
            recipient,
            amount,
        },
    };
    env.events()
        .publish((symbol_short!("REC_EXEC"), executor), event);
}

pub fn emit_ai_config_updated(env: &Env, admin: Address, ai_coordinator: Address) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "ai_config_updated"),
        version: 1,
        body: AiConfigUpdatedEvent {
            audit: AuditContext {
                actor: admin.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            ai_coordinator,
        },
    };
    env.events()
        .publish((symbol_short!("AI_CFG"), admin), event);
}

pub fn emit_anomaly_score_submitted(
    env: &Env,
    ai_coordinator: Address,
    record_id: u64,
    patient: Address,
    model_id: BytesN<32>,
    score_bps: u32,
    model_version: String,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "anomaly_score_submitted"),
        version: 1,
        body: AnomalyScoreSubmittedEvent {
            audit: AuditContext {
                actor: ai_coordinator.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            record_id,
            patient,
            model_id,
            score_bps,
            model_version,
        },
    };
    env.events()
        .publish((symbol_short!("ANOMALY"), ai_coordinator), event);
}

pub fn emit_risk_score_submitted(
    env: &Env,
    ai_coordinator: Address,
    patient: Address,
    model_id: BytesN<32>,
    score_bps: u32,
    model_version: String,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "risk_score_submitted"),
        version: 1,
        body: RiskScoreSubmittedEvent {
            audit: AuditContext {
                actor: ai_coordinator.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            patient,
            model_id,
            score_bps,
            model_version,
        },
    };
    env.events()
        .publish((symbol_short!("RISK_SCR"), ai_coordinator), event);
}

pub fn emit_ai_analysis_triggered(env: &Env, record_id: u64, patient: Address) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "ai_analysis_triggered"),
        version: 1,
        body: AiAnalysisTriggeredEvent {
            audit: AuditContext {
                actor: patient.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            record_id,
            patient,
        },
    };
    env.events()
        .publish((symbol_short!("AI_TRIG"), patient), event);
}

pub fn emit_health_check(env: &Env, status: String, gas_used: u64) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "health_check"),
        version: 1,
        body: HealthCheckEvent {
            audit: AuditContext {
                actor: env.current_contract_address(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            status,
            gas_used,
        },
    };
    env.events()
        .publish((symbol_short!("HEALTH"),), event);
}

pub fn emit_metadata_updated(
    env: &Env,
    caller: Address,
    record_id: u64,
    patient: Address,
    new_version: u32,
    tag_count: u32,
    custom_field_count: u32,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "metadata_updated"),
        version: 1,
        body: MetadataUpdatedEvent {
            audit: AuditContext {
                actor: caller.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            record_id,
            patient: patient.clone(),
            new_version,
            tag_count,
            custom_field_count,
        },
    };
    env.events()
        .publish((symbol_short!("META_UPD"), caller, patient), event);
}

pub fn emit_record_rolled_back(
    env: &Env,
    caller: Address,
    record_id: u64,
    patient: Address,
    from_version: u32,
    to_version: u32,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "record_rolled_back"),
        version: 1,
        body: RecordRolledBackEvent {
            audit: AuditContext {
                actor: caller.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            record_id,
            patient: patient.clone(),
            from_version,
            to_version,
        },
    };
    env.events()
        .publish((symbol_short!("REC_RBACK"), caller, patient), event);
}

pub fn emit_traditional_record_added(
    env: &Env,
    caller: Address,
    record_id: u64,
    patient: Address,
    practice_type: String,
) {
    let event = EventEnvelope {
        contract: env.current_contract_address(),
        name: String::from_str(env, "traditional_record_added"),
        version: 1,
        body: TraditionalRecordAddedEvent {
            audit: AuditContext {
                actor: caller.clone(),
                timestamp: env.ledger().timestamp(),
                block_height: env.ledger().sequence() as u64,
            },
            record_id,
            patient: patient.clone(),
            practice_type,
        },
    };
    env.events()
        .publish((symbol_short!("TRAD_REC"), caller, patient), event);
}