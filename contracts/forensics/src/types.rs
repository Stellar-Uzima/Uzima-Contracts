use soroban_sdk::{contracttype, Address, Bytes, BytesN, Map, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ActivityType {
    Transaction = 0,
    ContractCall = 1,
    GovernanceAction = 2,
    AdministrativeAction = 3,
    SuspiciousBehavior = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum ThreatLevel {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct ForensicEvidence {
    pub id: u64,
    pub timestamp: u64,
    pub actor: Address,
    pub activity_type: ActivityType,
    pub location_hash: BytesN<32>, // Hash of where the event occurred
    pub evidence_data: Bytes,     // Serialized evidence details
    pub threat_level: ThreatLevel,
    pub is_preserved: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct PatternAnalysis {
    pub pattern_id: String,
    pub occurrences: u32,
    pub last_seen: u64,
    pub risk_score: u32, // 0-10000 (bps)
}

#[derive(Clone)]
#[contracttype]
pub struct InvestigationReport {
    pub case_id: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub evidence_ids: Vec<u64>,
    pub findings: String,
    pub status: String, // "Open", "Closed", "Escalated"
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    EvidenceCount,
    Evidence(u64),
    Pattern(String),
    Investigator(Address),
    ReportCount,
    Report(u64),
    Blacklist(Address),
    WhitelistedContracts(Address),
}
