//! Timestamp Normalization for Patient Consent Management
//!
//! Integrates canonical timestamp handling into consent flows to ensure
//! consistent time representation across consent grants, revocations,
//! and expiration checks.

use soroban_sdk::{contracttype, Env, String};

/// Maximum timezone identifier length handled by the lookup below.
const MAX_TZ_LEN: usize = 32;

/// Compare a soroban [`String`] against a `&str` literal without `alloc`.
fn str_eq(s: &String, expected: &str) -> bool {
    let len = s.len() as usize;
    if len != expected.len() || len > MAX_TZ_LEN {
        return false;
    }
    let mut buf = [0u8; MAX_TZ_LEN];
    s.copy_into_slice(&mut buf[..len]);
    &buf[..len] == expected.as_bytes()
}

/// Timezone-aware consent timestamp with normalization support.
#[derive(Clone)]
#[contracttype]
pub struct ConsentTimestamp {
    /// UTC epoch seconds (normalized).
    pub utc_seconds: u64,
    /// Original timezone identifier before normalization.
    pub source_tz: String,
    /// Whether this timestamp has been normalized to UTC.
    pub is_normalized: bool,
}

/// Normalize a consent timestamp to UTC.
///
/// Takes a local timestamp and timezone identifier, returns a ConsentTimestamp
/// normalized to UTC. If the timezone is not recognized, assumes UTC.
pub fn normalize_consent_timestamp(
    _env: &Env,
    local_timestamp: u64,
    timezone_id: &String,
) -> ConsentTimestamp {
    // Simplified timezone offset lookup
    let offset_secs = if str_eq(timezone_id, "UTC") || str_eq(timezone_id, "GMT") {
        0
    } else if str_eq(timezone_id, "America/New_York")
        || str_eq(timezone_id, "EST")
        || str_eq(timezone_id, "EDT")
    {
        18000 // UTC-5
    } else if str_eq(timezone_id, "America/Chicago")
        || str_eq(timezone_id, "CST")
        || str_eq(timezone_id, "CDT")
    {
        21600 // UTC-6
    } else if str_eq(timezone_id, "America/Los_Angeles")
        || str_eq(timezone_id, "PST")
        || str_eq(timezone_id, "PDT")
    {
        28800 // UTC-8
    } else if str_eq(timezone_id, "Europe/London") || str_eq(timezone_id, "BST") {
        0
    } else if str_eq(timezone_id, "Europe/Berlin")
        || str_eq(timezone_id, "CET")
        || str_eq(timezone_id, "CEST")
    {
        3600 // UTC+1
    } else if str_eq(timezone_id, "Asia/Tokyo") || str_eq(timezone_id, "JST") {
        32400 // UTC+9
    } else if str_eq(timezone_id, "Asia/Shanghai") {
        28800 // UTC+8
    } else if str_eq(timezone_id, "Asia/Kolkata") || str_eq(timezone_id, "IST") {
        19800 // UTC+5:30
    } else if str_eq(timezone_id, "Australia/Sydney")
        || str_eq(timezone_id, "AEST")
        || str_eq(timezone_id, "AEDT")
    {
        36000 // UTC+10
    } else {
        0 // Default to UTC for unrecognized
    };

    let utc_seconds = if local_timestamp > offset_secs {
        local_timestamp - offset_secs
    } else {
        local_timestamp
    };

    ConsentTimestamp {
        utc_seconds,
        source_tz: timezone_id.clone(),
        is_normalized: true,
    }
}

/// Check if a consent has expired based on UTC-normalized timestamps.
pub fn is_consent_expired(env: &Env, expires_at_utc: u64) -> bool {
    let now = env.ledger().timestamp();
    now > expires_at_utc
}

/// Check if a consent is expiring soon (within the notification window).
pub fn is_consent_expiring_soon(
    env: &Env,
    expires_at_utc: u64,
    notification_window_secs: u64,
) -> bool {
    let now = env.ledger().timestamp();
    now <= expires_at_utc && (expires_at_utc - now) <= notification_window_secs
}

/// Calculate remaining seconds until consent expires.
pub fn consent_remaining_secs(env: &Env, expires_at_utc: u64) -> u64 {
    let now = env.ledger().timestamp();
    if now >= expires_at_utc {
        0
    } else {
        expires_at_utc - now
    }
}
