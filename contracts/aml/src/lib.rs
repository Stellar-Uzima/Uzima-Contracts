#![no_std]

pub mod detection;
pub mod enforcement;
pub mod monitoring;
pub mod types;

#[cfg(test)]
mod test;

use crate::types::{AMLReport, AMLRule, DataKey, GlobalAMLStats, RiskLevel, RiskProfile};
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Map, String,
    Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
}

#[contract]
pub struct AntiMoneyLaundering;

#[contractimpl]
impl AntiMoneyLaundering {
    /// Initialize AML with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextReportId, &0u64);
        env.storage().instance().set(
            &DataKey::GlobalStats,
            &GlobalAMLStats {
                total_monitored: 0,
                active_violations: 0,
                blacklisted_count: 0,
            },
        );
        env.events().publish((symbol_short!("Init"),), admin);
        Ok(())
    }

    /// Configure an AML rule
    pub fn configure_rule(
        env: Env,
        admin: Address,
        id: u32,
        name: String,
        description: String,
        threshold: i128,
        risk_contribution: u32,
    ) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let rule = AMLRule {
            rule_id: id,
            name,
            description,
            threshold,
            risk_contribution: risk_contribution.min(10000),
            is_enabled: true,
        };
        env.storage().instance().set(&DataKey::Rule(id), &rule);
    }

    /// Monitor a transaction and update risk profile
    pub fn monitor_transaction(
        env: Env,
        user: Address,
        amount: i128,
        _target: Option<Address>,
    ) -> RiskLevel {
        // Only monitored calls allowed (or system calls)
        // For simplicity, we assume internal platform calls trigger this

        let mut profile = Self::get_or_create_profile(&env, &user);

        // Example monitoring logic: velocity check (simplified)
        // If amount > threshold of any active rule, increase risk
        // Let's check rule #1 for demo
        if let Some(rule1) = env
            .storage()
            .instance()
            .get::<DataKey, AMLRule>(&DataKey::Rule(1))
        {
            if rule1.is_enabled && amount >= rule1.threshold {
                profile.risk_score = profile
                    .risk_score
                    .saturating_add(rule1.risk_contribution)
                    .min(10000);
                profile.violation_count += 1;
            }
        }

        profile.last_checked = env.ledger().timestamp();
        profile.last_risk_level = Self::compute_risk_level(profile.risk_score);

        if profile.risk_score >= 9000 {
            profile.is_blacklisted = true;
        }

        env.storage()
            .persistent()
            .set(&DataKey::UserRisk(user), &profile);
        profile.last_risk_level
    }

    /// Check if a user is compliant with platform AML policy
    pub fn is_compliant(env: Env, user: Address) -> bool {
        let profile = Self::get_or_create_profile(&env, &user);
        !profile.is_blacklisted && profile.risk_score < 7500
    }

    /// Blacklist or Whitelist an address manually by admin
    pub fn set_user_status(env: Env, admin: Address, user: Address, is_blacklisted: bool) {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let mut profile = Self::get_or_create_profile(&env, &user);
        profile.is_blacklisted = is_blacklisted;
        if is_blacklisted {
            profile.risk_score = 10000;
            profile.last_risk_level = RiskLevel::Sanctioned;
        } else {
            profile.risk_score = 0;
            profile.last_risk_level = RiskLevel::Safe;
        }

        env.storage()
            .persistent()
            .set(&DataKey::UserRisk(user.clone()), &profile);

        env.events().publish(
            (symbol_short!("AML"), symbol_short!("STATUS")),
            (user, is_blacklisted),
        );
    }

    /// Generate an AML compliance report for regulatory use
    pub fn report_incident(
        env: Env,
        admin: Address,
        subject: Address,
        summary: String,
        evidence: String,
    ) -> u64 {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let report_id = Self::next_id(&env, &DataKey::NextReportId);
        let profile = Self::get_or_create_profile(&env, &subject);

        let report = AMLReport {
            report_id,
            timestamp: env.ledger().timestamp(),
            issuer: admin.clone(),
            subject,
            risk_score_at_issue: profile.risk_score,
            incident_summary: summary,
            evidence_ref: evidence,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Report(report_id), &report);

        env.events()
            .publish((symbol_short!("AML"), symbol_short!("REPORT")), report_id);
        report_id
    }

    /// Helper to retrieve profile or create default
    fn get_or_create_profile(env: &Env, user: &Address) -> RiskProfile {
        env.storage()
            .persistent()
            .get(&DataKey::UserRisk(user.clone()))
            .unwrap_or(RiskProfile {
                user: user.clone(),
                risk_score: 0,
                last_checked: 0,
                last_risk_level: RiskLevel::Safe,
                violation_count: 0,
                is_blacklisted: false,
            })
    }

    fn compute_risk_level(score: u32) -> RiskLevel {
        if score >= 9000 {
            RiskLevel::Sanctioned
        } else if score >= 7000 {
            RiskLevel::High
        } else if score >= 4000 {
            RiskLevel::Elevated
        } else if score >= 1000 {
            RiskLevel::Low
        } else {
            RiskLevel::Safe
        }
    }

    fn require_admin(env: &Env, actor: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        if admin != *actor {
            panic!("Unauthorized");
        }
    }

    fn next_id(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }
}
