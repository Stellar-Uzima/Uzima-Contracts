use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum FactorType {
    Password = 0,
    Biometric = 1,
    HardwareKey = 2,
    EmailCode = 3,
    SMSCode = 4,
    AuthenticatorApp = 5, // TOTP
    MultiSig = 6,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum AuthStatus {
    Pending = 0,
    Partial = 1,
    Verified = 2,
    Expired = 3,
    Revoked = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct AuthFactor {
    pub factor_id: u64,
    pub user: Address,
    pub factor_type: FactorType,
    pub provider_address: Option<Address>, // External contract for verification
    pub metadata: String,                  // Public identifier (e.g., "YubiKey 5")
    pub created_at: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct AuthSession {
    pub session_id: u64,
    pub user: Address,
    pub required_factors: Vec<FactorType>,
    pub verified_factors: Vec<FactorType>,
    pub expires_at: u64,
    pub status: AuthStatus,
}

#[derive(Clone)]
#[contracttype]
pub struct RecoveryVault {
    pub user: Address,
    pub recovery_hashes: Vec<soroban_sdk::BytesN<32>>, // Salted hashes of recovery codes
    pub backup_address: Option<Address>,
    pub unlock_at: u64, // Time-lock for recovery
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    UserFactors(Address),
    UserSession(Address),
    Recovery(Address),
    NextFactorId,
    NextSessionId,
    GlobalConfig,
    AuditLogCount,
    AuditEntry(u64),
    TokenStore(Address),
    RefreshTokenStore(Address),
}

#[derive(Clone)]
#[contracttype]
pub struct MFAConfig {
    pub session_ttl: u64,
    pub min_factors_for_critical_op: u32,
    pub recovery_delay: u64,
}

// =============================================================================
// Token Expiration and Refresh Strategy Types
// =============================================================================

/// Classification of token types used across the platform.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum TokenType {
    /// Short-lived access token for API calls.
    AccessToken,
    /// Long-lived refresh token used to obtain new access tokens.
    RefreshToken,
    /// Identity verification token for cross-contract authentication.
    IdentityToken,
    /// Session-bound MFA verification token.
    MfaSessionToken,
}

/// Policy governing when and how tokens expire.
#[derive(Clone)]
#[contracttype]
pub struct TokenExpiration {
    /// Type of token this policy applies to.
    pub token_type: TokenType,
    /// Time-to-live in seconds from issuance.
    pub ttl_secs: u64,
    /// Maximum lifetime in seconds regardless of activity (0 = no max).
    pub max_lifetime_secs: u64,
    /// Whether the token can be refreshed after expiry.
    pub refreshable: bool,
    /// Grace period in seconds after expiry during which refresh is still allowed.
    pub grace_period_secs: u64,
}

/// Strategy for refreshing expired tokens.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum RefreshStrategy {
    /// Sliding window: refresh extends TTL from the time of refresh.
    SlidingWindow,
    /// Fixed window: token expires at a fixed time regardless of refresh.
    FixedWindow,
    /// Rotation: each refresh issues a new token and invalidates the old one.
    Rotation,
    /// Hybrid: sliding window with a hard max lifetime cap.
    Hybrid,
}

/// Complete refresh policy combining expiration and strategy.
#[derive(Clone)]
#[contracttype]
pub struct TokenRefreshPolicy {
    /// Expiration configuration.
    pub expiration: TokenExpiration,
    /// Refresh strategy to use.
    pub strategy: RefreshStrategy,
    /// Maximum number of consecutive refreshes before requiring re-authentication (0 = unlimited).
    pub max_refresh_count: u32,
    /// Whether to notify on refresh (emit event).
    pub notify_on_refresh: bool,
}

/// An issued token with its metadata.
#[derive(Clone)]
#[contracttype]
pub struct IssuedToken {
    /// Unique token identifier (hash).
    pub token_id: BytesN<32>,
    /// Owner of the token.
    pub owner: Address,
    /// Type of token.
    pub token_type: TokenType,
    /// Timestamp when token was issued.
    pub issued_at: u64,
    /// Timestamp when token expires.
    pub expires_at: u64,
    /// Maximum lifetime timestamp.
    pub max_lifetime_at: u64,
    /// Number of times this token has been refreshed.
    pub refresh_count: u32,
    /// Whether the token is still valid (not revoked).
    pub is_valid: bool,
    /// Parent token ID (for rotation chains).
    pub parent_token: Option<BytesN<32>>,
}

/// Result of a token refresh operation.
#[derive(Clone)]
#[contracttype]
pub struct TokenRefreshResult {
    /// Whether the refresh was successful.
    pub success: bool,
    /// The new token (if rotation strategy).
    pub new_token: Option<IssuedToken>,
    /// Old token that was invalidated (if rotation).
    pub invalidated_token: Option<BytesN<32>>,
    /// Error message if refresh failed.
    pub error: Option<String>,
}

/// Default token expiration policies for each token type.
pub fn default_token_expiration(token_type: TokenType) -> TokenExpiration {
    match token_type {
        TokenType::AccessToken => TokenExpiration {
            token_type,
            ttl_secs: 900,        // 15 minutes
            max_lifetime_secs: 3600, // 1 hour max
            refreshable: true,
            grace_period_secs: 60, // 1 minute grace
        },
        TokenType::RefreshToken => TokenExpiration {
            token_type,
            ttl_secs: 86400 * 7, // 7 days
            max_lifetime_secs: 86400 * 30, // 30 days max
            refreshable: true,
            grace_period_secs: 3600, // 1 hour grace
        },
        TokenType::IdentityToken => TokenExpiration {
            token_type,
            ttl_secs: 3600, // 1 hour
            max_lifetime_secs: 86400, // 24 hours max
            refreshable: true,
            grace_period_secs: 300, // 5 minutes grace
        },
        TokenType::MfaSessionToken => TokenExpiration {
            token_type,
            ttl_secs: 300, // 5 minutes
            max_lifetime_secs: 900, // 15 minutes max
            refreshable: false,
            grace_period_secs: 0,
        },
    }
}

/// Default refresh policy for a token type.
pub fn default_refresh_policy(token_type: TokenType) -> TokenRefreshPolicy {
    TokenRefreshPolicy {
        expiration: default_token_expiration(token_type),
        strategy: match token_type {
            TokenType::AccessToken => RefreshStrategy::SlidingWindow,
            TokenType::RefreshToken => RefreshStrategy::Rotation,
            TokenType::IdentityToken => RefreshStrategy::Hybrid,
            TokenType::MfaSessionToken => RefreshStrategy::FixedWindow,
        },
        max_refresh_count: match token_type {
            TokenType::AccessToken => 10,
            TokenType::RefreshToken => 5,
            TokenType::IdentityToken => 3,
            TokenType::MfaSessionToken => 0,
        },
        notify_on_refresh: matches!(token_type, TokenType::RefreshToken | TokenType::IdentityToken),
    }
}
