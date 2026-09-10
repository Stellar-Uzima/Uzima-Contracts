use soroban_sdk::{
    contracttype, Address, BytesN, Env, IntoVal, String, TryFromVal, TryIntoVal, Val, Vec,
};

/// Typed envelope for structured contract events.
///
/// Not a `#[contracttype]` — the derive does not support generics — so the
/// `Val` conversion that `publish` requires is implemented manually below:
/// each field converts through `IntoVal`, and the struct itself becomes a
/// Soroban `Vec` of its four field values, matching the wire shape the
/// `#[contracttype]` derive would have produced.
#[derive(Clone)]
pub struct EventEnvelope<T> {
    pub contract: Address,
    pub name: String,
    pub version: u32,
    pub body: T,
}

impl<T> IntoVal<Env, Val> for EventEnvelope<T>
where
    T: IntoVal<Env, Val>,
{
    fn into_val(&self, env: &Env) -> Val {
        (
            self.contract.clone(),
            self.name.clone(),
            self.version,
            T::into_val(&self.body, env),
        )
            .into_val(env)
    }
}

impl<T> TryFromVal<Env, Val> for EventEnvelope<T>
where
    T: TryFromVal<Env, Val>,
{
    type Error = soroban_sdk::ConversionError;

    fn try_from_val(env: &Env, val: &Val) -> Result<Self, Self::Error> {
        let (contract, name, version, body): (Address, String, u32, T) =
            val.try_into_val(env)?;
        Ok(EventEnvelope {
            contract,
            name,
            version,
            body,
        })
    }
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

#[contracttype]
pub struct MetadataUpdatedEvent {
    pub audit: AuditContext,
    pub record_id: u64,
    pub patient: Address,
    pub new_version: u32,
    pub tag_count: u32,
    pub custom_field_count: u32,
}

#[contracttype]
pub struct RecordRolledBackEvent {
    pub audit: AuditContext,
    pub record_id: u64,
    pub patient: Address,
    pub from_version: u32,
    pub to_version: u32,
}

#[contracttype]
pub struct TraditionalRecordAddedEvent {
    pub audit: AuditContext,
    pub record_id: u64,
    pub patient: Address,
    pub practice_type: String,
}

#[contracttype]
pub struct PermissionGrantedEvent {
    pub audit: AuditContext,
    pub granter: Address,
    pub grantee: Address,
    pub permission: u32,
    pub expires_at: u64,
    pub is_delegatable: bool,
}

#[contracttype]
pub struct PermissionRevokedEvent {
    pub audit: AuditContext,
    pub revoker: Address,
    pub grantee: Address,
    pub permission: u32,
}

#[contracttype]
pub struct DataQualityValidatedEvent {
    pub audit: AuditContext,
    pub validator: Address,
    pub record_id: u64,
    pub quality_score: u32,
    pub is_fhir_compliant: bool,
    pub issue_count: u32,
}