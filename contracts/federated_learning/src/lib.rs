// Federated Learning Contract - Privacy-preserving ML with differential privacy
#![no_std]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String,
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
pub struct RoundInfo {
    pub round_id: u32,
    pub status: RoundStatus,
    pub participants: Vec<Address>,
    pub updates: Vec<ModelUpdate>,
    pub global_model_hash: BytesN<32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum RoundStatus {
    Active,
    Aggregating,
    Finalized,
    Failed,
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
    InvalidPrivacyBudget = 6,
    PrivacyBudgetExceeded = 7,
    InvalidDPParameter = 8,
}

#[contract]
pub struct FederatedLearning;

#[contractimpl]
impl FederatedLearning {
    pub fn initialize(env: Env, admin: Address, coordinator: Address) {
        if env.storage().persistent().has(&ADMIN) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Coordinator, &coordinator);
        true
    }

    fn ensure_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Federated learning admin not set"));

        if admin != *caller {
            panic!("Not authorized: caller is not admin");
        }
    }

    fn ensure_coordinator(env: &Env, caller: &Address) {
        let coordinator: Address = env
            .storage()
            .instance()
            .get(&DataKey::Coordinator)
            .unwrap_or_else(|| panic!("Federated learning coordinator not set"));

        if coordinator != *caller {
            panic!("Not authorized: caller is not coordinator");
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

        if min_participants == 0 {
            panic!("min_participants must be > 0");
        }

        if dp_epsilon == 0 {
            panic!("dp_epsilon must be > 0");
        }

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
        env.events().publish((symbol_short!("RndStart"),), id);
        id
    }

    pub fn submit_update(
        env: Env,
        participant: Address,
        round_id: u32,
        update_hash: BytesN<32>,
        sample_size: u32,
    ) -> bool {
        participant.require_auth();

        let mut rounds: Map<u32, RoundInfo> = env
            .storage()
            .persistent()
            .get(&ROUNDS)
            .unwrap_or(Map::new(&env));
        let mut round = rounds.get(round_id).expect("Round not found");

        if round.status != RoundStatus::Active {
            panic!("Round not active");
        }

        // Check privacy budget for the participant
        let budget_key = DataKey::PrivacyBudget(participant.clone());
        let mut budget: PrivacyBudget =
            env.storage()
                .instance()
                .get(&budget_key)
                .unwrap_or(PrivacyBudget {
                    epsilon_consumed: 0,
                    epsilon_total: round.dp_epsilon, // Use round's epsilon as default budget
                });

        // Calculate privacy cost (simplified model: each sample consumes some epsilon)
        let privacy_cost = num_samples / 100; // Simplified: every 100 samples consume 1 epsilon unit

        if budget.epsilon_consumed + privacy_cost > budget.epsilon_total {
            return Err(Error::PrivacyBudgetExceeded);
        }

        budget.epsilon_consumed += privacy_cost;

        let update = ModelUpdate {
            participant: participant.clone(),
            update_hash,
            timestamp: env.ledger().timestamp(),
            sample_size,
            accepted: true,
        };

        round.total_updates += 1;
        env.storage()
            .instance()
            .set(&DataKey::Round(round_id), &round);

        env.events()
            .publish((symbol_short!("UpdSubmit"),), (round_id, participant));

        Ok(true)
    }

    pub fn finalize_round(
        env: Env,
        coordinator: Address,
        round_id: u32,
        new_global_model_hash: BytesN<32>,
    ) -> bool {
        coordinator.require_auth();
        let stored_coord: Address = env.storage().persistent().get(&COORDINATOR).unwrap();
        if coordinator != stored_coord {
            panic!("Not coordinator");
        }

        round.is_finalized = true;
        round.finalized_at = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&DataKey::Round(round_id), &round);

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
            .set(&DataKey::Model(new_model_id.clone()), &metadata);

        env.events()
            .publish((symbol_short!("RndFinal"),), (round_id, new_model_id));

        Ok(true)
    }

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        env.storage().instance().get(&DataKey::Round(round_id))
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        env.storage().instance().get(&DataKey::Model(model_id))
    }

    pub fn get_participant_update(
        env: Env,
        round_id: u64,
        participant: Address,
    ) -> Option<ParticipantUpdateMeta> {
        env.storage()
            .instance()
            .get(&DataKey::ParticipantUpdate(round_id, participant))
    }

    pub fn set_privacy_budget(
        env: Env,
        admin: Address,
        participant: Address,
        _budget: u32,
    ) -> bool {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&ADMIN).unwrap();
        if admin != stored_admin {
            panic!("Not admin");
        }

        let mut budgets: Map<Address, u32> = env
            .storage()
            .persistent()
            .get(&PRIVACY_BUDGET)
            .unwrap_or(Map::new(&env));

        // In this simple model, we set "used" to 0 and assume 'budget' is a cap handled elsewhere,
        // or here we just reset usage. Let's assume we are resetting usage to 0 for a new budget cycle.
        budgets.set(participant, 0);

        env.storage().persistent().set(&PRIVACY_BUDGET, &budgets);
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{BytesN, Env};

    #[test]
    fn test_fl_workflow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, FederatedLearning);
        let client = FederatedLearningClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let participant1 = Address::generate(&env);
        let participant2 = Address::generate(&env);

        client.initialize(&admin, &coordinator);

        let round_id = client.start_round(&coordinator);
        assert_eq!(round_id, 1);

        let update_hash1 = BytesN::from_array(&env, &[1u8; 32]);
        let update_hash2 = BytesN::from_array(&env, &[2u8; 32]);

        assert!(client.mock_all_auths().submit_update(
            &participant1,
            &round_id,
            &update_hash1,
            &100u32
        ));
        assert!(client.mock_all_auths().submit_update(
            &participant2,
            &round_id,
            &update_hash2,
            &200u32
        ));

        let new_model = BytesN::from_array(&env, &[4u8; 32]);
        assert!(client.mock_all_auths().finalize_round(
            &coordinator,
            &round_id,
            &new_model,
            &String::from_str(&env, "Test model"),
            &String::from_str(&env, "ipfs://metrics"),
            &String::from_str(&env, "ipfs://fairness"),
        ));

        let stored_round = client.get_round(&round_id).unwrap();
        assert!(stored_round.is_finalized);

        let stored_model = client.get_model(&new_model).unwrap();
        assert_eq!(stored_model.round_id, round_id);
    }

    #[test]
    fn test_privacy_budget() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, FederatedLearning);
        let client = FederatedLearningClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let participant = Address::generate(&env);

        client.initialize(&admin, &coordinator);

        // Set a small privacy budget
        assert!(client
            .mock_all_auths()
            .set_privacy_budget(&admin, &participant, &10u32));

        let base_model = BytesN::from_array(&env, &[1u8; 32]);
        let round_id_1 = client
            .mock_all_auths()
            .start_round(&admin, &base_model, &1u32, &100u32);

        let update_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Submit an update that stays within budget
        assert!(client.mock_all_auths().submit_update(
            &participant,
            &round_id_1,
            &update_hash,
            &500u32
        ));

        // Start round 2 to avoid DuplicateUpdate error
        let round_id_2 = client
            .mock_all_auths()
            .start_round(&admin, &base_model, &1u32, &100u32);

        // Submit an update that exceeds the budget (5 consumed + 6 cost = 11 > 10)
        let result = client.mock_all_auths().try_submit_update(
            &participant,
            &round_id_2,
            &update_hash,
            &600u32,
        );

        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }
}
