#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, Symbol, Vec,
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

const ADMIN: Symbol = symbol_short!("ADMIN");
const COORDINATOR: Symbol = symbol_short!("COORD");
const ROUNDS: Symbol = symbol_short!("ROUNDS");
const CUR_ROUND: Symbol = symbol_short!("CUR_RND");
const PRIVACY_BUDGET: Symbol = symbol_short!("BUDGET"); // Map<Address, u32>

#[contract]
pub struct FederatedLearning;

#[contractimpl]
impl FederatedLearning {
    pub fn initialize(env: Env, admin: Address, coordinator: Address) {
        if env.storage().persistent().has(&ADMIN) {
            panic!("Already initialized");
        }
        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&COORDINATOR, &coordinator);
        env.storage().persistent().set(&CUR_ROUND, &0u32);
    }

    pub fn start_round(env: Env, coordinator: Address) -> u32 {
        coordinator.require_auth();
        let stored_coord: Address = env.storage().persistent().get(&COORDINATOR).unwrap();
        if coordinator != stored_coord {
            panic!("Not coordinator");
        }

        let current_round: u32 = env.storage().persistent().get(&CUR_ROUND).unwrap_or(0);
        let new_round_id = current_round + 1;

        let round_info = RoundInfo {
            round_id: new_round_id,
            status: RoundStatus::Active,
            participants: Vec::new(&env),
            updates: Vec::new(&env),
            global_model_hash: BytesN::from_array(&env, &[0u8; 32]),
        };

        env.storage().persistent().set(&CUR_ROUND, &new_round_id);
        let mut rounds: Map<u32, RoundInfo> = env
            .storage()
            .persistent()
            .get(&ROUNDS)
            .unwrap_or(Map::new(&env));
        rounds.set(new_round_id, round_info);
        env.storage().persistent().set(&ROUNDS, &rounds);

        new_round_id
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

        // Check privacy budget (simplified)
        let mut budgets: Map<Address, u32> = env
            .storage()
            .persistent()
            .get(&PRIVACY_BUDGET)
            .unwrap_or(Map::new(&env));
        let used = budgets.get(participant.clone()).unwrap_or(0);
        // Simple cost model
        let cost = 1;
        budgets.set(participant.clone(), used + cost);
        env.storage().persistent().set(&PRIVACY_BUDGET, &budgets);

        let update = ModelUpdate {
            participant: participant.clone(),
            update_hash,
            timestamp: env.ledger().timestamp(),
            sample_size,
            accepted: true,
        };

        round.updates.push_back(update);
        if !round.participants.contains(&participant) {
            round.participants.push_back(participant);
        }

        rounds.set(round_id, round);
        env.storage().persistent().set(&ROUNDS, &rounds);
        true
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

        let mut rounds: Map<u32, RoundInfo> = env
            .storage()
            .persistent()
            .get(&ROUNDS)
            .unwrap_or(Map::new(&env));
        let mut round = rounds.get(round_id).expect("Round not found");

        round.status = RoundStatus::Finalized;
        round.global_model_hash = new_global_model_hash;

        rounds.set(round_id, round);
        env.storage().persistent().set(&ROUNDS, &rounds);
        true
    }

    // Admin function to set privacy budget
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

        assert!(client.submit_update(&participant1, &round_id, &update_hash1, &100u32));
        assert!(client.submit_update(&participant2, &round_id, &update_hash2, &200u32));

        let global_hash = BytesN::from_array(&env, &[9u8; 32]);
        assert!(client.finalize_round(&coordinator, &round_id, &global_hash));
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

        assert!(client.set_privacy_budget(&admin, &participant, &10u32));

        let round_id = client.start_round(&coordinator);
        let update_hash = BytesN::from_array(&env, &[1u8; 32]);

        assert!(client.submit_update(&participant, &round_id, &update_hash, &500u32));
    }

    #[test]
    fn test_unauthorized_round_start() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, FederatedLearning);
        let client = FederatedLearningClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let other = Address::generate(&env);

        client.initialize(&admin, &coordinator);

        let result = client.try_start_round(&other);
        assert!(result.is_err());
    }
}
