use soroban_sdk::{contracterror, contracttype, Address, BytesN, String};

#[derive(Clone)]
#[contracttype]
pub struct FederatedRound {
    pub id: u64,
    pub base_model_id: BytesN<32>,
    pub min_participants: u32,
    pub dp_epsilon: u32,
    pub started_at: u64,
    pub finalized_at: u64,
    pub total_updates: u32,
    pub is_finalized: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct ParticipantUpdateMeta {
    pub round_id: u64,
    pub participant: Address,
    pub update_hash: BytesN<32>,
    pub num_samples: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ModelMetadata {
    pub model_id: BytesN<32>,
    pub round_id: u64,
    pub description: String,
    pub metrics_ref: String,
    pub fairness_report_ref: String,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    RoundCounter,
    Round(u64),
    ParticipantUpdate(u64, Address),
    Model(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    RoundNotFound = 2,
    RoundFinalized = 3,
    NotEnoughParticipants = 4,
    DuplicateUpdate = 5,
    AlreadyInitialized = 6,
    AdminNotSet = 7,
}
