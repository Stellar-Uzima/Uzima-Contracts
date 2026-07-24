//! Canonical Timestamps and Timezone Handling
//!
//! Provides normalized timestamp types and utility functions for consistent
//! time handling across medical record workflows. All timestamps use
//! Unix epoch seconds in UTC for on-chain storage, with timezone-aware
//! conversion helpers for display and cross-system interoperability.

use soroban_sdk::{contracttype, symbol_short, Env, String};

/// Supported IANA timezone identifiers for medical record workflows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum TimezoneRegion {
    /// UTC (Coordinated Universal Time)
    UTC,
    /// US Eastern Time (America/New_York)
    USEastern,
    /// US Central Time (America/Chicago)
    USCentral,
    /// US Pacific Time (America/Los_Angeles)
    USPacific,
    /// Europe/London (GMT/BST)
    EuropeLondon,
    /// Europe/Berlin (CET/CEST)
    EuropeBerlin,
    /// Asia/Tokyo (JST)
    AsiaTokyo,
    /// Asia/Shanghai (CST)
    AsiaShanghai,
    /// Asia/Kolkata (IST)
    AsiaKolkata,
    /// Australia/Sydney (AEST/AEDT)
    AustraliaSydney,
    /// Custom timezone with UTC offset in minutes.
    Custom(i32),
}

impl TimezoneRegion {
    /// Returns the UTC offset in minutes for this timezone region.
    pub fn utc_offset_minutes(&self) -> i32 {
        match self {
            TimezoneRegion::UTC => 0,
            TimezoneRegion::USEastern => -300,   // -5:00
            TimezoneRegion::USCentral => -360,   // -6:00
            TimezoneRegion::USPacific => -480,   // -8:00
            TimezoneRegion::EuropeLondon => 0,   // +0:00 (simplified, no DST)
            TimezoneRegion::EuropeBerlin => 60,  // +1:00
            TimezoneRegion::AsiaTokyo => 540,    // +9:00
            TimezoneRegion::AsiaShanghai => 480, // +8:00
            TimezoneRegion::AsiaKolkata => 330,  // +5:30
            TimezoneRegion::AustraliaSydney => 600, // +10:00
            TimezoneRegion::Custom(offset) => *offset,
        }
    }

    /// Returns the IANA timezone identifier string.
    pub fn identifier(&self) -> &str {
        match self {
            TimezoneRegion::UTC => "UTC",
            TimezoneRegion::USEastern => "America/New_York",
            TimezoneRegion::USCentral => "America/Chicago",
            TimezoneRegion::USPacific => "America/Los_Angeles",
            TimezoneRegion::EuropeLondon => "Europe/London",
            TimezoneRegion::EuropeBerlin => "Europe/Berlin",
            TimezoneRegion::AsiaTokyo => "Asia/Tokyo",
            TimezoneRegion::AsiaShanghai => "Asia/Shanghai",
            TimezoneRegion::AsiaKolkata => "Asia/Kolkata",
            TimezoneRegion::AustraliaSydney => "Australia/Sydney",
            TimezoneRegion::Custom(_) => "Custom",
        }
    }
}

/// Configuration for timezone-aware timestamp handling.
#[derive(Clone)]
#[contracttype]
pub struct TimezoneConfig {
    /// Default timezone for new records.
    pub default_timezone: TimezoneRegion,
    /// Whether to store timestamps in UTC (recommended) or local time.
    pub store_as_utc: bool,
    /// Whether to include timezone metadata in record timestamps.
    pub include_timezone_metadata: bool,
}

impl Default for TimezoneConfig {
    fn default() -> Self {
        Self {
            default_timezone: TimezoneRegion::UTC,
            store_as_utc: true,
            include_timezone_metadata: true,
        }
    }
}

/// A canonical timestamp with timezone awareness.
#[derive(Clone)]
#[contracttype]
pub struct CanonicalTimestamp {
    /// Unix epoch timestamp in seconds (always UTC).
    pub utc_seconds: u64,
    /// Timezone the timestamp was originally recorded in.
    pub source_timezone: TimezoneRegion,
    /// Human-readable ISO 8601 string (e.g., "2026-07-24T13:30:00Z").
    pub iso8601: String,
    /// Whether this timestamp has been normalized to UTC.
    pub is_normalized: bool,
}

/// Normalization result for batch timestamp operations.
#[derive(Clone)]
#[contracttype]
pub struct TimestampNormalization {
    /// Number of timestamps successfully normalized.
    pub normalized_count: u32,
    /// Number of timestamps that failed normalization.
    pub failed_count: u32,
    /// First error encountered (if any).
    pub first_error: Option<String>,
    /// Processing timestamp (when normalization was performed).
    pub processed_at: u64,
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Normalize a local timestamp to UTC canonical form.
///
/// Takes a local-time unix timestamp and the source timezone, and returns
/// a CanonicalTimestamp in UTC.
pub fn normalize_to_utc(
    env: &Env,
    local_timestamp: u64,
    source_tz: TimezoneRegion,
) -> CanonicalTimestamp {
    let offset_minutes = source_tz.utc_offset_minutes();
    let utc_seconds = if offset_minutes >= 0 {
        local_timestamp.saturating_sub((offset_minutes as u64) * 60)
    } else {
        local_timestamp.saturating_add((offset_minutes.unsigned_abs() as u64) * 60)
    };

    let iso8601 = format_utc_iso8601(env, utc_seconds);

    CanonicalTimestamp {
        utc_seconds,
        source_timezone: source_tz,
        iso8601,
        is_normalized: true,
    }
}

/// Convert a UTC timestamp to a local timezone.
///
/// Takes a UTC unix timestamp and a target timezone, and returns
/// the local-time equivalent.
pub fn utc_to_local(utc_seconds: u64, target_tz: TimezoneRegion) -> u64 {
    let offset_minutes = target_tz.utc_offset_minutes();
    if offset_minutes >= 0 {
        utc_seconds.saturating_add((offset_minutes as u64) * 60)
    } else {
        utc_seconds.saturating_sub((offset_minutes.unsigned_abs() as u64) * 60)
    }
}

/// Format a UTC timestamp as ISO 8601 string.
///
/// Simplified formatting for on-chain use. Produces "YYYY-MM-DDTHH:MM:SSZ".
pub fn format_utc_iso8601(env: &Env, utc_seconds: u64) -> String {
    // Simplified ISO 8601 formatting
    // In production, use a proper datetime library
    let days = utc_seconds / 86400;
    let remaining = utc_seconds % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Calculate year/month/day from days since epoch (simplified)
    let year = 1970 + (days / 365) as u32;
    let day_of_year = days % 365;
    let month = ((day_of_year / 30) + 1).min(12) as u32;
    let day = (day_of_year % 30) + 1;

    let s = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    );
    String::from_str(env, &s)
}

/// Check if two timestamps are within a specified tolerance (in seconds).
///
/// Used for comparing timestamps across systems with different clock skews.
pub fn timestamps_match(t1: u64, t2: u64, tolerance_secs: u64) -> bool {
    let diff = if t1 > t2 { t1 - t2 } else { t2 - t1 };
    diff <= tolerance_secs
}

/// Get the current ledger timestamp as a CanonicalTimestamp in UTC.
pub fn current_canonical_timestamp(env: &Env) -> CanonicalTimestamp {
    let utc_seconds = env.ledger().timestamp();
    let iso8601 = format_utc_iso8601(env, utc_seconds);

    CanonicalTimestamp {
        utc_seconds,
        source_timezone: TimezoneRegion::UTC,
        iso8601,
        is_normalized: true,
    }
}

/// Normalize a batch of timestamps.
pub fn normalize_timestamp_batch(
    env: &Env,
    timestamps: &[(u64, TimezoneRegion)],
) -> TimestampNormalization {
    let mut normalized_count = 0u32;
    let mut failed_count = 0u32;

    for (ts, tz) in timestamps.iter() {
        // Validate timestamp is reasonable (after 2000-01-01, before 2100-01-01)
        if *ts >= 946684800 && *ts <= 4102444800 {
            let _normalized = normalize_to_utc(env, *ts, *tz);
            normalized_count += 1;
        } else {
            failed_count += 1;
        }
    }

    TimestampNormalization {
        normalized_count,
        failed_count,
        first_error: if failed_count > 0 {
            Some(String::from_str(env, "some timestamps out of valid range"))
        } else {
            None
        },
        processed_at: env.ledger().timestamp(),
    }
}

/// Validate that a timestamp is within acceptable bounds for medical records.
///
/// Medical records should not have timestamps before 2000-01-01 or after 2100-01-01.
pub fn validate_medical_timestamp(timestamp: u64) -> bool {
    timestamp >= 946684800 && timestamp <= 4102444800
}

/// Calculate the difference between two timestamps in seconds.
pub fn timestamp_diff_secs(t1: u64, t2: u64) -> u64 {
    if t1 > t2 {
        t1 - t2
    } else {
        t2 - t1
    }
}

/// Check if a timestamp is older than a specified number of seconds from now.
pub fn is_older_than(env: &Env, timestamp: u64, age_secs: u64) -> bool {
    let now = env.ledger().timestamp();
    now > timestamp && (now - timestamp) > age_secs
}
