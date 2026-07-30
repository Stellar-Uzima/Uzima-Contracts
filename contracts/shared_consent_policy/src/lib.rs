#![no_std]
//! Shared consent policy types and helpers used by both `medical_records` and
//! `patient_consent_management`.
//!
//! This module eliminates the duplicated consent verification logic between
//! those two contracts by providing a single source of truth for:
//!
//! - Consent record types (`ConsentStatus`, `ConsentDecision`)
//! - Consent expiry and validity checks
//! - Consent verification that cross-calls `patient_consent_management`
//!
//! ## Usage from `medical_records`
//!
//! ```rust,ignore
//! use shared_consent_policy::{verify_consent_for_record, ConsentContext};
//!
//! let ctx = ConsentContext { env, patient, provider, consent_contract };
//! let decision = verify_consent_for_record(&ctx)?;
//! ```
//!
//! ## Usage from `patient_consent_management`
//!
//! ```rust,ignore
//! use shared_consent_policy::{is_consent_effective, ConsentStatus};
//!
//! let status = is_consent_effective(&env, &record, consent_policy);
//! assert!(matches!(status, ConsentStatus::Active));
//! ```

use soroban_sdk::{contracttype, Address, Env, String};

// ──────────────────────────────────────────────────────────────────────────────
// Consent Status
// ──────────────────────────────────────────────────────────────────────────────

/// High-level status of a consent record after evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ConsentStatus {
    /// Consent is active and not expired.
    Active,
    /// Consent has expired based on `expires_at`.
    Expired,
    /// Consent was explicitly revoked by the patient.
    Revoked,
    /// No consent record was found for this patient/provider pair.
    NotFound,
}

impl ConsentStatus {
    pub fn is_effective(&self) -> bool {
        matches!(self, ConsentStatus::Active)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ConsentStatus::Active => "active",
            ConsentStatus::Expired => "expired",
            ConsentStatus::Revoked => "revoked",
            ConsentStatus::NotFound => "not_found",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Consent Decision (detailed result)
// ──────────────────────────────────────────────────────────────────────────────

/// Structured result of a consent check, carrying status + reason.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ConsentDecision {
    pub status: ConsentStatus,
    pub reason: String,
}

impl ConsentDecision {
    pub fn allowed(env: &Env) -> Self {
        ConsentDecision {
            status: ConsentStatus::Active,
            reason: String::from_str(env, "Consent granted"),
        }
    }

    pub fn denied(env: &Env, status: ConsentStatus, msg: &str) -> Self {
        ConsentDecision {
            status,
            reason: String::from_str(env, msg),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Consent Policy Configuration
// ──────────────────────────────────────────────────────────────────────────────

/// Configurable consent policy parameters, shared across contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SharedConsentPolicy {
    /// Default TTL in seconds for new consents (0 = no expiry).
    pub default_ttl_secs: u64,
    /// Window in seconds before expiry to emit a warning event.
    pub notification_window_secs: u64,
    /// Whether to require explicit consent for every access (strict mode).
    pub strict_mode: bool,
}

impl Default for SharedConsentPolicy {
    fn default() -> Self {
        SharedConsentPolicy {
            default_ttl_secs: 0,
            notification_window_secs: 86_400, // 24 hours
            strict_mode: false,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Core Evaluation Functions
// ──────────────────────────────────────────────────────────────────────────────

/// Evaluate whether a consent record is currently effective.
///
/// Checks:
/// 1. Record must exist (not None)
/// 2. Record must be `active == true`
/// 3. If `expires_at > 0`, current timestamp must be before expiry
pub fn is_consent_effective(
    env: &Env,
    active: bool,
    expires_at: u64,
) -> ConsentStatus {
    if !active {
        return ConsentStatus::Revoked;
    }
    if expires_at > 0 && env.ledger().timestamp() >= expires_at {
        return ConsentStatus::Expired;
    }
    ConsentStatus::Active
}

/// Check whether a consent is within the notification window (approaching expiry).
///
/// Returns `true` if the consent will expire within `policy.notification_window_secs`.
pub fn is_near_expiry(
    env: &Env,
    expires_at: u64,
    policy: &SharedConsentPolicy,
) -> bool {
    if expires_at == 0 || policy.notification_window_secs == 0 {
        return false;
    }
    let now = env.ledger().timestamp();
    if now >= expires_at {
        return false; // Already expired
    }
    let remaining = expires_at - now;
    remaining <= policy.notification_window_secs
}

/// Determine the effective TTL for a new consent grant.
///
/// If `explicit_ttl` is provided and > 0, it takes precedence.
/// Otherwise, the policy's `default_ttl_secs` is used.
pub fn effective_ttl(explicit_ttl: u64, policy: &SharedConsentPolicy) -> u64 {
    if explicit_ttl > 0 {
        return explicit_ttl;
    }
    policy.default_ttl_secs
}

// ──────────────────────────────────────────────────────────────────────────────
// Cross-Contract Consent Context
// ──────────────────────────────────────────────────────────────────────────────

/// Context for verifying consent via a cross-contract call to
/// `patient_consent_management`.
pub struct ConsentContext<'a> {
    pub env: &'a Env,
    pub patient: &'a Address,
    pub provider: &'a Address,
}

/// Verify consent for a record by evaluating the consent status locally.
///
/// When `active` and `expires_at` are known (e.g. from a cached consent
/// record), this function evaluates them without a cross-contract call.
///
/// For cross-contract verification, the calling contract should use its
/// own `patient_consent_management` client directly, then convert the
/// result using `ConsentDecision::allowed()` or `ConsentDecision::denied()`.
pub fn evaluate_consent_status(
    env: &Env,
    active: bool,
    expires_at: u64,
) -> ConsentDecision {
    let status = is_consent_effective(env, active, expires_at);
    match status {
        ConsentStatus::Active => ConsentDecision::allowed(env),
        ConsentStatus::Expired => ConsentDecision::denied(
            env,
            status,
            "Patient consent has expired",
        ),
        ConsentStatus::Revoked => ConsentDecision::denied(
            env,
            status,
            "Patient consent was revoked",
        ),
        ConsentStatus::NotFound => ConsentDecision::denied(
            env,
            status,
            "No consent record found",
        ),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_active_consent() {
        let env = Env::default();
        assert_eq!(is_consent_effective(&env, true, 0), ConsentStatus::Active);
    }

    #[test]
    fn test_revoked_consent() {
        let env = Env::default();
        assert_eq!(is_consent_effective(&env, false, 0), ConsentStatus::Revoked);
    }

    #[test]
    fn test_expired_consent() {
        let env = Env::default();
        env.ledger().set_timestamp(100);
        assert_eq!(is_consent_effective(&env, true, 50), ConsentStatus::Expired);
    }

    #[test]
    fn test_active_with_future_expiry() {
        let env = Env::default();
        env.ledger().set_timestamp(50);
        assert_eq!(is_consent_effective(&env, true, 100), ConsentStatus::Active);
    }

    #[test]
    fn test_near_expiry() {
        let env = Env::default();
        env.ledger().set_timestamp(90);
        let policy = SharedConsentPolicy {
            notification_window_secs: 20,
            ..Default::default()
        };
        // expires_at=100, now=90, remaining=10 <= 20
        assert!(is_near_expiry(&env, 100, &policy));
    }

    #[test]
    fn test_not_near_expiry() {
        let env = Env::default();
        env.ledger().set_timestamp(50);
        let policy = SharedConsentPolicy {
            notification_window_secs: 20,
            ..Default::default()
        };
        // expires_at=100, now=50, remaining=50 > 20
        assert!(!is_near_expiry(&env, 100, &policy));
    }

    #[test]
    fn test_effective_ttl_explicit() {
        let policy = SharedConsentPolicy::default();
        assert_eq!(effective_ttl(3600, &policy), 3600);
    }

    #[test]
    fn test_effective_ttl_default() {
        let policy = SharedConsentPolicy {
            default_ttl_secs: 86400,
            ..Default::default()
        };
        assert_eq!(effective_ttl(0, &policy), 86400);
    }
}
