#![no_std]
#![allow(clippy::arithmetic_side_effects, clippy::panic, clippy::unwrap_used)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env,
    String, Vec,
};

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum ModelType {
    CNN,
    RNN,
    Transformer,
    FeedForward,
    GNN,
    Hybrid,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum RoundStatus {
    Open,
    Aggregating,
    Finalized,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum InstitutionStatus {
    Active,
    Suspended,
    Blacklisted,
}

#[derive(Clone)]
#[contracttype]
pub struct Institution {
    pub id: Address,
    pub name: String,
    pub credential_hash: BytesN<32>,
    pub reputation_score: u32,
    pub total_contributions: u32,
    pub reward_balance: i128,
    pub status: InstitutionStatus,
    pub registered_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct RoundConfig {
    pub model_type: ModelType,
    pub min_participants: u32,
    pub max_participants: u32,
    pub dp_epsilon: u32,
    pub dp_delta: u32,
    pub reward_per_participant: i128,
    pub duration_seconds: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct FederatedRound {
    pub id: u64,
    pub base_model_id: BytesN<32>,
    pub model_type: ModelType,
    pub min_participants: u32,
    pub max_participants: u32,
    pub reward_per_participant: i128,
    pub total_updates: u32,
    pub status: RoundStatus,
    pub started_at: u64,
    pub deadline: u64,
    pub finalized_at: u64,
    pub aggregated_model_id: BytesN<32>,
}

#[derive(Clone)]
#[contracttype]
pub struct ModelOutput {
    pub model_id: BytesN<32>,
    pub description: String,
    pub weights_ref: String,
    pub global_accuracy: u32,
    pub validation_score: u32,
    pub version: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ModelMetadata {
    pub model_id: BytesN<32>,
    pub round_id: u64,
    pub model_type: ModelType,
    pub num_contributors: u32,
    pub validation_score: u32,
    pub version: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Coordinator,
    RoundCounter,
    Institution(Address),
    Round(u64),
    RoundParticipants(u64),
    UpdateSubmitted(u64, Address),
    Model(BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    RoundNotFound = 3,
    RoundNotOpen = 4,
    RoundNotAggregating = 5,
    RoundFinalized = 6,
    NotEnoughParticipants = 7,
    TooManyParticipants = 8,
    DuplicateUpdate = 9,
    InvalidDPParam = 10,
    InstitutionNotFound = 11,
    InstitutionNotActive = 12,
    InstitutionAlreadyRegistered = 13,
    LowReputation = 14,
    InvalidParameter = 15,
    DeadlineExceeded = 16,
    ValidationFailed = 17,
}

#[contract]
pub struct FederatedLearningContract;

#[contractimpl]
impl FederatedLearningContract {
    pub fn initialize(env: Env, admin: Address, coordinator: Address) -> Result<bool, Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Coordinator, &coordinator);
        Ok(true)
    }

    fn check_auth(env: &Env, caller: &Address, key: &DataKey) -> Result<(), Error> {
        let stored: Address = env
            .storage()
            .instance()
            .get(key)
            .unwrap_or_else(|| panic!("not initialized"));
        if stored != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn next_round_id(env: &Env) -> u64 {
        let n: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RoundCounter)
            .unwrap_or(0)
            + 1;
        env.storage().instance().set(&DataKey::RoundCounter, &n);
        n
    }

    pub fn register_institution(
        env: Env,
        admin: Address,
        institution: Address,
        name: String,
        credential_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::check_auth(&env, &admin, &DataKey::Admin)?;
        let key = DataKey::Institution(institution.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::InstitutionAlreadyRegistered);
        }
        env.storage().persistent().set(
            &key,
            &Institution {
                id: institution.clone(),
                name,
                credential_hash,
                reputation_score: 50,
                total_contributions: 0,
                reward_balance: 0,
                status: InstitutionStatus::Active,
                registered_at: env.ledger().timestamp(),
            },
        );
        env.events()
            .publish((symbol_short!("InstReg"),), institution);
        Ok(true)
    }

    pub fn start_round(
        env: Env,
        admin: Address,
        base_model_id: BytesN<32>,
        cfg: RoundConfig,
    ) -> Result<u64, Error> {
        admin.require_auth();
        Self::check_auth(&env, &admin, &DataKey::Admin)?;
        if cfg.min_participants == 0 || cfg.max_participants < cfg.min_participants {
            return Err(Error::InvalidParameter);
        }
        if cfg.dp_epsilon == 0 || cfg.dp_delta == 0 {
            return Err(Error::InvalidDPParam);
        }
        let id = Self::next_round_id(&env);
        let now = env.ledger().timestamp();
        env.storage().persistent().set(
            &DataKey::Round(id),
            &FederatedRound {
                id,
                base_model_id,
                model_type: cfg.model_type,
                min_participants: cfg.min_participants,
                max_participants: cfg.max_participants,
                reward_per_participant: cfg.reward_per_participant,
                total_updates: 0,
                status: RoundStatus::Open,
                started_at: now,
                deadline: now + cfg.duration_seconds,
                finalized_at: 0,
                aggregated_model_id: BytesN::from_array(&env, &[0u8; 32]),
            },
        );
        let empty: Vec<Address> = vec![&env];
        env.storage()
            .persistent()
            .set(&DataKey::RoundParticipants(id), &empty);
        env.events().publish((symbol_short!("RndStart"),), id);
        Ok(id)
    }

    pub fn submit_update(
        env: Env,
        institution: Address,
        round_id: u64,
        gradient_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        institution.require_auth();
        let inst_key = DataKey::Institution(institution.clone());
        let mut inst: Institution = env
            .storage()
            .persistent()
            .get(&inst_key)
            .ok_or(Error::InstitutionNotFound)?;
        if inst.status != InstitutionStatus::Active {
            return Err(Error::InstitutionNotActive);
        }
        if inst.reputation_score < 10 {
            return Err(Error::LowReputation);
        }
        let mut round: FederatedRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;
        if round.status != RoundStatus::Open {
            return Err(Error::RoundNotOpen);
        }
        if env.ledger().timestamp() > round.deadline {
            return Err(Error::DeadlineExceeded);
        }
        let upd_key = DataKey::UpdateSubmitted(round_id, institution.clone());
        if env.storage().persistent().has(&upd_key) {
            return Err(Error::DuplicateUpdate);
        }
        if round.total_updates >= round.max_participants {
            return Err(Error::TooManyParticipants);
        }
        env.storage().persistent().set(&upd_key, &gradient_hash);
        let mut participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(vec![&env]);
        participants.push_back(institution.clone());
        env.storage()
            .persistent()
            .set(&DataKey::RoundParticipants(round_id), &participants);
        round.total_updates += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);
        inst.total_contributions += 1;
        env.storage().persistent().set(&inst_key, &inst);
        env.events()
            .publish((symbol_short!("UpdSub"),), (round_id, institution));
        Ok(true)
    }

    pub fn begin_aggregation(env: Env, coordinator: Address, round_id: u64) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::check_auth(&env, &coordinator, &DataKey::Coordinator)?;
        let mut round: FederatedRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;
        if round.status != RoundStatus::Open {
            return Err(Error::RoundNotOpen);
        }
        if round.total_updates < round.min_participants {
            return Err(Error::NotEnoughParticipants);
        }
        round.status = RoundStatus::Aggregating;
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);
        env.events().publish((symbol_short!("AggStart"),), round_id);
        Ok(true)
    }

    pub fn finalize_round(
        env: Env,
        coordinator: Address,
        round_id: u64,
        out: ModelOutput,
    ) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::check_auth(&env, &coordinator, &DataKey::Coordinator)?;
        let mut round: FederatedRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;
        if round.status == RoundStatus::Finalized {
            return Err(Error::RoundFinalized);
        }
        if round.status != RoundStatus::Aggregating {
            return Err(Error::RoundNotAggregating);
        }
        if out.validation_score < 60 {
            return Err(Error::ValidationFailed);
        }
        let vscore = out.validation_score;
        let vid = out.model_id.clone();
        round.status = RoundStatus::Finalized;
        round.finalized_at = env.ledger().timestamp();
        round.aggregated_model_id = vid.clone();
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);
        env.storage().persistent().set(
            &DataKey::Model(vid.clone()),
            &ModelMetadata {
                model_id: vid.clone(),
                round_id,
                model_type: round.model_type.clone(),
                num_contributors: round.total_updates,
                validation_score: vscore,
                version: out.version,
            },
        );
        let participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(vec![&env]);
        let rep_delta: u32 = if vscore >= 90 {
            3
        } else if vscore >= 70 {
            2
        } else {
            1
        };
        for addr in participants.iter() {
            let k = DataKey::Institution(addr.clone());
            if let Some(mut inst) = env.storage().persistent().get::<DataKey, Institution>(&k) {
                inst.reward_balance += round.reward_per_participant;
                inst.reputation_score = (inst.reputation_score + rep_delta).min(100);
                env.storage().persistent().set(&k, &inst);
            }
        }
        env.events()
            .publish((symbol_short!("RndFin"),), (round_id, vid));
        Ok(true)
    }

    pub fn get_institution(env: Env, institution: Address) -> Option<Institution> {
        env.storage()
            .persistent()
            .get(&DataKey::Institution(institution))
    }

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        env.storage().persistent().get(&DataKey::Round(round_id))
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        env.storage().persistent().get(&DataKey::Model(model_id))
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup(env: &Env) -> (FederatedLearningContractClient, Address, Address) {
        let client = FederatedLearningContractClient::new(
            env,
            &env.register_contract(None, FederatedLearningContract),
        );
        let admin = Address::generate(env);
        let coord = Address::generate(env);
        client.mock_all_auths().initialize(&admin, &coord);
        (client, admin, coord)
    }

    fn add_inst(client: &FederatedLearningContractClient, env: &Env, admin: &Address) -> Address {
        let inst = Address::generate(env);
        client.mock_all_auths().register_institution(
            admin,
            &inst,
            &String::from_str(env, "Hospital"),
            &BytesN::from_array(env, &[9u8; 32]),
        );
        inst
    }

    fn default_cfg(_env: &Env, min_p: u32, reward: i128) -> RoundConfig {
        RoundConfig {
            model_type: ModelType::CNN,
            min_participants: min_p,
            max_participants: 10,
            dp_epsilon: 10,
            dp_delta: 5,
            reward_per_participant: reward,
            duration_seconds: 86400,
        }
    }

    #[test]
    fn test_round_lifecycle() {
        let env = Env::default();
        let (client, admin, coord) = setup(&env);
        let inst1 = add_inst(&client, &env, &admin);
        let inst2 = add_inst(&client, &env, &admin);
        let round_id = client.mock_all_auths().start_round(
            &admin,
            &BytesN::from_array(&env, &[1u8; 32]),
            &default_cfg(&env, 2, 50),
        );
        client.mock_all_auths().submit_update(
            &inst1,
            &round_id,
            &BytesN::from_array(&env, &[2u8; 32]),
        );
        client.mock_all_auths().submit_update(
            &inst2,
            &round_id,
            &BytesN::from_array(&env, &[3u8; 32]),
        );
        assert!(client.mock_all_auths().begin_aggregation(&coord, &round_id));
        let mid = BytesN::from_array(&env, &[4u8; 32]);
        client.mock_all_auths().finalize_round(
            &coord,
            &round_id,
            &ModelOutput {
                model_id: mid.clone(),
                description: String::from_str(&env, "model"),
                weights_ref: String::from_str(&env, "ipfs://w"),
                global_accuracy: 88,
                validation_score: 80,
                version: 1,
            },
        );
        assert_eq!(
            client.get_round(&round_id).unwrap().status,
            RoundStatus::Finalized
        );
        assert_eq!(client.get_model(&mid).unwrap().num_contributors, 2);
        assert!(client.get_institution(&inst1).unwrap().reward_balance >= 50);
        // finalized round rejects new updates
        assert!(client
            .mock_all_auths()
            .try_submit_update(&inst1, &round_id, &BytesN::from_array(&env, &[9u8; 32]))
            .unwrap()
            .is_err());
    }
}
