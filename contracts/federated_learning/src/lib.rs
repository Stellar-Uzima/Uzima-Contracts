// Federated Learning Contract - Privacy-preserving ML with differential privacy
#![no_std]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol,
};

#[derive(Clone)]
#[contracttype]
pub struct ModelUpdate {
    pub participant: Address,
    pub update_hash: BytesN<32>,
    pub timestamp: u64,
    pub sample_size: u32,
    pub accepted: bool,
}

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
pub struct PrivacyBudget {
    pub epsilon_consumed: u32,
    pub epsilon_total: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Coordinator,
    RoundCounter,
    Round(u64),
    Model(BytesN<32>),
    PrivacyBudget(Address),
    ParticipantUpdate(u64, Address),
}

const USERS: Symbol = symbol_short!("USERS");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    RoundNotFound = 2,
    RoundFinalized = 3,
    NotEnoughParticipants = 4,
    DuplicateUpdate = 5,
    InvalidPrivacyBudget = 6,
    PrivacyBudgetExceeded = 7,
    InvalidDPParameter = 8,
}

#[contract]
pub struct FederatedLearning;

#[contractimpl]
impl FederatedLearning {
    pub fn initialize(env: Env, admin: Address, coordinator: Address) -> bool {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Coordinator, &coordinator);
        env.storage().instance().set(&DataKey::RoundCounter, &0u64);
        true
    }

    fn ensure_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        if admin != *caller {
            panic!("Not authorized");
        }
    }

    fn next_round_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RoundCounter)
            .unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&DataKey::RoundCounter, &next);
        next
    }

    pub fn start_round(
        env: Env,
        caller: Address,
        base_model_id: BytesN<32>,
        min_participants: u32,
        dp_epsilon: u32,
    ) -> u64 {
        caller.require_auth();
        Self::ensure_admin(&env, &caller);

        let id = Self::next_round_id(&env);
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
        id
    }

    pub fn submit_update(
        env: Env,
        participant: Address,
        round_id: u64,
        update_hash: BytesN<32>,
        sample_size: u32,
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

        let budget_key = DataKey::PrivacyBudget(participant.clone());
        let mut budget: PrivacyBudget =
            env.storage()
                .instance()
                .get(&budget_key)
                .unwrap_or(PrivacyBudget {
                    epsilon_consumed: 0,
                    epsilon_total: 1000,
                });

        let privacy_cost = sample_size / 100;
        if budget.epsilon_consumed + privacy_cost > budget.epsilon_total {
            return Err(Error::PrivacyBudgetExceeded);
        }

        budget.epsilon_consumed += privacy_cost;
        round.total_updates += 1;

        env.storage().instance().set(&budget_key, &budget);
        env.storage()
            .instance()
            .set(&DataKey::Round(round_id), &round);

        let update_meta = ModelUpdate {
            participant: participant.clone(),
            update_hash,
            timestamp: env.ledger().timestamp(),
            sample_size,
            accepted: true,
        };
        env.storage().instance().set(
            &DataKey::ParticipantUpdate(round_id, participant),
            &update_meta,
        );

        Ok(true)
    }

    pub fn finalize_round(
        env: Env,
        coordinator: Address,
        round_id: u64,
        new_model_id: BytesN<32>,
        description: String,
        metrics_ref: String,
        fairness_report_ref: String,
    ) -> Result<bool, Error> {
        coordinator.require_auth();

        let mut round: FederatedRound = env
            .storage()
            .instance()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;
        round.is_finalized = true;
        round.finalized_at = env.ledger().timestamp();

        let metadata = ModelMetadata {
            model_id: new_model_id.clone(),
            round_id,
            description,
            metrics_ref,
            fairness_report_ref,
            created_at: round.finalized_at,
        };

        env.storage()
            .instance()
            .set(&DataKey::Round(round_id), &round);
        env.storage()
            .instance()
            .set(&DataKey::Model(new_model_id), &metadata);
        Ok(true)
    }

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        env.storage().instance().get(&DataKey::Round(round_id))
    }
    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        env.storage().instance().get(&DataKey::Model(model_id))
    }

    pub fn set_privacy_budget(
        env: Env,
        admin: Address,
        participant: Address,
        budget_val: u32,
    ) -> bool {
        admin.require_auth();
        Self::ensure_admin(&env, &admin);
        let budget = PrivacyBudget {
            epsilon_consumed: 0,
            epsilon_total: budget_val,
        };
        env.storage()
            .instance()
            .set(&DataKey::PrivacyBudget(participant), &budget);
        true
    }
}
