use crate::types::{ActionType, AuditLog, DataKey};
use soroban_sdk::{Address, Env, Vec};

pub struct AuditQuery;

impl AuditQuery {
    /// Returns all AuditLog entries for a given actor address.
    pub fn logs_by_actor(env: &Env, actor: &Address) -> Vec<AuditLog> {
        let indices: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ActorLogs(actor.clone()))
            .unwrap_or(Vec::new(env));

        let mut results = Vec::new(env);
        for id in indices.iter() {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditLog>(&DataKey::Log(id))
            {
                results.push_back(log);
            }
        }
        results
    }

    /// Returns all AuditLog entries for a given ActionType.
    pub fn logs_by_action(env: &Env, action: ActionType) -> Vec<AuditLog> {
        let key = DataKey::ActionLogs(action as u32);
        let indices: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        let mut results = Vec::new(env);
        for id in indices.iter() {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditLog>(&DataKey::Log(id))
            {
                results.push_back(log);
            }
        }
        results
    }

    /// Returns AuditLog entries within a timestamp range.
    pub fn logs_by_timeframe(env: &Env, start: u64, end: u64) -> Vec<AuditLog> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LogCount)
            .unwrap_or(0u64);
        let mut results = Vec::new(env);
        for i in 1..=count {
            if let Some(log) = env
                .storage()
                .persistent()
                .get::<DataKey, AuditLog>(&DataKey::Log(i))
            {
                if log.timestamp >= start && log.timestamp <= end {
                    results.push_back(log);
                }
            }
        }
        results
    }
}
