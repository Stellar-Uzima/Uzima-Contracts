use crate::types::{AuditRecord, DataKey};
use soroban_sdk::{Address, BytesN, Env};

pub struct ImmutableStorage;

impl ImmutableStorage {
    /// Commits an audit record to persistent storage, making it immutable.
    pub fn commit_record(env: &Env, id: u64, record: AuditRecord) {
        if env.storage().persistent().has(&DataKey::Record(id)) {
            panic!("Record already exists and cannot be modified.");
        }
        env.storage()
            .persistent()
            .set(&DataKey::Record(id), &record);
    }

    /// Retrieves an immutable record by its unique ID.
    pub fn fetch_record(env: &Env, id: u64) -> Option<AuditRecord> {
        env.storage().persistent().get(&DataKey::Record(id))
    }

    /// Verifies the presence and immutability of a specific audit entry.
    pub fn verify_existence(env: &Env, id: u64) -> bool {
        env.storage().persistent().has(&DataKey::Record(id))
    }
}
