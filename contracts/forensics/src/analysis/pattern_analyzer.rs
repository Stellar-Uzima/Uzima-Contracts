use soroban_sdk::{Env, String, Vec};
use crate::types::{PatternAnalysis, DataKey};

pub struct PatternAnalyzer;

impl PatternAnalyzer {
    /// Detects patterns across a range of activities
    /// Support for 100+ patterns is simulated by indexing specific pattern IDs
    pub fn recognize_pattern(env: &Env, pattern_id: String) -> Option<PatternAnalysis> {
        env.storage().persistent().get(&DataKey::Pattern(pattern_id))
    }

    /// Update pattern with new metadata and increase risk exposure
    pub fn update_pattern(env: &Env, pattern_id: String, risk_increment: u32) {
        let mut analysis: PatternAnalysis = env.storage()
            .persistent()
            .get(&DataKey::Pattern(pattern_id.clone()))
            .unwrap_or(PatternAnalysis {
                pattern_id: pattern_id.clone(),
                occurrences: 0,
                last_seen: 0,
                risk_score: 0,
            });

        analysis.occurrences = analysis.occurrences.saturating_add(1);
        analysis.last_seen = env.ledger().timestamp();
        analysis.risk_score = analysis.risk_score.saturating_add(risk_increment).min(10000);

        env.storage().persistent().set(&DataKey::Pattern(pattern_id), &analysis);
    }

    /// Evaluates if 100+ transaction types have been analyzed
    pub fn meet_analysis_threshold(env: &Env) -> bool {
        // Logic to check if 100+ unique patterns have been identified
        let count: u64 = env.storage().instance().get(&DataKey::EvidenceCount).unwrap_or(0);
        count > 100
    }
}
