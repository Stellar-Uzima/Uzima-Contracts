#![cfg(test)]

use super::timestamp_normalization::*;
use soroban_sdk::Env;

#[test]
fn test_timezone_region_utc_offset() {
    assert_eq!(TimezoneRegion::UTC.utc_offset_minutes(), 0);
    assert_eq!(TimezoneRegion::USEastern.utc_offset_minutes(), -300);
    assert_eq!(TimezoneRegion::USPacific.utc_offset_minutes(), -480);
    assert_eq!(TimezoneRegion::AsiaTokyo.utc_offset_minutes(), 540);
    assert_eq!(TimezoneRegion::AsiaShanghai.utc_offset_minutes(), 480);
    assert_eq!(TimezoneRegion::AsiaKolkata.utc_offset_minutes(), 330);
}

#[test]
fn test_timezone_region_identifier() {
    assert_eq!(TimezoneRegion::UTC.identifier(), "UTC");
    assert_eq!(TimezoneRegion::USEastern.identifier(), "America/New_York");
    assert_eq!(TimezoneRegion::AsiaTokyo.identifier(), "Asia/Tokyo");
}

#[test]
fn test_custom_timezone_offset() {
    let tz = TimezoneRegion::Custom(120);
    assert_eq!(tz.utc_offset_minutes(), 120);
    assert_eq!(tz.identifier(), "Custom");
}

#[test]
fn test_normalize_to_utc_from_eastern() {
    let env = Env::default();
    // 2024-01-15 10:00:00 UTC = 2024-01-15 05:00:00 EST
    let utc_timestamp = 1705312800u64;
    let result = normalize_to_utc(&env, utc_timestamp, TimezoneRegion::USEastern);
    // Eastern is UTC-5, so local = UTC - 5h = UTC - 18000
    assert_eq!(result.utc_seconds, utc_timestamp - 18000);
    assert!(result.is_normalized);
    assert_eq!(result.source_timezone, TimezoneRegion::USEastern);
}

#[test]
fn test_normalize_to_utc_from_tokyo() {
    let env = Env::default();
    // 2024-01-15 10:00:00 UTC = 2024-01-15 19:00:00 JST
    let local_timestamp = 1705345200u64; // 19:00 JST
    let result = normalize_to_utc(&env, local_timestamp, TimezoneRegion::AsiaTokyo);
    // Tokyo is UTC+9, so UTC = local - 9h = local - 32400
    assert_eq!(result.utc_seconds, local_timestamp - 32400);
}

#[test]
fn test_utc_to_local_tokyo() {
    let utc = 1705312800u64;
    let local = utc_to_local(utc, TimezoneRegion::AsiaTokyo);
    assert_eq!(local, utc + 32400);
}

#[test]
fn test_utc_to_local_eastern() {
    let utc = 1705312800u64;
    let local = utc_to_local(utc, TimezoneRegion::USEastern);
    assert_eq!(local, utc - 18000);
}

#[test]
fn test_utc_to_local_utc() {
    let utc = 1705312800u64;
    let local = utc_to_local(utc, TimezoneRegion::UTC);
    assert_eq!(local, utc);
}

#[test]
fn test_timestamps_match_within_tolerance() {
    assert!(timestamps_match(1000, 1005, 10));
    assert!(timestamps_match(1000, 1000, 0));
    assert!(!timestamps_match(1000, 1020, 10));
}

#[test]
fn test_timestamps_match_order_independent() {
    assert!(timestamps_match(1005, 1000, 10));
    assert!(timestamps_match(1000, 1005, 10));
}

#[test]
fn test_validate_medical_timestamp_valid() {
    assert!(validate_medical_timestamp(1705312800)); // 2024
    assert!(validate_medical_timestamp(946684800));  // 2000-01-01
    assert!(validate_medical_timestamp(4102444800)); // 2100-01-01
}

#[test]
fn test_validate_medical_timestamp_invalid() {
    assert!(!validate_medical_timestamp(0));         // Before 2000
    assert!(!validate_medical_timestamp(946684799)); // Just before 2000
    assert!(!validate_medical_timestamp(4102444801)); // After 2100
}

#[test]
fn test_timestamp_diff_secs() {
    assert_eq!(timestamp_diff_secs(1000, 1050), 50);
    assert_eq!(timestamp_diff_secs(1050, 1000), 50);
    assert_eq!(timestamp_diff_secs(1000, 1000), 0);
}

#[test]
fn test_is_older_than() {
    let env = Env::default();
    // Current ledger timestamp is 0 in default env
    // A timestamp of 100 with age 50 should be "older" since 0 < 100 but 100 - 0 = 100 > 50
    // Actually is_older_than checks now > timestamp && (now - timestamp) > age
    // In default env, now = 0, so no timestamp is older
    assert!(!is_older_than(&env, 100, 50));
}

#[test]
fn test_normalize_timestamp_batch() {
    let env = Env::default();
    let timestamps = vec![
        (1705312800u64, TimezoneRegion::UTC),
        (1705312800, TimezoneRegion::USEastern),
        (1705312800, TimezoneRegion::AsiaTokyo),
    ];
    let result = normalize_timestamp_batch(&env, &timestamps);
    assert_eq!(result.normalized_count, 3);
    assert_eq!(result.failed_count, 0);
    assert!(result.first_error.is_none());
}

#[test]
fn test_normalize_timestamp_batch_with_invalid() {
    let env = Env::default();
    let timestamps = vec![
        (1705312800u64, TimezoneRegion::UTC),
        (0u64, TimezoneRegion::UTC),           // Before 2000
        (5000000000u64, TimezoneRegion::UTC),  // After 2100
    ];
    let result = normalize_timestamp_batch(&env, &timestamps);
    assert_eq!(result.normalized_count, 1);
    assert_eq!(result.failed_count, 2);
    assert!(result.first_error.is_some());
}

#[test]
fn test_current_canonical_timestamp() {
    let env = Env::default();
    let ts = current_canonical_timestamp(&env);
    assert!(ts.is_normalized);
    assert_eq!(ts.source_timezone, TimezoneRegion::UTC);
}

#[test]
fn test_timezone_config_default() {
    let config = TimezoneConfig::default();
    assert_eq!(config.default_timezone, TimezoneRegion::UTC);
    assert!(config.store_as_utc);
    assert!(config.include_timezone_metadata);
}
