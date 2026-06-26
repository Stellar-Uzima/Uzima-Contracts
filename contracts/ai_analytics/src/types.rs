use crate::serialization_utils::{SafeSerialize, SerializationError, SerializationUtils};
use soroban_sdk::{contracterror, contracttype, Address, BytesN, Env, String, Symbol};

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

impl SafeSerialize for FederatedRound {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        // Validate individual fields
        SerializationUtils::validate_bytes_n(env, &self.base_model_id)?;

        // Validate edge cases
        if self.min_participants == 0 {
            env.events().publish(
                (Symbol::new(env, "SER_WARN"), Symbol::new(env, "ZERO_MIN")),
                (),
            );
        }

        if self.total_updates == 0 && !self.is_finalized {
            env.events().publish(
                (Symbol::new(env, "SER_WARN"), Symbol::new(env, "NO_UPDATES")),
                (),
            );
        }

        Ok(())
    }
}

#[derive(Clone)]
#[contracttype]
pub struct ParticipantUpdateMeta {
    pub round_id: u64,
    pub participant: Address,
    pub update_hash: BytesN<32>,
    pub num_samples: u32,
}

impl SafeSerialize for ParticipantUpdateMeta {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        // Validate individual fields
        SerializationUtils::validate_address(env, &self.participant)?;
        SerializationUtils::validate_bytes_n(env, &self.update_hash)?;

        // Validate edge cases
        if self.num_samples == 0 {
            env.events().publish(
                (Symbol::new(env, "SER_WARN"), Symbol::new(env, "ZERO_SAMP")),
                (),
            );
        }

        Ok(())
    }
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

impl SafeSerialize for ModelMetadata {
    fn safe_serialize(&self, env: &Env) -> Result<(), SerializationError> {
        // Validate individual fields
        SerializationUtils::validate_bytes_n(env, &self.model_id)?;
        SerializationUtils::safe_serialize_string(env, &self.description)?;
        SerializationUtils::safe_serialize_string(env, &self.metrics_ref)?;
        SerializationUtils::safe_serialize_string(env, &self.fairness_report_ref)?;

        // Validate edge cases
        if self.description.is_empty() {
            env.events().publish(
                (Symbol::new(env, "SER_WARN"), Symbol::new(env, "EMPTY_DESC")),
                (),
            );
        }

        if self.metrics_ref.is_empty() && self.fairness_report_ref.is_empty() {
            env.events().publish(
                (Symbol::new(env, "SER_WARN"), Symbol::new(env, "NO_REFS")),
                (),
            );
        }

        if self.created_at == 0 {
            env.events().publish(
                (Symbol::new(env, "SER_WARN"), Symbol::new(env, "ZERO_TS")),
                (),
            );
        }

        Ok(())
    }
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
    SerializationError = 8,
    CollectionTooLarge = 9,
    StringTooLong = 10,
    NestingTooDeep = 11,
}
