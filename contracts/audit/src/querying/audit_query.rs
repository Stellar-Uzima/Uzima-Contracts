use crate::types::{AuditRecord, DataKey};
use soroban_sdk::{Address, Env, Vec};

pub struct AuditQuery;

impl AuditQuery {
    /// Retrieves a list of audit records associated with a specific user.
    pub fn list_user_records(env: &Env, user: &Address) -> Vec<AuditRecord> {
        let indices: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::UserAudits(user.clone()))
            .unwrap_or(Vec::new(env));

        let mut results = Vec::new(env);
        for id in indices.iter() {
            if let Some(record) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditRecord>(&DataKey::Record(id))
            {
                results.push_back(record);
            }
        }
        results
    }

    /// Conducts a historical search Filtered by timeframe.
    pub fn search_by_timeframe(env: &Env, start: u64, end: u64) -> Vec<AuditRecord> {
        let count = env
            .storage()
            .instance()
            .get(&DataKey::RecordCount)
            .unwrap_or(0u64);
        let mut results = Vec::new(env);
        for i in 0..count {
            if let Some(record) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditRecord>(&DataKey::Record(i))
            {
                if record.timestamp >= start && record.timestamp <= end {
                    results.push_back(record);
                }
            }
        }
        results
    }
}
