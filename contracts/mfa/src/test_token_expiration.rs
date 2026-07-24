#![cfg(test)]

use super::types::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_default_token_expiration_access_token() {
    let exp = default_token_expiration(TokenType::AccessToken);
    assert_eq!(exp.ttl_secs, 900);
    assert_eq!(exp.max_lifetime_secs, 3600);
    assert!(exp.refreshable);
    assert_eq!(exp.grace_period_secs, 60);
}

#[test]
fn test_default_token_expiration_refresh_token() {
    let exp = default_token_expiration(TokenType::RefreshToken);
    assert_eq!(exp.ttl_secs, 86400 * 7);
    assert_eq!(exp.max_lifetime_secs, 86400 * 30);
    assert!(exp.refreshable);
    assert_eq!(exp.grace_period_secs, 3600);
}

#[test]
fn test_default_token_expiration_identity_token() {
    let exp = default_token_expiration(TokenType::IdentityToken);
    assert_eq!(exp.ttl_secs, 3600);
    assert_eq!(exp.max_lifetime_secs, 86400);
    assert!(exp.refreshable);
}

#[test]
fn test_default_token_expiration_mfa_session() {
    let exp = default_token_expiration(TokenType::MfaSessionToken);
    assert_eq!(exp.ttl_secs, 300);
    assert!(!exp.refreshable);
    assert_eq!(exp.grace_period_secs, 0);
}

#[test]
fn test_default_refresh_policy_access_token() {
    let policy = default_refresh_policy(TokenType::AccessToken);
    assert_eq!(policy.strategy, RefreshStrategy::SlidingWindow);
    assert_eq!(policy.max_refresh_count, 10);
    assert!(!policy.notify_on_refresh);
}

#[test]
fn test_default_refresh_policy_refresh_token() {
    let policy = default_refresh_policy(TokenType::RefreshToken);
    assert_eq!(policy.strategy, RefreshStrategy::Rotation);
    assert_eq!(policy.max_refresh_count, 5);
    assert!(policy.notify_on_refresh);
}

#[test]
fn test_default_refresh_policy_identity_token() {
    let policy = default_refresh_policy(TokenType::IdentityToken);
    assert_eq!(policy.strategy, RefreshStrategy::Hybrid);
    assert_eq!(policy.max_refresh_count, 3);
}

#[test]
fn test_default_refresh_policy_mfa_session() {
    let policy = default_refresh_policy(TokenType::MfaSessionToken);
    assert_eq!(policy.strategy, RefreshStrategy::FixedWindow);
    assert_eq!(policy.max_refresh_count, 0);
}

#[test]
fn test_token_expiration_fields() {
    let exp = TokenExpiration {
        token_type: TokenType::AccessToken,
        ttl_secs: 600,
        max_lifetime_secs: 7200,
        refreshable: true,
        grace_period_secs: 120,
    };
    assert_eq!(exp.token_type, TokenType::AccessToken);
    assert_eq!(exp.ttl_secs, 600);
    assert_eq!(exp.max_lifetime_secs, 7200);
    assert!(exp.refreshable);
    assert_eq!(exp.grace_period_secs, 120);
}

#[test]
fn test_refresh_strategy_equality() {
    assert_eq!(RefreshStrategy::SlidingWindow, RefreshStrategy::SlidingWindow);
    assert_ne!(RefreshStrategy::SlidingWindow, RefreshStrategy::Rotation);
    assert_ne!(RefreshStrategy::FixedWindow, RefreshStrategy::Hybrid);
}

#[test]
fn test_token_type_equality() {
    assert_eq!(TokenType::AccessToken, TokenType::AccessToken);
    assert_ne!(TokenType::AccessToken, TokenType::RefreshToken);
    assert_ne!(TokenType::IdentityToken, TokenType::MfaSessionToken);
}

#[test]
fn test_token_refresh_policy_fields() {
    let policy = TokenRefreshPolicy {
        expiration: default_token_expiration(TokenType::AccessToken),
        strategy: RefreshStrategy::Hybrid,
        max_refresh_count: 20,
        notify_on_refresh: true,
    };
    assert_eq!(policy.strategy, RefreshStrategy::Hybrid);
    assert_eq!(policy.max_refresh_count, 20);
    assert!(policy.notify_on_refresh);
}
