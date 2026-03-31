use soroban_sdk::{contracttype, Address, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum RiskLevel {
    Safe = 0,
    Low = 1,
    Elevated = 2,
    High = 3,
    Sanctioned = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct AMLRule {
    pub rule_id: u32,
    pub name: String,
    pub description: String,
    pub threshold: i128,
    pub risk_contribution: u32, // bps (basis points)
    pub is_enabled: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct RiskProfile {
    pub user: Address,
    pub risk_score: u32, // 0-10000
    pub last_checked: u64,
    pub last_risk_level: RiskLevel,
    pub violation_count: u32,
    pub is_blacklisted: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct AMLReport {
    pub report_id: u64,
    pub timestamp: u64,
    pub issuer: Address,
    pub subject: Address,
    pub risk_score_at_issue: u32,
    pub incident_summary: String,
    pub evidence_ref: String, // Reference to off-chain or forensics evidence
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Rule(u32),
    UserRisk(Address),
    NextReportId,
    Report(u64),
    Whitelist(Address),
    GlobalStats,
}

#[derive(Clone)]
#[contracttype]
pub struct GlobalAMLStats {
    pub total_monitored: u32,
    pub active_violations: u32,
    pub blacklisted_count: u32,
}
