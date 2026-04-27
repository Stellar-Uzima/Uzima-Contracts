#![no_std]

mod admin;
mod rounds;
mod types;
mod utils;
mod serialization_utils;
mod serialization_edge_cases;

#[cfg(all(test, feature = "testutils"))]
mod test;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String};

pub use types::{DataKey, Error, FederatedRound, ModelMetadata, ParticipantUpdateMeta};

#[contract]
pub struct AiAnalyticsContract;

#[contractimpl]
impl AiAnalyticsContract {
    pub fn initialize(env: Env, admin: Address) -> Result<bool, Error> {
        admin::initialize(env, admin)
    }

    pub fn start_round(
        env: Env,
        caller: Address,
        base_model_id: BytesN<32>,
        min_participants: u32,
        dp_epsilon: u32,
    ) -> Result<u64, Error> {
        rounds::start_round(env, caller, base_model_id, min_participants, dp_epsilon)
    }

    pub fn submit_update(
        env: Env,
        participant: Address,
        round_id: u64,
        update_hash: BytesN<32>,
        num_samples: u32,
    ) -> Result<bool, Error> {
        rounds::submit_update(env, participant, round_id, update_hash, num_samples)
    }

    pub fn finalize_round(
        env: Env,
        caller: Address,
        round_id: u64,
        new_model_id: BytesN<32>,
        description: String,
        metrics_ref: String,
        fairness_report_ref: String,
    ) -> Result<bool, Error> {
        rounds::finalize_round(
            env,
            caller,
            round_id,
            new_model_id,
            description,
            metrics_ref,
            fairness_report_ref,
        )
    }

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        rounds::get_round(env, round_id)
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        rounds::get_model(env, model_id)
    }
}
