//! Storage helpers for the Medical Records contract.
//!
//! Centralizes storage access patterns for records, users, and metadata
//! with consistent TTL management.
//!
//! Primary types and DataKey are defined in `lib.rs`.

use soroban_sdk::Env;

/// TTL threshold and extension target for persistent data.
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND_TO: u32 = 10_000;

/// Extend TTL on a persistent storage key.
/// Call after writing any long-lived persistent data.
pub fn extend_persistent_ttl(env: &Env, key: &crate::DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}
