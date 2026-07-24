//! Timestamp Normalization for Patient Consent Management
//!
//! Integrates canonical timestamp handling into consent flows to ensure
//! consistent time representation across consent grants, revocations,
//! and expiration checks.

use soroban_sdk::{contracttype, Env, String};

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
    let offset_secs = match timezone_id.as_str() {
        "UTC" | "GMT" => 0,
        "America/New_York" | "EST" | "EDT" => 18000,   // UTC-5
        "America/Chicago" | "CST" | "CDT" => 21600,    // UTC-6
        "America/Los_Angeles" | "PST" | "PDT" => 28800, // UTC-8
        "Europe/London" | "GMT" | "BST" => 0,
        "Europe/Berlin" | "CET" | "CEST" => 3600,       // UTC+1
        "Asia/Tokyo" | "JST" => 32400,                   // UTC+9
        "Asia/Shanghai" | "CST" => 28800,                // UTC+8
        "Asia/Kolkata" | "IST" => 19800,                 // UTC+5:30
        "Australia/Sydney" | "AEST" | "AEDT" => 36000,  // UTC+10
        _ => 0, // Default to UTC for unrecognized
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
