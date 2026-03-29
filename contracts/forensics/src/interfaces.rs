use soroban_sdk::{Env, Address, Bytes, BytesN, String, Vec};
use crate::types::{ActivityType, ThreatLevel, PatternAnalysis, InvestigationReport};

pub trait IOnChainForensics {
    fn initialize(env: Env, admin: Address);
    fn collect_evidence(
        env: Env,
        actor: Address,
        activity: ActivityType,
        location: BytesN<32>,
        evidence_data: Bytes,
        threat: ThreatLevel,
    ) -> u64;
    fn analyze_pattern(env: Env, pattern_id: String) -> PatternAnalysis;
    fn detect_suspicious(env: Env, actor: Address, threshold: u32) -> bool;
    fn generate_report(
        env: Env,
        admin: Address,
        start: u64,
        end: u64,
        evidence_ids: Vec<u64>,
        findings: String,
    ) -> u64;
}
