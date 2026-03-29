#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    UserAlreadyForgotten = 3,
    RuleNotConfigured = 4,
    RightToBeForgottenDisabled = 5,
}

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
    Rule(String),
    Consent(Address, String),
    AuditLogs(Address),
    Forgotten(Address),
}

#[contract]
pub struct RegulatoryComplianceContract;

#[contractimpl]
impl RegulatoryComplianceContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn set_rule(env: Env, framework: String, rule: ComplianceRule) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Rule(framework), &rule);
        Ok(())
    }

    pub fn get_rule(env: Env, framework: String) -> Option<ComplianceRule> {
        env.storage().instance().get(&DataKey::Rule(framework))
    }

    pub fn grant_consent(env: Env, user: Address, action: String) -> Result<(), Error> {
        user.require_auth();
        if Self::is_forgotten(&env, user.clone()) {
            return Err(Error::UserAlreadyForgotten);
        }
        env.storage()
            .persistent()
            .set(&DataKey::Consent(user, action), &true);
        Ok(())
    }

    pub fn revoke_consent(env: Env, user: Address, action: String) -> Result<(), Error> {
        user.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Consent(user, action), &false);
        Ok(())
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

    pub fn get_audit_logs(env: Env, user: Address) -> Result<Vec<AuditLog>, Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::AuditLogs(user))
            .unwrap_or(Vec::new(&env)))
    }

    pub fn invoke_right_to_be_forgotten(env: Env, user: Address) -> Result<(), Error> {
        user.require_auth();
        let framework = String::from_str(&env, "GDPR");
        let rule = Self::get_rule(env.clone(), framework);
        if let Some(r) = rule {
            if !r.right_to_be_forgotten {
                return Err(Error::RightToBeForgottenDisabled);
            }
            env.storage()
                .persistent()
                .set(&DataKey::Forgotten(user.clone()), &true);
            Ok(())
        } else {
            Err(Error::RuleNotConfigured)
        }
    }

    pub fn is_forgotten(env: &Env, user: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Forgotten(user))
            .unwrap_or(false)
    }
}
