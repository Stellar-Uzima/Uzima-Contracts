use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

#[contracttype]
pub struct EventEnvelope<T> {
    pub contract: Address,
    pub name: String,
    pub version: u32,
    pub body: T,
}

#[contracttype]
pub struct AuditContext {
    pub actor: Address,
    pub timestamp: u64,
    pub block_height: u64,
}

#[contracttype]
pub struct UserCreatedEvent {
    pub audit: AuditContext,
    pub user: Address,
    pub role: String,
}

#[contracttype]
pub struct UserRoleUpdatedEvent {
    pub audit: AuditContext,
    pub user: Address,
    pub new_role: String,
    pub previous_role: Option<String>,
}

#[contracttype]
pub struct UserDeactivatedEvent {
    pub audit: AuditContext,
    pub user: Address,
}

#[contracttype]
pub struct RecordCreatedEvent {
    pub audit: AuditContext,
    pub record_id: u64,
    pub patient: Address,
    pub doctor: Address,
    pub is_confidential: bool,
    pub category: String,
    pub tags: Vec<String>,
}

#[contracttype]
pub struct RecordAccessedEvent {
    pub audit: AuditContext,
    pub record_id: u64,
    pub patient: Address,
}

#[contracttype]
pub struct AccessRequestedEvent {
    pub audit: AuditContext,
    pub requester: Address,
    pub patient: Address,
    pub record_id: u64,
    pub purpose: String,
    pub credential_hash: Option<String>,
}

#[contracttype]
pub struct AccessGrantedEvent {
    pub audit: AuditContext,
    pub granter: Address,
    pub requester: Address,
    pub patient: Address,
    pub record_id: u64,
    pub purpose: String,
    pub credential_hash: Option<String>,
}

#[contracttype]
pub struct EmergencyAccessGrantedEvent {
    pub audit: AuditContext,
    pub granter: Address,
    pub grantee: Address,
    pub patient: Address,
    pub record_scope: Vec<u64>,
    pub expires_at: u64,
}

#[contracttype]
pub struct ContractPausedEvent {
    pub audit: AuditContext,
}

#[contracttype]
pub struct ContractUnpausedEvent {
    pub audit: AuditContext,
}

#[contracttype]
pub struct RecoveryProposedEvent {
    pub audit: AuditContext,
    pub proposal_id: u64,
    pub token_contract: Address,
    pub recipient: Address,
    pub amount: i128,
}

#[contracttype]
pub struct RecoveryApprovedEvent {
    pub audit: AuditContext,
    pub proposal_id: u64,
}

#[contracttype]
pub struct RecoveryExecutedEvent {
    pub audit: AuditContext,
    pub proposal_id: u64,
    pub token_contract: Address,
    pub recipient: Address,
    pub amount: i128,
}

#[contracttype]
pub struct AiConfigUpdatedEvent {
    pub audit: AuditContext,
    pub ai_coordinator: Address,
}

#[contracttype]
pub struct AnomalyScoreSubmittedEvent {
    pub audit: AuditContext,
    pub record_id: u64,
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub score_bps: u32,
    pub model_version: String,
}

#[contracttype]
pub struct RiskScoreSubmittedEvent {
    pub audit: AuditContext,
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub score_bps: u32,
    pub model_version: String,
}

#[contracttype]
pub struct AiAnalysisTriggeredEvent {
    pub audit: AuditContext,
    pub record_id: u64,
    pub patient: Address,
}

#[contracttype]
pub struct HealthCheckEvent {
    pub audit: AuditContext,
    pub status: String,
    pub gas_used: u64,
}