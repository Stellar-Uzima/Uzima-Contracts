//! Storage helpers for the Patient Consent Management contract.
//!
//! Centralizes all storage read/write operations for consistent TTL management
//! and access patterns.

use crate::types::{ConsentLog, ConsentRecord, DataKey};
use soroban_sdk::{Address, Env, Vec};

/// TTL threshold: extend persistent data if remaining TTL falls below this.
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
/// Extend persistent data to this many ledgers.
const PERSISTENT_TTL_EXTEND_TO: u32 = 10_000;

/// Store a consent record at the given key with TTL extension.
pub fn set_consent_record(env: &Env, key: &DataKey, record: &ConsentRecord) {
    env.storage().persistent().set(key, record);
    env.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

/// Retrieve a consent record from storage with TTL extension.
pub fn get_consent_record(env: &Env, key: &DataKey) -> Option<ConsentRecord> {
    let val: Option<ConsentRecord> = env.storage().persistent().get(key);
    if val.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    }
    val
}

/// Store a consent log for a patient with TTL extension.
pub fn set_consent_log(env: &Env, patient: &Address, log: &ConsentLog) {
    let key = DataKey::ConsentStorage(patient.clone());
    env.storage().persistent().set(&key, log);
    env.storage()
        .persistent()
        .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}

/// Retrieve a consent log for a patient with TTL extension.
pub fn get_consent_log(env: &Env, patient: &Address) -> Option<ConsentLog> {
    let key = DataKey::ConsentStorage(patient.clone());
    let val: Option<ConsentLog> = env.storage().persistent().get(&key);
    if val.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    }
    val
}
