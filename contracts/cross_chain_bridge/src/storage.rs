//! Storage helpers for the Cross-Chain Bridge contract.
//!
//! Provides TTL-aware persistent storage read/write operations for
//! bridge messages, validators, oracle nodes, and proofs.
//!
//! Types are defined in `lib.rs` (CrossChainMessage, Validator, DataKey, etc.).

use soroban_sdk::{Address, Env};

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

/// Extend TTL on a temporary storage key to the session TTL.
pub fn extend_temporary_ttl(env: &Env, key: &crate::DataKey, ttl: u32) {
    env.storage().temporary().extend_ttl(key, 0, ttl);
}
