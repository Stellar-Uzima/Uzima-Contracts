#![no_std]

pub mod analysis;
pub mod detection;
pub mod evidence;
pub mod interfaces;
pub mod libraries;
pub mod types;

#[cfg(test)]
mod test;

use crate::types::{
    ActivityType, DataKey, ForensicEvidence, InvestigationReport, PatternAnalysis, ThreatLevel,
};
use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Map, String, Symbol, Vec,
};

#[contract]
pub struct OnChainForensics;

#[contractimpl]
impl OnChainForensics {
    /// Initialize with administrator
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::EvidenceCount, &0u64);
        env.storage().instance().set(&DataKey::ReportCount, &0u64);
    }

    /// Log a forensic event and collect evidence
    pub fn collect_evidence(
        env: Env,
        actor: Address,
        activity: ActivityType,
        location: BytesN<32>,
        evidence_data: Bytes,
        threat: ThreatLevel,
    ) -> u64 {
        actor.require_auth();

        let evidence_id = Self::next_id(&env, &DataKey::EvidenceCount);
        let evidence = ForensicEvidence {
            id: evidence_id,
            timestamp: env.ledger().timestamp(),
            actor: actor.clone(),
            activity_type: activity,
            location_hash: location,
            evidence_data,
            threat_level: threat,
            is_preserved: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Evidence(evidence_id), &evidence);

        // Update patterns automatically
        Self::update_patterns_internal(&env, actor, activity, threat);

        // Emit events for off-chain indexing
        env.events().publish(
            (symbol_short!("FORENSIC"), symbol_short!("COLLECT")),
            (
                evidence_id,
                evidence.actor,
                evidence.activity_type,
                evidence.threat_level,
            ),
        );

        evidence_id
    }

    /// Analyze activity patterns for potential threats
    pub fn analyze_pattern(env: Env, pattern_id: String) -> PatternAnalysis {
        env.storage()
            .persistent()
            .get(&DataKey::Pattern(pattern_id))
            .unwrap_or(PatternAnalysis {
                pattern_id: String::from_str(&env, "unknown"),
                occurrences: 0,
                last_seen: 0,
                risk_score: 0,
            })
    }

    /// Detect suspicious activity using adaptive algorithms (simplified)
    pub fn detect_suspicious(env: Env, actor: Address, threshold: u32) -> bool {
        let pattern_id = String::from_str(&env, "suspicious_volume");
        let analysis = Self::analyze_pattern(env.clone(), pattern_id);

        // High risk score or too many occurrences trigger detection
        analysis.risk_score >= threshold || analysis.occurrences > 100
    }

    /// Generate an immutable forensic report
    pub fn generate_report(
        env: Env,
        admin: Address,
        start: u64,
        end: u64,
        evidence_ids: Vec<u64>,
        findings: String,
    ) -> u64 {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let report_id = Self::next_id(&env, &DataKey::ReportCount);
        let report = InvestigationReport {
            case_id: report_id,
            start_timestamp: start,
            end_timestamp: end,
            evidence_ids,
            findings,
            status: String::from_str(&env, "Open"),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Report(report_id), &report);

        env.events().publish(
            (symbol_short!("FORENSIC"), symbol_short!("REPORT")),
            (report_id, report.start_timestamp, report.end_timestamp),
        );

        report_id
    }

    /// Update an investigation status
    pub fn update_investigation(env: Env, admin: Address, report_id: u64, status: String) -> bool {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let mut report: InvestigationReport = env
            .storage()
            .persistent()
            .get(&DataKey::Report(report_id))
            .expect("Report not found");

        report.status = status;
        env.storage()
            .persistent()
            .set(&DataKey::Report(report_id), &report);

        true
    }

    /// Blacklist a suspicious address after forensic evidence
    pub fn blacklist_actor(env: Env, admin: Address, actor_to_blacklist: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        env.storage()
            .instance()
            .set(&DataKey::Blacklist(actor_to_blacklist.clone()), &true);

        env.events().publish(
            (symbol_short!("FORENSIC"), symbol_short!("B_LIST")),
            actor_to_blacklist,
        );
    }

    /// Internal helper to update pattern analysis
    fn update_patterns_internal(
        env: &Env,
        actor: Address,
        activity: ActivityType,
        threat: ThreatLevel,
    ) {
        let key = String::from_str(env, "general_monitoring");
        let mut pattern: PatternAnalysis = env
            .storage()
            .persistent()
            .get(&DataKey::Pattern(key.clone()))
            .unwrap_or(PatternAnalysis {
                pattern_id: key.clone(),
                occurrences: 0,
                last_seen: 0,
                risk_score: 0,
            });

        pattern.occurrences = pattern.occurrences.saturating_add(1);
        pattern.last_seen = env.ledger().timestamp();

        // Dynamic risk scoring logic
        let mut added_risk = match threat {
            ThreatLevel::None => 0,
            ThreatLevel::Low => 10,
            ThreatLevel::Medium => 50,
            ThreatLevel::High => 200,
            ThreatLevel::Critical => 1000,
        };

        if activity == ActivityType::SuspiciousBehavior {
            added_risk = added_risk.saturating_mul(2);
        }

        pattern.risk_score = pattern.risk_score.saturating_add(added_risk).min(10000);

        env.storage()
            .persistent()
            .set(&DataKey::Pattern(key), &pattern);
    }

    /// Private helper to get the administrator
    fn require_admin(env: &Env, actor: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        if admin != *actor {
            panic!("Unauthorized: Admin access required");
        }
    }

    /// Helper to increment counters
    fn next_id(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }
}
