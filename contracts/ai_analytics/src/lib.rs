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

const ADMIN: Symbol = symbol_short!("ADMIN");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    RoundNotFound = 2,
    RoundFinalized = 3,
    NotEnoughParticipants = 4,
    DuplicateUpdate = 5,
}

#[contract]
pub struct AiAnalyticsContract;

#[contractimpl]
impl AiAnalyticsContract {
    pub fn initialize(env: Env, admin: Address) -> bool {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        true
    }

    fn ensure_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("AI analytics admin not set"));

        if admin != *caller {
            panic!("Not authorized: caller is not AI admin");
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
        env.events()
            .publish((symbol_short!("RndStart"),), id);
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

        let update = ParticipantUpdateMeta {
            round_id,
            participant: participant.clone(),
            update_hash,
            num_samples,
        };

        env.storage().instance().set(&key, &update);

        round.total_updates += 1;
        env.storage().instance().set(&DataKey::Round(round_id), &round);

        env.events().publish(
            (symbol_short!("UpdSubmit"),),
            (round_id, participant),
        );

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
        Self::ensure_admin(&env, &caller);

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

        env.events().publish(
            (symbol_short!("RndFinal"),),
            (round_id, new_model_id),
        );

        Ok(true)
    }

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        env.storage().instance().get(&DataKey::Round(round_id))
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        env.storage().instance().get(&DataKey::Model(model_id))
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_federated_round_flow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AiAnalyticsContract);
        let client = AiAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let participant1 = Address::generate(&env);
        let participant2 = Address::generate(&env);

        client.mock_all_auths().initialize(&admin);

        let base_model = BytesN::from_array(&env, &[1u8; 32]);
        let round_id = client
            .mock_all_auths()
            .start_round(&admin, &base_model, &2u32, &1u32);

        let update_hash1 = BytesN::from_array(&env, &[2u8; 32]);
        let update_hash2 = BytesN::from_array(&env, &[3u8; 32]);

        assert!(client
            .mock_all_auths()
            .submit_update(&participant1, &round_id, &update_hash1, &10u32)
            .is_ok());
        assert!(client
            .mock_all_auths()
            .submit_update(&participant2, &round_id, &update_hash2, &20u32)
            .is_ok());

        let new_model = BytesN::from_array(&env, &[4u8; 32]);
        assert!(client
            .mock_all_auths()
            .finalize_round(
                &admin,
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
}
