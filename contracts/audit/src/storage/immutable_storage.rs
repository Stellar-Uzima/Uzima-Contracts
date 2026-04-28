use crate::types::{AuditLog, DataKey};
use soroban_sdk::{Env, Vec};

pub struct ImmutableStorage;

impl ImmutableStorage {
    /// Commits a new AuditLog entry. Once written it cannot be overwritten.
    pub fn commit_log(env: &Env, id: u64, log: &AuditLog) {
        if env.storage().persistent().has(&DataKey::Log(id)) {
            panic!("AuditLog already exists and is immutable.");
        }
        env.storage().persistent().set(&DataKey::Log(id), log);
    }

    /// Retrieves an AuditLog by its unique ID.
    pub fn fetch_log(env: &Env, id: u64) -> Option<AuditLog> {
        env.storage().persistent().get(&DataKey::Log(id))
    }

    /// Returns true if the log entry exists (immutability proof).
    pub fn log_exists(env: &Env, id: u64) -> bool {
        env.storage().persistent().has(&DataKey::Log(id))
    }

    /// Fetches a page of AuditLog entries [offset, offset+limit).
    pub fn fetch_logs_page(env: &Env, offset: u64, limit: u64) -> Vec<AuditLog> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LogCount)
            .unwrap_or(0u64);
        let mut results = Vec::new(env);
        let end = (offset + limit).min(count);
        for i in offset..end {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditLog>(&DataKey::Log(i))
            {
                results.push_back(log);
            }
        }
        results
    }
}
