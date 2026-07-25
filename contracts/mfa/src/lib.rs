#![no_std]
//! mfa - Healthcare smart contract on Stellar blockchain.

pub mod factors;
pub mod recovery;
pub mod types;
pub mod verification;

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_token_expiration;

use crate::types::{
    AuthFactor, AuthSession, AuthStatus, DataKey, FactorType, IssuedToken, MFAConfig,
    RecoveryVault, RefreshStrategy, TokenExpiration, TokenRefreshPolicy, TokenRefreshResult,
    TokenType,
};
use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, String, Symbol, Vec,
};

#[contract]
pub struct MultiFactorAuth;

#[contractimpl]
impl MultiFactorAuth {
    /// Initialize with global MFA configuration
    pub fn initialize(env: Env, admin: Address, config: MFAConfig) {
        governance_commons::init_guard(&env);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::GlobalConfig, &config);
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

        let mut user_factors: Vec<AuthFactor> = env
            .storage()
            .persistent()
            .get(&DataKey::UserFactors(user.clone()))
            .unwrap_or(Vec::new(&env));

        user_factors.push_back(auth_factor);
        env.storage()
            .persistent()
            .set(&DataKey::UserFactors(user), &user_factors);

        Self::log_auth_event(&env, factor_id, symbol_short!("FACTOR_A"));
        factor_id
    }

    /// Initiate an authentication session requiring specific factors
    pub fn start_session(env: Env, user: Address, required: Vec<FactorType>) -> u64 {
        user.require_auth();

        let cfg: MFAConfig = env
            .storage()
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

        env.storage()
            .persistent()
            .set(&DataKey::UserSession(user), &session);

        Self::log_auth_event(&env, session_id, symbol_short!("SES_START"));
        session_id
    }

    /// Verify a specific factor for an existing session
    pub fn verify_mfa_factor(env: Env, user: Address, factor: FactorType, proof: Bytes) -> bool {
        user.require_auth();

        let mut session: AuthSession = env
            .storage()
            .persistent()
            .get(&DataKey::UserSession(user.clone()))
            .expect("Session not found");

        if env.ledger().timestamp() > session.expires_at {
            session.status = AuthStatus::Expired;
            env.storage()
                .persistent()
                .set(&DataKey::UserSession(user), &session);
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
        if !found {
            return false;
        }

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

            env.storage()
                .persistent()
                .set(&DataKey::UserSession(user), &session);

            Self::log_auth_event(&env, session.session_id, symbol_short!("F_VERIFY"));
            return true;
        }

        false
    }

    /// Check if the user has a valid verified MFA session
    pub fn is_authenticated(env: Env, user: Address) -> bool {
        let session: Option<AuthSession> =
            env.storage().persistent().get(&DataKey::UserSession(user));

        match session {
            Some(s) => s.status == AuthStatus::Verified && env.ledger().timestamp() <= s.expires_at,
            None => false,
        }
    }

    /// Recovery mechanism for lost factors
    pub fn initiate_recovery(env: Env, user: Address, _secret_hash: BytesN<32>) {
        user.require_auth();

        let cfg: MFAConfig = env
            .storage()
            .instance()
            .get(&DataKey::GlobalConfig)
            .expect("Not configured");

        let recovery = RecoveryVault {
            user: user.clone(),
            recovery_hashes: Vec::new(&env), // In a real app we'd pre-load this
            backup_address: None,
            unlock_at: env.ledger().timestamp().saturating_add(cfg.recovery_delay),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Recovery(user), &recovery);
        Self::log_auth_event(&env, 0, symbol_short!("RECOVERY"));
    }

    /// Emergency override using admin signatures (multi-sig simulation)
    pub fn emergency_override(env: Env, admin: Address, target_user: Address) -> bool {
        admin.require_auth();
        Self::require_admin(&env, &admin);

        let mut session: AuthSession = env
            .storage()
            .persistent()
            .get(&DataKey::UserSession(target_user.clone()))
            .expect("No session to override");

        session.status = AuthStatus::Verified;
        env.storage()
            .persistent()
            .set(&DataKey::UserSession(target_user), &session);

        Self::log_auth_event(&env, session.session_id, symbol_short!("OVERRIDE"));
        true
    }

    /// Private helpers
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

    fn log_auth_event(env: &Env, id: u64, topic: Symbol) {
        env.events().publish((symbol_short!("MFA"), topic), id);
    }

    // =========================================================================
    // Token Expiration and Refresh Strategy
    // =========================================================================

    /// Issue a new token with the specified type and policy.
    pub fn issue_token(
        env: Env,
        owner: Address,
        token_type: TokenType,
    ) -> IssuedToken {
        owner.require_auth();

        let now = env.ledger().timestamp();
        let policy = Self::get_refresh_policy(&env, &token_type);
        let expiration = &policy.expiration;

        let max_lifetime_at = if expiration.max_lifetime_secs > 0 {
            now.saturating_add(expiration.max_lifetime_secs)
        } else {
            u64::MAX
        };

        let token_id = Self::generate_token_id(&env, &owner, token_type, now);

        let token = IssuedToken {
            token_id: token_id.clone(),
            owner: owner.clone(),
            token_type,
            issued_at: now,
            expires_at: now.saturating_add(expiration.ttl_secs),
            max_lifetime_at,
            refresh_count: 0,
            is_valid: true,
            parent_token: None,
        };

        env.storage()
            .persistent()
            .set(&DataKey::TokenStore(owner), &token);

        Self::log_auth_event(&env, 0, symbol_short!("TOK_ISSUE"));
        token
    }

    /// Check if a token is currently valid (not expired, not revoked).
    pub fn is_token_valid(env: Env, token_id: BytesN<32>, owner: Address) -> bool {
        let token: Option<IssuedToken> = env
            .storage()
            .persistent()
            .get(&DataKey::TokenStore(owner));

        match token {
            Some(t) if t.token_id == token_id && t.is_valid => {
                let now = env.ledger().timestamp();
                now <= t.expires_at || {
                    // Check grace period
                    let policy = Self::get_refresh_policy(&env, &t.token_type);
                    now <= t.expires_at.saturating_add(policy.expiration.grace_period_secs)
                }
            }
            _ => false,
        }
    }

    /// Refresh a token according to the configured strategy.
    pub fn refresh_token(
        env: Env,
        owner: Address,
        token_id: BytesN<32>,
    ) -> TokenRefreshResult {
        owner.require_auth();

        let token: IssuedToken = env
            .storage()
            .persistent()
            .get(&DataKey::TokenStore(owner.clone()))
            .map(|t: IssuedToken| t)
            .unwrap_or_else(|| panic!("Token not found"));

        if token.token_id != token_id {
            return TokenRefreshResult {
                success: false,
                new_token: None,
                invalidated_token: None,
                error: Some(String::from_str(&env, "token_id mismatch")),
            };
        }

        if !token.is_valid {
            return TokenRefreshResult {
                success: false,
                new_token: None,
                invalidated_token: None,
                error: Some(String::from_str(&env, "token revoked")),
            };
        }

        let now = env.ledger().timestamp();
        let policy = Self::get_refresh_policy(&env, &token.token_type);

        // Check max lifetime
        if token.max_lifetime_at < u64::MAX && now > token.max_lifetime_at {
            return TokenRefreshResult {
                success: false,
                new_token: None,
                invalidated_token: None,
                error: Some(String::from_str(&env, "max lifetime exceeded")),
            };
        }

        // Check refresh count
        if policy.max_refresh_count > 0 && token.refresh_count >= policy.max_refresh_count {
            return TokenRefreshResult {
                success: false,
                new_token: None,
                invalidated_token: None,
                error: Some(String::from_str(&env, "max refresh count exceeded")),
            };
        }

        // Check if within grace period (for expired tokens)
        if now > token.expires_at {
            if policy.expiration.grace_period_secs == 0
                || now > token.expires_at.saturating_add(policy.expiration.grace_period_secs)
            {
                return TokenRefreshResult {
                    success: false,
                    new_token: None,
                    invalidated_token: None,
                    error: Some(String::from_str(&env, "token expired beyond grace period")),
                };
            }
        }

        let new_expires_at = match policy.strategy {
            RefreshStrategy::SlidingWindow | RefreshStrategy::Hybrid => {
                now.saturating_add(policy.expiration.ttl_secs)
            }
            RefreshStrategy::FixedWindow => token.expires_at,
            RefreshStrategy::Rotation => now.saturating_add(policy.expiration.ttl_secs),
        };

        match policy.strategy {
            RefreshStrategy::Rotation => {
                // Invalidate old token
                let mut old_token = token.clone();
                old_token.is_valid = false;
                env.storage()
                    .persistent()
                    .set(&DataKey::TokenStore(owner.clone()), &old_token);

                // Issue new token
                let new_token_id = Self::generate_token_id(&env, &owner, token.token_type, now);
                let new_token = IssuedToken {
                    token_id: new_token_id,
                    owner: owner.clone(),
                    token_type: token.token_type,
                    issued_at: now,
                    expires_at: new_expires_at,
                    max_lifetime_at: token.max_lifetime_at,
                    refresh_count: token.refresh_count.saturating_add(1),
                    is_valid: true,
                    parent_token: Some(token.token_id.clone()),
                };

                env.storage()
                    .persistent()
                    .set(&DataKey::TokenStore(owner), &new_token);

                Self::log_auth_event(&env, 0, symbol_short!("TOK_REFRESH"));

                TokenRefreshResult {
                    success: true,
                    new_token: Some(new_token),
                    invalidated_token: Some(token.token_id),
                    error: None,
                }
            }
            _ => {
                // Update TTL in place
                let mut updated_token = token.clone();
                updated_token.expires_at = new_expires_at;
                updated_token.refresh_count = token.refresh_count.saturating_add(1);

                env.storage()
                    .persistent()
                    .set(&DataKey::TokenStore(owner), &updated_token);

                Self::log_auth_event(&env, 0, symbol_short!("TOK_REFRESH"));

                TokenRefreshResult {
                    success: true,
                    new_token: Some(updated_token),
                    invalidated_token: None,
                    error: None,
                }
            }
        }
    }

    /// Revoke a token immediately.
    pub fn revoke_token(env: Env, owner: Address, token_id: BytesN<32>) -> bool {
        owner.require_auth();

        let token: Option<IssuedToken> = env
            .storage()
            .persistent()
            .get(&DataKey::TokenStore(owner.clone()));

        match token {
            Some(mut t) if t.token_id == token_id => {
                t.is_valid = false;
                env.storage()
                    .persistent()
                    .set(&DataKey::TokenStore(owner), &t);
                Self::log_auth_event(&env, 0, symbol_short!("TOK_REVOKE"));
                true
            }
            _ => false,
        }
    }

    /// Get the refresh policy for a token type (uses defaults).
    fn get_refresh_policy(_env: &Env, token_type: &TokenType) -> TokenRefreshPolicy {
        crate::types::default_refresh_policy(*token_type)
    }

    /// Generate a deterministic token ID.
    fn generate_token_id(
        env: &Env,
        owner: &Address,
        token_type: TokenType,
        timestamp: u64,
    ) -> BytesN<32> {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_slice(env, b"UZIMA_TOKEN_V1"));
        payload.append(&Bytes::from_slice(env, &owner.to_array()));
        let type_byte = token_type as u8;
        payload.append(&Bytes::from_slice(env, &[type_byte]));
        payload.append(&Bytes::from_slice(env, &timestamp.to_be_bytes()));
        env.crypto().sha256(&payload).into()
    }
}
