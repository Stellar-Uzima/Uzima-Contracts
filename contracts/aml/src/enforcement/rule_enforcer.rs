use crate::types::{AMLRule, DataKey, RiskProfile};
use soroban_sdk::{Address, Env};

pub struct RuleEnforcer;

impl RuleEnforcer {
    /// Enforces restrictive actions based on current user risk profile.
    pub fn enforce_limits(env: &Env, user: &Address, amount: i128) -> bool {
        let profile = env
            .storage()
            .persistent()
            .get::<DataKey, RiskProfile>(&DataKey::UserRisk(user.clone()))
            .unwrap();

        if profile.is_blacklisted {
            return false;
        }

        if profile.risk_score >= 8000 && amount > 1_000_000 {
            // High risk users have strict limits
            return false;
        }
        true
    }

    /// Triggers automated alerts when specific rules are tripped.
    pub fn alert_violations(env: &Env, rule_id: u32, user: &Address) {
        let rule: AMLRule = env
            .storage()
            .instance()
            .get(&DataKey::Rule(rule_id))
            .expect("Rule not found");

        if rule.is_enabled {
            // Logic to emit a system-level event
            crate::AntiMoneyLaundering::log_violation_internal(env, user, rule_id);
        }
    }
}
