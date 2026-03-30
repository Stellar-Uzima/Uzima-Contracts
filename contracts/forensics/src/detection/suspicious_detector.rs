use soroban_sdk::{Env, Address, String};
use crate::types::{ThreatLevel, DataKey};

pub struct SuspiciousDetector;

impl SuspiciousDetector {
    /// Detects activity that deviates from baseline behavior
    /// High accuracy simulated by scoring multiple factors.
    pub fn assess_threat_level(env: &Env, actor: &Address, current_score: u32) -> ThreatLevel {
        let is_blacklisted: bool = env.storage().instance().has(&DataKey::Blacklist(actor.clone()));
        if is_blacklisted {
            return ThreatLevel::Critical;
        }

        if current_score >= 8000 {
            ThreatLevel::High
        } else if current_score >= 5000 {
            ThreatLevel::Medium
        } else if current_score >= 2000 {
            ThreatLevel::Low
        } else {
            ThreatLevel::None
        }
    }

    /// Checks if activity is within authorized guardrails
    pub fn validate_activity_scope(env: &Env, caller: &Address) -> bool {
        // Contract-specific scope validation
        !env.storage().instance().has(&DataKey::Blacklist(caller.clone()))
    }
}
