#![no_std]
//! batch_audit - Batched audit trail ingestion for high-volume compliance workflows.
//!
//! Accumulates audit entries in a batch buffer and flushes them in a single
//! contract call, reducing per-entry overhead for high-throughput scenarios.

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec as SVec};

/// Maximum entries per batch flush.
pub const MAX_BATCH_SIZE: u32 = 50;

/// A single audit entry in the batch.
#[derive(Clone)]
#[contracttype]
pub struct AuditEntry {
    pub actor: Address,
    pub action: Symbol,
    pub resource: Symbol,
    pub outcome: bool,
    pub ledger: u32,
}

/// Batch of audit entries pending flush.
#[derive(Clone)]
#[contracttype]
pub struct AuditBatch {
    pub entries: SVec<AuditEntry>,
    pub batch_id: u64,
}

#[derive(Clone)]
#[contracttype]
enum BatchKey {
    Pending,
    BatchCounter,
}

/// Batched audit ingestion manager.
pub struct BatchAudit;

impl BatchAudit {
    /// Append an entry to the pending batch.
    /// Auto-flushes when batch reaches MAX_BATCH_SIZE.
    pub fn append(
        env: &Env,
        actor: Address,
        action: Symbol,
        resource: Symbol,
        outcome: bool,
    ) -> Result<(), BatchAuditError> {
        let mut batch: AuditBatch = env
            .storage()
            .temporary()
            .get(&BatchKey::Pending)
            .unwrap_or_else(|| AuditBatch {
                entries: SVec::new(env),
                batch_id: Self::next_batch_id(env),
            });

        if batch.entries.len() >= MAX_BATCH_SIZE {
            return Err(BatchAuditError::BatchFull);
        }

        batch.entries.push_back(AuditEntry {
            actor,
            action,
            resource,
            outcome,
            ledger: env.ledger().sequence(),
        });

        env.storage().temporary().set(&BatchKey::Pending, &batch);
        env.storage().temporary().extend_ttl(&BatchKey::Pending, 0, 1440); // ~2 hours

        Ok(())
    }

    /// Flush the pending batch, emitting one event per entry.
    /// Returns the number of entries flushed.
    pub fn flush(env: &Env) -> u32 {
        let batch: AuditBatch = match env.storage().temporary().get(&BatchKey::Pending) {
            Some(b) => b,
            None => return 0,
        };

        let count = batch.entries.len();
        for entry in batch.entries.iter() {
            env.events().publish(
                (symbol_short!("audit"), symbol_short!("entry")),
                (&entry.actor, &entry.action, &entry.resource, entry.outcome),
            );
        }

        // Clear the batch
        env.storage().temporary().remove(&BatchKey::Pending);

        env.events().publish(
            (symbol_short!("audit"), symbol_short!("flushed")),
            (batch.batch_id, count),
        );

        count
    }

    /// Returns how many entries are pending.
    pub fn pending_count(env: &Env) -> u32 {
        env.storage()
            .temporary()
            .get::<BatchKey, AuditBatch>(&BatchKey::Pending)
            .map(|b| b.entries.len())
            .unwrap_or(0)
    }

    fn next_batch_id(env: &Env) -> u64 {
        let id: u64 = env.storage().instance().get(&BatchKey::BatchCounter).unwrap_or(0);
        env.storage().instance().set(&BatchKey::BatchCounter, &(id + 1));
        id + 1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[soroban_sdk::contracterror]
#[repr(u32)]
pub enum BatchAuditError {
    BatchFull = 900,
}
