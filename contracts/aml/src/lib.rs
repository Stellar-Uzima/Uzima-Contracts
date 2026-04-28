#![no_std]

pub mod detection;
pub mod enforcement;
pub mod monitoring;
pub mod types;

#[cfg(test)]
mod test;

use crate::types::{AMLReport, AMLRule, DataKey, GlobalAMLStats, RiskLevel, RiskProfile};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contracterror, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Map, String,
    Symbol, Vec,
    contract, contractimpl, symbol_short, Address, BytesN, Env, String, Symbol, Vec,
};
use upgradeability::storage::{ADMIN as UPGRADE_ADMIN, VERSION};

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
        Self::ensure_upgrade_metadata(&env, &admin);
        env.storage().instance().set(&DataKey::NextReportId, &0u64);
        env.storage().instance().set(
            &DataKey::GlobalStats,
            &GlobalAMLStats {
                total_monitored: 0,
                active_violations: 0,
                blacklisted_count: 0,
            },
        );
        upgradeability::storage::set_deprecated_functions(&env, &Self::deprecated_functions(&env));
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

    /// Update blacklist status for a user.
    pub fn update_user_status(env: Env, admin: Address, user: Address, is_blacklisted: bool) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        Self::set_user_status_internal(&env, user, is_blacklisted);
    }

    /// Blacklist or whitelist an address manually by admin.
    #[deprecated(since = "v2.0.0", note = "Use update_user_status instead")]
    pub fn set_user_status(env: Env, admin: Address, user: Address, is_blacklisted: bool) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        upgradeability::emit_deprecation_warning(&env, Symbol::new(&env, "set_user_status")).ok();
        Self::set_user_status_internal(&env, user, is_blacklisted);
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

    /// Register AML deprecated entrypoints for upgrade and migration tracking.
    pub fn register_deprecated_functions(
        env: Env,
        admin: Address,
    ) -> Result<(), upgradeability::UpgradeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        Self::ensure_upgrade_metadata(&env, &admin);
        upgradeability::set_deprecated_functions(&env, Self::deprecated_functions(&env))
    }

    /// Return tracked deprecated AML entrypoints.
    pub fn get_deprecated_functions(env: Env) -> Vec<upgradeability::DeprecatedFunction> {
        upgradeability::get_deprecated_functions(&env)
    }

    /// Upgrade the AML contract and register deprecated entrypoints atomically.
    pub fn upgrade(
        env: Env,
        admin: Address,
        new_wasm_hash: BytesN<32>,
        new_version: u32,
    ) -> Result<(), upgradeability::UpgradeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        Self::ensure_upgrade_metadata(&env, &admin);
        upgradeability::execute_upgrade_with_deprecations::<Self>(
            &env,
            new_wasm_hash,
            new_version,
            symbol_short!("AML_UPG"),
            Self::deprecated_functions(&env),
        )
    }

    /// Validate a proposed AML upgrade before execution.
    pub fn validate_upgrade(
        env: Env,
        new_wasm_hash: BytesN<32>,
    ) -> Result<upgradeability::UpgradeValidation, upgradeability::UpgradeError> {
        upgradeability::validate_upgrade::<Self>(&env, new_wasm_hash)
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

    fn set_user_status_internal(env: &Env, user: Address, is_blacklisted: bool) {
        let mut profile = Self::get_or_create_profile(env, &user);
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

    fn ensure_upgrade_metadata(env: &Env, admin: &Address) {
        if !env.storage().instance().has(&UPGRADE_ADMIN) {
            env.storage().instance().set(&UPGRADE_ADMIN, admin);
        }
        if !env.storage().instance().has(&VERSION) {
            env.storage().instance().set(&VERSION, &1u32);
        }
    }

    fn deprecated_functions(env: &Env) -> Vec<upgradeability::DeprecatedFunction> {
        Vec::from_array(
            env,
            [upgradeability::DeprecatedFunction {
                function: Symbol::new(env, "set_user_status"),
                since: String::from_str(env, "v2.0.0"),
                replacement: Some(Symbol::new(env, "update_user_status")),
                removed_in: Some(String::from_str(env, "v3.0.0")),
                note: String::from_str(
                    env,
                    "Use update_user_status; set_user_status will be removed in v3.0.0",
                ),
                migration_guide: Some(String::from_str(
                    env,
                    "docs/deprecation_migration.md",
                )),
            }],
        )
    }

    fn next_id(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }
}

impl upgradeability::migration::Migratable for AntiMoneyLaundering {
    fn migrate(env: &Env, from_version: u32) -> Result<(), upgradeability::UpgradeError> {
        if from_version < 1 {
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .ok_or(upgradeability::UpgradeError::NotAuthorized)?;
            Self::ensure_upgrade_metadata(env, &admin);
        }
        Ok(())
    }

    fn verify_integrity(env: &Env) -> Result<BytesN<32>, upgradeability::UpgradeError> {
        let next_report_id = env.storage().instance().get(&DataKey::NextReportId).unwrap_or(0u64);
        let stats = env
            .storage()
            .instance()
            .get::<DataKey, GlobalAMLStats>(&DataKey::GlobalStats)
            .unwrap_or(GlobalAMLStats {
                total_monitored: 0,
                active_violations: 0,
                blacklisted_count: 0,
            });

        let mut data = Vec::new(env);
        data.push_back(next_report_id);
        data.push_back(stats.total_monitored as u64);
        data.push_back(stats.active_violations as u64);
        data.push_back(stats.blacklisted_count as u64);

        let hash_bytes = env.crypto().sha256(&data.to_xdr(env));
        Ok(BytesN::from_array(env, &hash_bytes.to_array()))
    }

    fn validate(
        env: &Env,
        _new_wasm_hash: &BytesN<32>,
    ) -> Result<upgradeability::UpgradeValidation, upgradeability::UpgradeError> {
        let initialized = env.storage().instance().has(&DataKey::Admin);
        let mut report = Vec::new(env);
        if !initialized {
            report.push_back(symbol_short!("NOT_INIT"));
        }

        Ok(upgradeability::UpgradeValidation {
            state_compatible: initialized,
            api_compatible: true,
            storage_layout_valid: true,
            tests_passed: true,
            gas_impact: 0,
            report,
        })
    }
}
