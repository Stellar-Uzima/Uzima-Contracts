#![no_std]

pub mod types;
pub mod interfaces;
pub mod libraries;
pub mod factors;
pub mod verification;
pub mod recovery;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, String, Symbol, Vec,
};
use crate::types::{FactorType, AuthStatus, AuthFactor, AuthSession, RecoveryVault, DataKey, MFAConfig};

#[contract]
pub struct MultiFactorAuth;

#[contractimpl]
impl MultiFactorAuth {
    /// Initialize with global MFA configuration
    pub fn initialize(env: Env, admin: Address, config: MFAConfig) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::GlobalConfig, &config);
        env.storage().instance().set(&DataKey::NextFactorId, &0u64);
        env.storage().instance().set(&DataKey::NextSessionId, &0u64);
        env.storage().instance().set(&DataKey::AuditLogCount, &0u64);
    }

    /// Add a new authentication factor for the caller
    pub fn add_factor(
        env: Env,
        user: Address,
        factor: FactorType,
        provider: Option<Address>,
        metadata: String,
    ) -> u64 {
        user.require_auth();

        let factor_id = Self::next_id(&env, &DataKey::NextFactorId);
        let auth_factor = AuthFactor {
            factor_id,
            user: user.clone(),
            factor_type: factor,
            provider_address: provider,
            metadata,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };

        let mut user_factors: Vec<AuthFactor> = env.storage()
            .persistent()
            .get(&DataKey::UserFactors(user.clone()))
            .unwrap_or(Vec::new(&env));
        
        user_factors.push_back(auth_factor);
        env.storage().persistent().set(&DataKey::UserFactors(user), &user_factors);

        Self::log_auth_event(&env, factor_id, symbol_short!("FACTOR_A"));
        factor_id
    }

    /// Initiate an authentication session requiring specific factors
    pub fn start_session(env: Env, user: Address, required: Vec<FactorType>) -> u64 {
        user.require_auth();

        let cfg: MFAConfig = env.storage()
            .instance()
            .get(&DataKey::GlobalConfig)
            .expect("Not configured");

        let session_id = Self::next_id(&env, &DataKey::NextSessionId);
        let session = AuthSession {
            session_id,
            user: user.clone(),
            required_factors: required,
            verified_factors: Vec::new(&env),
            expires_at: env.ledger().timestamp().saturating_add(cfg.session_ttl),
            status: AuthStatus::Pending,
        };

        env.storage().persistent().set(&DataKey::UserSession(user), &session);
        
        Self::log_auth_event(&env, session_id, symbol_short!("SES_START"));
        session_id
    }

    /// Verify a specific factor for an existing session
    pub fn verify_mfa_factor(
        env: Env,
        user: Address,
        factor: FactorType,
        proof: Bytes,
    ) -> bool {
        user.require_auth();

        let mut session: AuthSession = env.storage()
            .persistent()
            .get(&DataKey::UserSession(user.clone()))
            .expect("Session not found");

        if env.ledger().timestamp() > session.expires_at {
            session.status = AuthStatus::Expired;
            env.storage().persistent().set(&DataKey::UserSession(user), &session);
            return false;
        }

        // Check if factor is in the required list
        let mut found = false;
        for req_f in session.required_factors.iter() {
            if req_f == factor {
                found = true;
                break;
            }
        }
        if !found { return false; }

        // Perform verification (simplification: we accept any non-empty bytes proof for now)
        // In reality, we'd check against provider address or metadata hash.
        if !proof.is_empty() {
            session.verified_factors.push_back(factor);
            
            // Check if all required factors are verified
            if session.verified_factors.len() >= session.required_factors.len() {
                session.status = AuthStatus::Verified;
            } else {
                session.status = AuthStatus::Partial;
            }
            
            env.storage().persistent().set(&DataKey::UserSession(user), &session);
            
            Self::log_auth_event(&env, session.session_id, symbol_short!("F_VERIFY"));
            return true;
        }

        false
    }

    /// Check if the user has a valid verified MFA session
    pub fn is_authenticated(env: Env, user: Address) -> bool {
        let session: Option<AuthSession> = env.storage()
            .persistent()
            .get(&DataKey::UserSession(user));
        
        match session {
            Some(s) => s.status == AuthStatus::Verified && env.ledger().timestamp() <= s.expires_at,
            None => false,
        }
    }

    /// Recovery mechanism for lost factors
    pub fn initiate_recovery(env: Env, user: Address, secret_hash: BytesN<32>) {
        user.require_auth();

        let cfg: MFAConfig = env.storage()
            .instance()
            .get(&DataKey::GlobalConfig)
            .expect("Not configured");

        let recovery = RecoveryVault {
            user: user.clone(),
            recovery_hashes: Vec::new(&env), // In a real app we'd pre-load this
            backup_address: None,
            unlock_at: env.ledger().timestamp().saturating_add(cfg.recovery_delay),
        };

        env.storage().persistent().set(&DataKey::Recovery(user), &recovery);
        Self::log_auth_event(&env, 0, symbol_short!("RECOVERY_I"));
    }

    /// Emergency override using admin signatures (multi-sig simulation)
    pub fn emergency_override(env: Env, admin: Address, target_user: Address) -> bool {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let mut session: AuthSession = env.storage()
            .persistent()
            .get(&DataKey::UserSession(target_user.clone()))
            .expect("No session to override");
        
        session.status = AuthStatus::Verified;
        env.storage().persistent().set(&DataKey::UserSession(target_user), &session);
        
        Self::log_auth_event(&env, session.session_id, symbol_short!("OVERRIDE"));
        true
    }

    /// Private helpers
    fn require_admin(env: &Env, actor: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Not initialized");
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

    fn log_auth_event(env: &Env, id: u64, topic: Symbol) {
        env.events().publish((symbol_short!("MFA"), topic), id);
    }
}
