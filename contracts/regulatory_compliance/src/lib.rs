#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataResidency {
    Global,
    EU,
    US,
    Local(String),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceRule {
    pub require_consent: bool,
    pub right_to_be_forgotten: bool,
    pub residency: DataResidency,
    pub strict_auditing: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditLog {
    pub action: String,
    pub actor: Address,
    pub timestamp: u64,
    pub details: String,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Rule(String),             // Framework or region name
    Consent(Address, String), // User Address -> Action -> bool
    AuditLogs(Address),       // Logs for a specific user
    Forgotten(Address),       // Has this user invoked right-to-be-forgotten?
}

#[contract]
pub struct RegulatoryComplianceContract;

#[contractimpl]
impl RegulatoryComplianceContract {
    pub fn initialize(env: Env, admin: Address) {
        assert!(
            !env.storage().instance().has(&DataKey::Admin),
            "Already initialized"
        );
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn set_rule(env: Env, framework: String, rule: ComplianceRule) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Rule(framework), &rule);
    }

    pub fn get_rule(env: Env, framework: String) -> Option<ComplianceRule> {
        env.storage().instance().get(&DataKey::Rule(framework))
    }

    pub fn grant_consent(env: Env, user: Address, action: String) {
        user.require_auth();
        assert!(
            !Self::is_forgotten(&env, user.clone()),
            "User has been forgotten"
        );
        env.storage()
            .persistent()
            .set(&DataKey::Consent(user, action), &true);
    }

    pub fn revoke_consent(env: Env, user: Address, action: String) {
        user.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Consent(user, action), &false);
    }

    pub fn has_consent(env: Env, user: Address, action: String) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Consent(user, action))
            .unwrap_or(false)
    }

    pub fn log_audit(env: Env, user: Address, action: String, details: String) {
        let framework = String::from_str(&env, "HIPAA");
        if let Some(rule) = Self::get_rule(env.clone(), framework) {
            if rule.strict_auditing {
                let mut logs: Vec<AuditLog> = env
                    .storage()
                    .persistent()
                    .get(&DataKey::AuditLogs(user.clone()))
                    .unwrap_or(Vec::new(&env));
                let timestamp = env.ledger().timestamp();
                logs.push_back(AuditLog {
                    action,
                    actor: user.clone(),
                    timestamp,
                    details,
                });
                env.storage()
                    .persistent()
                    .set(&DataKey::AuditLogs(user), &logs);
            }
        }
    }

    pub fn get_audit_logs(env: Env, user: Address) -> Vec<AuditLog> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage()
            .persistent()
            .get(&DataKey::AuditLogs(user))
            .unwrap_or(Vec::new(&env))
    }

    pub fn invoke_right_to_be_forgotten(env: Env, user: Address) {
        user.require_auth();
        let framework = String::from_str(&env, "GDPR");
        let rule = Self::get_rule(env.clone(), framework);
        if let Some(r) = rule {
            assert!(
                r.right_to_be_forgotten,
                "Right to be forgotten not enabled in rules"
            );
            env.storage()
                .persistent()
                .set(&DataKey::Forgotten(user.clone()), &true);

            // Note: In a real system we would remove the data or flag it across all contracts.
            // Here we just mark it.
        } else {
            panic!("GDPR rule not configured");
        }
    }

    pub fn is_forgotten(env: &Env, user: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Forgotten(user))
            .unwrap_or(false)
    }
}
