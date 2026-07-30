use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationStatus {
    Pending,
    Processed,
    Failed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconciliationJob {
    pub id: u64,
    pub source_contract: Address,
    pub payload_hash: BytesN<32>,
    pub status: ReconciliationStatus,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    JobCount,
    Job(u64),
    Admin,
}