use crate::types::{DataKey, RiskProfile};
use soroban_sdk::{Address, Env};

pub struct TransactionMonitor;

impl TransactionMonitor {
    /// Observes transaction volume and frequency for a given user.
    pub fn observe_velocity(env: &Env, user: &Address, amount: i128) -> u32 {
        let profile = env
            .storage()
            .persistent()
            .get::<DataKey, RiskProfile>(&DataKey::UserRisk(user.clone()))
            .unwrap_or(RiskProfile {
                user: user.clone(),
                risk_score: 0,
                last_checked: 0,
                last_risk_level: crate::types::RiskLevel::Safe,
                violation_count: 0,
                is_blacklisted: false,
            });

        let mut score: u32 = 0;
        // Logic for checking frequency and abnormal volumes
        if amount > 100_000_000 {
            // Example: >10k XLM
            score += 500;
        }

        if profile.violation_count > 5 {
            score += 2000;
        }

        score
    }
}
