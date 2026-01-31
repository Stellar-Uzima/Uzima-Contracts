#![no_std]
#![allow(clippy::len_zero)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, String,
};

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
    Model(BytesN<32>),
    ParticipantUpdate(u64, Address),
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

#[contract]
pub struct AIAnalyticsContract;

#[contractimpl]
impl AIAnalyticsContract {
    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(true)
    }

    pub fn start_round(
        env: Env,
        caller: Address,
        base_model_id: BytesN<32>,
        min_participants: u32,
        dp_epsilon: u32,
    ) -> Result<u64, Error> {
        caller.require_auth();
        let id = env
            .storage()
            .instance()
            .get(&DataKey::RoundCounter)
            .unwrap_or(0u64)
            + 1;
        let round = FederatedRound {
            id,
            base_model_id,
            min_participants,
            dp_epsilon,
            started_at: env.ledger().timestamp(),
            finalized_at: 0,
            total_updates: 0,
            is_finalized: false,
        };
        env.storage().instance().set(&DataKey::Round(id), &round);
        env.storage().instance().set(&DataKey::RoundCounter, &id);
        Ok(id)
    }

    pub fn submit_update(
        env: Env,
        participant: Address,
        round_id: u64,
        _hash: BytesN<32>,
        _samples: u32,
    ) -> Result<bool, Error> {
        participant.require_auth();
        let mut round: FederatedRound = env
            .storage()
            .instance()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;
        if round.is_finalized {
            return Err(Error::RoundFinalized);
        }
        round.total_updates += 1;
        env.storage()
            .instance()
            .set(&DataKey::Round(round_id), &round);
        Ok(true)
    }

    pub fn finalize_round(
        env: Env,
        caller: Address,
        round_id: u64,
        new_model_id: BytesN<32>,
        description: String,
        metrics: String,
        fairness: String,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut round: FederatedRound = env
            .storage()
            .instance()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;
        round.is_finalized = true;
        round.finalized_at = env.ledger().timestamp();
        let meta = ModelMetadata {
            model_id: new_model_id.clone(),
            round_id,
            description,
            metrics_ref: metrics,
            fairness_report_ref: fairness,
            created_at: round.finalized_at,
        };
        env.storage()
            .instance()
            .set(&DataKey::Round(round_id), &round);
        env.storage()
            .instance()
            .set(&DataKey::Model(new_model_id), &meta);
        Ok(true)
    }

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        env.storage().instance().get(&DataKey::Round(round_id))
    }
    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        env.storage().instance().get(&DataKey::Model(model_id))
    }
}
