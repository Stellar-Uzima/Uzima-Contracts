#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
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
pub struct ParticipantUpdateMeta {
    pub round_id: u64,
    pub participant: Address,
    pub update_hash: BytesN<32>,
    pub num_samples: u32,
    pub submitted_at: u64,
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
    ParticipantUpdate(u64, Address),
    Model(BytesN<32>),
    PrivacyBudget(Address),
}

const ADMIN: Symbol = symbol_short!("ADMIN");
const COORDINATOR: Symbol = symbol_short!("COORD");

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
pub struct FederatedLearningContract;

#[contractimpl]
impl FederatedLearningContract {
    pub fn initialize(env: Env, admin: Address, coordinator: Address) -> bool {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Coordinator, &coordinator);
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
        env.storage()
            .instance()
            .set(&DataKey::RoundCounter, &next);
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
        env.events().publish((symbol_short!("RND_STRT"),), id);
        id
    }

    pub fn submit_update(
        env: Env,
        participant: Address,
        round_id: u64,
        update_hash: BytesN<32>,
        num_samples: u32,
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

        let key = DataKey::ParticipantUpdate(round_id, participant.clone());
        if env.storage().instance().has(&key) {
            return Err(Error::DuplicateUpdate);
        }

        // Check privacy budget for the participant
        let budget_key = DataKey::PrivacyBudget(participant.clone());
        let mut budget: PrivacyBudget = env
            .storage()
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

        let update = ParticipantUpdateMeta {
            round_id,
            participant: participant.clone(),
            update_hash,
            num_samples,
            submitted_at: env.ledger().timestamp(),
        };

        env.storage().instance().set(&key, &update);
        env.storage().instance().set(&budget_key, &budget);

        round.total_updates += 1;
        env.storage().instance().set(&DataKey::Round(round_id), &round);

        env.events()
            .publish((symbol_short!("UPD_SUB"),), (round_id, participant));

        Ok(true)
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
        caller.require_auth();
        Self::ensure_coordinator(&env, &caller);

        let mut round: FederatedRound = env
            .storage()
            .instance()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;

        if round.is_finalized {
            return Err(Error::RoundFinalized);
        }

        if round.total_updates < round.min_participants {
            return Err(Error::NotEnoughParticipants);
        }

        round.is_finalized = true;
        round.finalized_at = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Round(round_id), &round);

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
            .publish((symbol_short!("RND_FIN"),), (round_id, new_model_id));

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
        caller: Address,
        participant: Address,
        epsilon_total: u32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        Self::ensure_admin(&env, &caller);

        if epsilon_total == 0 {
            return Err(Error::InvalidPrivacyBudget);
        }

        let budget = PrivacyBudget {
            epsilon_consumed: 0,
            epsilon_total,
        };

        env.storage()
            .instance()
            .set(&DataKey::PrivacyBudget(participant), &budget);

        Ok(true)
    }

    pub fn get_privacy_budget(env: Env, participant: Address) -> Option<PrivacyBudget> {
        env.storage()
            .instance()
            .get(&DataKey::PrivacyBudget(participant))
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_federated_round_flow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FederatedLearningContract);
        let client = FederatedLearningContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let participant1 = Address::generate(&env);
        let participant2 = Address::generate(&env);

        client.mock_all_auths().initialize(&admin, &coordinator);

        let base_model = BytesN::from_array(&env, &[1u8; 32]);
        let round_id = client
            .mock_all_auths()
            .start_round(&admin, &base_model, &2u32, &100u32);

        let update_hash1 = BytesN::from_array(&env, &[2u8; 32]);
        let update_hash2 = BytesN::from_array(&env, &[3u8; 32]);

        assert!(client
            .mock_all_auths()
            .submit_update(&participant1, &round_id, &update_hash1, &100u32)
            .is_ok());
        assert!(client
            .mock_all_auths()
            .submit_update(&participant2, &round_id, &update_hash2, &200u32)
            .is_ok());

        let new_model = BytesN::from_array(&env, &[4u8; 32]);
        assert!(client
            .mock_all_auths()
            .finalize_round(
                &coordinator,
                &round_id,
                &new_model,
                &String::from_str(&env, "Test model"),
                &String::from_str(&env, "ipfs://metrics"),
                &String::from_str(&env, "ipfs://fairness"),
            )
            .is_ok());

        let stored_round = client.get_round(&round_id).unwrap();
        assert!(stored_round.is_finalized);

        let stored_model = client.get_model(&new_model).unwrap();
        assert_eq!(stored_model.round_id, round_id);
    }

    #[test]
    fn test_privacy_budget_enforcement() {
        let env = Env::default();
        let contract_id = env.register_contract(None, FederatedLearningContract);
        let client = FederatedLearningContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let participant = Address::generate(&env);

        client.mock_all_auths().initialize(&admin, &coordinator);

        // Set a small privacy budget
        assert!(client
            .mock_all_auths()
            .set_privacy_budget(&admin, &participant, &10u32)
            .is_ok());

        let base_model = BytesN::from_array(&env, &[1u8; 32]);
        let round_id = client
            .mock_all_auths()
            .start_round(&admin, &base_model, &1u32, &100u32);

        let update_hash = BytesN::from_array(&env, &[2u8; 32]);

        // Submit an update that stays within budget
        assert!(client
            .mock_all_auths()
            .submit_update(&participant, &round_id, &update_hash, &500u32)
            .is_ok());

        // Submit an update that exceeds the budget
        let result = client.mock_all_auths().submit_update(
            &participant,
            &round_id,
            &update_hash,
            &600u32,
        );
        assert!(result.is_err());
    }
}
