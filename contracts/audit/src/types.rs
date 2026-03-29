use soroban_sdk::{contracttype, Address, Bytes, BytesN, Map, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AuditType {
    Event = 0,
    StateChange = 1,
    AdminAction = 2,
    SecurityAlert = 3,
    ComplianceReport = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditRecord {
    pub id: u64,
    pub timestamp: u64,
    pub actor: Address,
    pub audit_type: AuditType,
    pub target_contract: Option<Address>,
    pub action_hash: BytesN<32>,
    pub previous_state_hash: Option<BytesN<32>>,
    pub current_state_hash: BytesN<32>,
    pub metadata: Map<String, String>,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditSummary {
    pub start_time: u64,
    pub end_time: u64,
    pub total_records: u64,
    pub event_count: u32,
    pub admin_action_count: u32,
    pub root_hash: BytesN<32>, // Merkle tree root or rolling hash for integrity
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    RecordCount,
    Record(u64),
    ContractAudits(Address),
    UserAudits(Address),
    RollingHash,
    Config,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditConfig {
    pub archive_threshold: u64,
    pub enabled_types: Vec<AuditType>,
}
