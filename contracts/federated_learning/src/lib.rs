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

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum RoundStatus {
    Open,
    Aggregating,
    Finalized,
    Cancelled,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum InstitutionStatus {
    Active,
    Suspended,
    Blacklisted,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum ListingStatus {
    Active,
    Sold,
    Withdrawn,
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
    pub noise_multiplier: u32,
    pub clipping_threshold: u32,
    pub mpc_threshold: u32,
    pub is_rare_disease: bool,
    pub disease_code: String,
    pub reward_per_participant: i128,
    pub duration_seconds: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ModelOutput {
    pub model_id: BytesN<32>,
    pub description: String,
    pub weights_ref: String,
    pub metrics_ref: String,
    pub global_accuracy: u32,
    pub validation_score: u32,
    pub version: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct FederatedRound {
    pub id: u64,
    pub base_model_id: BytesN<32>,
    pub model_type: ModelType,
    pub min_participants: u32,
    pub max_participants: u32,
    pub dp_epsilon: u32,
    pub mpc_threshold: u32,
    pub is_rare_disease: bool,
    pub disease_code: String,
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
pub struct ParticipantUpdate {
    pub institution: Address,
    pub gradient_hash: BytesN<32>,
    pub encrypted_ref: String,
    pub num_samples: u32,
    pub noise_applied: bool,
    pub submitted_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ModelMetadata {
    pub model_id: BytesN<32>,
    pub round_id: u64,
    pub model_type: ModelType,
    pub description: String,
    pub weights_ref: String,
    pub global_accuracy: u32,
    pub num_contributors: u32,
    pub validation_score: u32,
    pub is_rare_disease: bool,
    pub disease_code: String,
    pub created_at: u64,
    pub version: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct PrivacyBudget {
    pub epsilon_total: u32,
    pub epsilon_consumed: u32,
    pub delta_total: u32,
    pub delta_consumed: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct MPCSession {
    pub session_id: u64,
    pub round_id: u64,
    pub threshold: u32,
    pub committed_shares: u32,
    pub is_complete: bool,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct MarketplaceListing {
    pub listing_id: u64,
    pub model_id: BytesN<32>,
    pub seller: Address,
    pub price: i128,
    pub description: String,
    pub license_type: String,
    pub status: ListingStatus,
    pub listed_at: u64,
    pub buyer: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Coordinator,
    RoundCounter,
    MPCCounter,
    ListingCounter,
    ValidationThreshold,
    MarketplaceFeeBps,
    RareRewardMultiplier,
    MaxEpsilon,
    MinReputation,
    Institution(Address),
    Round(u64),
    RoundParticipants(u64),
    ParticipantUpdate(u64, Address),
    Model(BytesN<32>),
    PrivacyBudget(Address),
    MPCSession(u64),
    MPCShare(u64, Address),
    MarketplaceListing(u64),
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
    PrivacyBudgetExceeded = 10,
    InvalidDPParam = 11,
    InstitutionNotFound = 12,
    InstitutionNotActive = 13,
    InstitutionAlreadyRegistered = 14,
    LowReputation = 15,
    ModelNotFound = 16,
    MPCSessionNotFound = 17,
    MPCShareAlreadySubmitted = 18,
    MPCThresholdNotMet = 19,
    ListingNotFound = 20,
    ListingNotActive = 21,
    InsufficientFunds = 22,
    InvalidParameter = 23,
    DeadlineExceeded = 24,
    ValidationFailed = 25,
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
        env.storage().instance().set(&DataKey::Coordinator, &coordinator);
        env.storage().instance().set(&DataKey::ValidationThreshold, &60u32);
        env.storage().instance().set(&DataKey::MarketplaceFeeBps, &200u32);
        env.storage().instance().set(&DataKey::RareRewardMultiplier, &3u32);
        env.storage().instance().set(&DataKey::MaxEpsilon, &200u32);
        env.storage().instance().set(&DataKey::MinReputation, &10u32);
        Ok(true)
    }

    fn get_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("not initialized"))
    }

    fn get_coordinator(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Coordinator)
            .unwrap_or_else(|| panic!("not initialized"))
    }

    fn ensure_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        if Self::get_admin(env) != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn ensure_coordinator(env: &Env, caller: &Address) -> Result<(), Error> {
        if Self::get_coordinator(env) != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn next_u64(env: &Env, key: &DataKey) -> u64 {
        let n: u64 = env.storage().instance().get(key).unwrap_or(0) + 1;
        env.storage().instance().set(key, &n);
        n
    }

    fn cfg(env: &Env, key: &DataKey, default: u32) -> u32 {
        env.storage().instance().get(key).unwrap_or(default)
    }

    pub fn register_institution(
        env: Env,
        admin: Address,
        institution: Address,
        name: String,
        credential_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
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
        env.events().publish((symbol_short!("InstReg"),), institution);
        Ok(true)
    }

    pub fn update_institution_status(
        env: Env,
        admin: Address,
        institution: Address,
        status: InstitutionStatus,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        let key = DataKey::Institution(institution.clone());
        let mut inst: Institution = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::InstitutionNotFound)?;
        inst.status = status;
        env.storage().persistent().set(&key, &inst);
        Ok(true)
    }

    pub fn set_privacy_budget(
        env: Env,
        admin: Address,
        institution: Address,
        epsilon_total: u32,
        delta_total: u32,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        if epsilon_total == 0 || delta_total == 0 {
            return Err(Error::InvalidParameter);
        }
        env.storage().persistent().set(
            &DataKey::PrivacyBudget(institution),
            &PrivacyBudget {
                epsilon_total,
                epsilon_consumed: 0,
                delta_total,
                delta_consumed: 0,
            },
        );
        Ok(true)
    }

    fn consume_privacy_budget(
        env: &Env,
        institution: &Address,
        eps: u32,
        delta: u32,
        default_eps: u32,
    ) -> Result<(), Error> {
        let key = DataKey::PrivacyBudget(institution.clone());
        let mut b: PrivacyBudget = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(PrivacyBudget {
                epsilon_total: default_eps * 10,
                epsilon_consumed: 0,
                delta_total: 1000,
                delta_consumed: 0,
            });
        if b.epsilon_consumed + eps > b.epsilon_total
            || b.delta_consumed + delta > b.delta_total
        {
            return Err(Error::PrivacyBudgetExceeded);
        }
        b.epsilon_consumed += eps;
        b.delta_consumed += delta;
        env.storage().persistent().set(&key, &b);
        Ok(())
    }

    pub fn start_round(
        env: Env,
        admin: Address,
        base_model_id: BytesN<32>,
        cfg: RoundConfig,
    ) -> Result<u64, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;
        if cfg.min_participants == 0 || cfg.max_participants < cfg.min_participants {
            return Err(Error::InvalidParameter);
        }
        let max_eps = Self::cfg(&env, &DataKey::MaxEpsilon, 200);
        if cfg.dp_epsilon == 0 || cfg.dp_epsilon > max_eps {
            return Err(Error::InvalidDPParam);
        }
        if cfg.dp_delta == 0 {
            return Err(Error::InvalidDPParam);
        }
        let id = Self::next_u64(&env, &DataKey::RoundCounter);
        let now = env.ledger().timestamp();
        let round = FederatedRound {
            id,
            base_model_id,
            model_type: cfg.model_type,
            min_participants: cfg.min_participants,
            max_participants: cfg.max_participants,
            dp_epsilon: cfg.dp_epsilon,
            mpc_threshold: cfg.mpc_threshold,
            is_rare_disease: cfg.is_rare_disease,
            disease_code: cfg.disease_code,
            reward_per_participant: cfg.reward_per_participant,
            total_updates: 0,
            status: RoundStatus::Open,
            started_at: now,
            deadline: now + cfg.duration_seconds,
            finalized_at: 0,
            aggregated_model_id: BytesN::from_array(&env, &[0u8; 32]),
        };
        env.storage().persistent().set(&DataKey::Round(id), &round);
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
        encrypted_ref: String,
        num_samples: u32,
        noise_applied: bool,
        mpc_share_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        institution.require_auth();
        let inst: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(institution.clone()))
            .ok_or(Error::InstitutionNotFound)?;
        if inst.status != InstitutionStatus::Active {
            return Err(Error::InstitutionNotActive);
        }
        let min_rep = Self::cfg(&env, &DataKey::MinReputation, 10);
        if inst.reputation_score < min_rep {
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
        let upd_key = DataKey::ParticipantUpdate(round_id, institution.clone());
        if env.storage().persistent().has(&upd_key) {
            return Err(Error::DuplicateUpdate);
        }
        if round.total_updates >= round.max_participants {
            return Err(Error::TooManyParticipants);
        }
        let eps_cost = num_samples / 1000 + if noise_applied { 0 } else { 2 };
        Self::consume_privacy_budget(&env, &institution, eps_cost, 1, round.dp_epsilon)?;
        let _ = mpc_share_hash;
        env.storage().persistent().set(
            &upd_key,
            &ParticipantUpdate {
                institution: institution.clone(),
                gradient_hash,
                encrypted_ref,
                num_samples,
                noise_applied,
                submitted_at: env.ledger().timestamp(),
            },
        );
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
        env.storage().persistent().set(&DataKey::Round(round_id), &round);
        env.events()
            .publish((symbol_short!("UpdSub"),), (round_id, institution));
        Ok(true)
    }

    pub fn begin_aggregation(env: Env, coordinator: Address, round_id: u64) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::ensure_coordinator(&env, &coordinator)?;
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
        env.storage().persistent().set(&DataKey::Round(round_id), &round);
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
        Self::ensure_coordinator(&env, &coordinator)?;
        let val_threshold = Self::cfg(&env, &DataKey::ValidationThreshold, 60);
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
        if out.validation_score < val_threshold {
            return Err(Error::ValidationFailed);
        }
        let vscore = out.validation_score;
        let vid = out.model_id.clone();
        round.status = RoundStatus::Finalized;
        round.finalized_at = env.ledger().timestamp();
        round.aggregated_model_id = vid.clone();
        env.storage().persistent().set(&DataKey::Round(round_id), &round);
        env.storage().persistent().set(
            &DataKey::Model(vid.clone()),
            &ModelMetadata {
                model_id: vid.clone(),
                round_id,
                model_type: round.model_type.clone(),
                description: out.description,
                weights_ref: out.weights_ref,
                global_accuracy: out.global_accuracy,
                num_contributors: round.total_updates,
                validation_score: vscore,
                is_rare_disease: round.is_rare_disease,
                disease_code: round.disease_code.clone(),
                created_at: round.finalized_at,
                version: out.version,
            },
        );
        let participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(vec![&env]);
        let rare_mult = Self::cfg(&env, &DataKey::RareRewardMultiplier, 3);
        let reward_mult: i128 = if round.is_rare_disease { rare_mult as i128 } else { 1 };
        for addr in participants.iter() {
            let k = DataKey::Institution(addr.clone());
            if let Some(mut inst) = env.storage().persistent().get::<DataKey, Institution>(&k) {
                inst.reward_balance += round.reward_per_participant * reward_mult;
                inst.total_contributions += 1;
                let rep = if vscore >= 90 {
                    3u32
                } else if vscore >= 70 {
                    2u32
                } else {
                    1u32
                };
                inst.reputation_score = (inst.reputation_score + rep).min(100);
                env.storage().persistent().set(&k, &inst);
            }
        }
        env.events().publish((symbol_short!("RndFin"),), (round_id, vid));
        Ok(true)
    }

    pub fn create_mpc_session(
        env: Env,
        coordinator: Address,
        round_id: u64,
        threshold: u32,
    ) -> Result<u64, Error> {
        coordinator.require_auth();
        Self::ensure_coordinator(&env, &coordinator)?;
        let round: FederatedRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;
        if matches!(round.status, RoundStatus::Finalized | RoundStatus::Cancelled) {
            return Err(Error::RoundFinalized);
        }
        if threshold == 0 || threshold > round.total_updates {
            return Err(Error::InvalidParameter);
        }
        let sid = Self::next_u64(&env, &DataKey::MPCCounter);
        env.storage().persistent().set(
            &DataKey::MPCSession(sid),
            &MPCSession {
                session_id: sid,
                round_id,
                threshold,
                committed_shares: 0,
                is_complete: false,
                created_at: env.ledger().timestamp(),
            },
        );
        env.events().publish((symbol_short!("MPCNew"),), sid);
        Ok(sid)
    }

    pub fn submit_mpc_share(
        env: Env,
        institution: Address,
        session_id: u64,
        share_hash: BytesN<32>,
        commitment: BytesN<32>,
    ) -> Result<bool, Error> {
        institution.require_auth();
        let share_key = DataKey::MPCShare(session_id, institution.clone());
        if env.storage().persistent().has(&share_key) {
            return Err(Error::MPCShareAlreadySubmitted);
        }
        let mut session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::MPCSession(session_id))
            .ok_or(Error::MPCSessionNotFound)?;
        if session.is_complete {
            return Err(Error::RoundFinalized);
        }
        env.storage()
            .persistent()
            .set(&share_key, &(share_hash, commitment));
        session.committed_shares += 1;
        if session.committed_shares >= session.threshold {
            session.is_complete = true;
            env.events().publish((symbol_short!("MPCDone"),), session_id);
        }
        env.storage()
            .persistent()
            .set(&DataKey::MPCSession(session_id), &session);
        Ok(true)
    }

    pub fn list_model(
        env: Env,
        seller: Address,
        model_id: BytesN<32>,
        price: i128,
        description: String,
        license_type: String,
    ) -> Result<u64, Error> {
        seller.require_auth();
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Institution(seller.clone()))
        {
            return Err(Error::InstitutionNotFound);
        }
        if !env.storage().persistent().has(&DataKey::Model(model_id.clone())) {
            return Err(Error::ModelNotFound);
        }
        if price < 0 {
            return Err(Error::InvalidParameter);
        }
        let id = Self::next_u64(&env, &DataKey::ListingCounter);
        env.storage().persistent().set(
            &DataKey::MarketplaceListing(id),
            &MarketplaceListing {
                listing_id: id,
                model_id,
                seller: seller.clone(),
                price,
                description,
                license_type,
                status: ListingStatus::Active,
                listed_at: env.ledger().timestamp(),
                buyer: seller,
            },
        );
        env.events().publish((symbol_short!("MktList"),), id);
        Ok(id)
    }

    pub fn purchase_model(env: Env, buyer: Address, listing_id: u64) -> Result<bool, Error> {
        buyer.require_auth();
        let mut listing: MarketplaceListing = env
            .storage()
            .persistent()
            .get(&DataKey::MarketplaceListing(listing_id))
            .ok_or(Error::ListingNotFound)?;
        if listing.status != ListingStatus::Active {
            return Err(Error::ListingNotActive);
        }
        let buyer_key = DataKey::Institution(buyer.clone());
        let mut buyer_inst: Institution = env
            .storage()
            .persistent()
            .get(&buyer_key)
            .ok_or(Error::InstitutionNotFound)?;
        if buyer_inst.reward_balance < listing.price {
            return Err(Error::InsufficientFunds);
        }
        let fee_bps = Self::cfg(&env, &DataKey::MarketplaceFeeBps, 200);
        let fee = listing.price * fee_bps as i128 / 10000;
        buyer_inst.reward_balance -= listing.price;
        env.storage().persistent().set(&buyer_key, &buyer_inst);
        let seller_key = DataKey::Institution(listing.seller.clone());
        if let Some(mut seller_inst) = env
            .storage()
            .persistent()
            .get::<DataKey, Institution>(&seller_key)
        {
            seller_inst.reward_balance += listing.price - fee;
            env.storage().persistent().set(&seller_key, &seller_inst);
        }
        listing.status = ListingStatus::Sold;
        listing.buyer = buyer.clone();
        env.storage()
            .persistent()
            .set(&DataKey::MarketplaceListing(listing_id), &listing);
        env.events()
            .publish((symbol_short!("MktSold"),), (listing_id, buyer));
        Ok(true)
    }

    pub fn withdraw_rewards(env: Env, institution: Address, amount: i128) -> Result<i128, Error> {
        institution.require_auth();
        let key = DataKey::Institution(institution.clone());
        let mut inst: Institution = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::InstitutionNotFound)?;
        if amount <= 0 || inst.reward_balance < amount {
            return Err(Error::InsufficientFunds);
        }
        inst.reward_balance -= amount;
        env.storage().persistent().set(&key, &inst);
        env.events()
            .publish((symbol_short!("Withdraw"),), (institution, amount));
        Ok(inst.reward_balance)
    }

    pub fn get_institution(env: Env, institution: Address) -> Option<Institution> {
        env.storage().persistent().get(&DataKey::Institution(institution))
    }

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        env.storage().persistent().get(&DataKey::Round(round_id))
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        env.storage().persistent().get(&DataKey::Model(model_id))
    }

    pub fn get_privacy_budget(env: Env, institution: Address) -> Option<PrivacyBudget> {
        env.storage().persistent().get(&DataKey::PrivacyBudget(institution))
    }

    pub fn get_mpc_session(env: Env, session_id: u64) -> Option<MPCSession> {
        env.storage().persistent().get(&DataKey::MPCSession(session_id))
    }

    pub fn get_listing(env: Env, listing_id: u64) -> Option<MarketplaceListing> {
        env.storage().persistent().get(&DataKey::MarketplaceListing(listing_id))
    }

    pub fn get_round_participants(env: Env, round_id: u64) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or_else(|| vec![&env])
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

    fn add_inst(
        client: &FederatedLearningContractClient,
        env: &Env,
        admin: &Address,
    ) -> Address {
        let inst = Address::generate(env);
        client.mock_all_auths().register_institution(
            admin,
            &inst,
            &String::from_str(env, "Hospital"),
            &BytesN::from_array(env, &[9u8; 32]),
        );
        inst
    }

    fn default_cfg(env: &Env, min_p: u32, reward: i128) -> RoundConfig {
        RoundConfig {
            model_type: ModelType::CNN,
            min_participants: min_p,
            max_participants: 10,
            dp_epsilon: 10,
            dp_delta: 5,
            noise_multiplier: 100,
            clipping_threshold: 50,
            mpc_threshold: 3,
            is_rare_disease: false,
            disease_code: String::from_str(env, ""),
            reward_per_participant: reward,
            duration_seconds: 86400,
        }
    }

    fn do_submit(
        client: &FederatedLearningContractClient,
        env: &Env,
        inst: &Address,
        round_id: u64,
        seed: u8,
    ) {
        client.mock_all_auths().submit_update(
            inst,
            &round_id,
            &BytesN::from_array(env, &[seed; 32]),
            &String::from_str(env, "ipfs://enc"),
            &500u32,
            &true,
            &BytesN::from_array(env, &[5u8; 32]),
        );
    }

    fn do_finalize(
        client: &FederatedLearningContractClient,
        env: &Env,
        coord: &Address,
        round_id: u64,
        model_seed: u8,
    ) -> BytesN<32> {
        let mid = BytesN::from_array(env, &[model_seed; 32]);
        client.mock_all_auths().finalize_round(
            coord,
            &round_id,
            &ModelOutput {
                model_id: mid.clone(),
                description: String::from_str(env, "model"),
                weights_ref: String::from_str(env, "ipfs://w"),
                metrics_ref: String::from_str(env, "ipfs://m"),
                global_accuracy: 88,
                validation_score: 80,
                version: 1,
            },
        );
        mid
    }

    #[test]
    fn test_round_lifecycle() {
        let env = Env::default();
        let (client, admin, coord) = setup(&env);
        let inst1 = add_inst(&client, &env, &admin);
        let inst2 = add_inst(&client, &env, &admin);
        let round_id = client
            .mock_all_auths()
            .start_round(&admin, &BytesN::from_array(&env, &[1u8; 32]), &default_cfg(&env, 2, 50));
        do_submit(&client, &env, &inst1, round_id, 2);
        do_submit(&client, &env, &inst2, round_id, 3);
        assert!(client
            .mock_all_auths()
            .begin_aggregation(&coord, &round_id));
        let mid = do_finalize(&client, &env, &coord, round_id, 4);
        assert_eq!(client.get_round(&round_id).unwrap().status, RoundStatus::Finalized);
        assert_eq!(client.get_model(&mid).unwrap().num_contributors, 2);
        assert!(client.get_institution(&inst1).unwrap().reward_balance >= 50);
    }

    #[test]
    fn test_dp_budget_enforcement() {
        let env = Env::default();
        let (client, admin, _) = setup(&env);
        let inst = add_inst(&client, &env, &admin);
        // budget of 3 epsilon; 1000 samples + no noise => cost = 1 + 2 = 3 (exhausts)
        assert!(client
            .mock_all_auths()
            .set_privacy_budget(&admin, &inst, &3u32, &5u32));
        let r1 = client
            .mock_all_auths()
            .start_round(&admin, &BytesN::from_array(&env, &[1u8; 32]), &default_cfg(&env, 1, 10));
        assert!(client
            .mock_all_auths()
            .try_submit_update(
                &inst,
                &r1,
                &BytesN::from_array(&env, &[2u8; 32]),
                &String::from_str(&env, "ipfs://enc"),
                &1000u32,
                &false,
                &BytesN::from_array(&env, &[5u8; 32]),
            )
            .unwrap()
            .is_ok());
        // second round should fail: budget exhausted
        let r2 = client
            .mock_all_auths()
            .start_round(&admin, &BytesN::from_array(&env, &[1u8; 32]), &default_cfg(&env, 1, 10));
        assert!(client
            .mock_all_auths()
            .try_submit_update(
                &inst,
                &r2,
                &BytesN::from_array(&env, &[3u8; 32]),
                &String::from_str(&env, "ipfs://enc"),
                &1000u32,
                &false,
                &BytesN::from_array(&env, &[5u8; 32]),
            )
            .unwrap()
            .is_err());
    }

    #[test]
    fn test_rare_disease_reward_multiplier() {
        let env = Env::default();
        let (client, admin, coord) = setup(&env);
        let inst = add_inst(&client, &env, &admin);
        let round_id = client.mock_all_auths().start_round(
            &admin,
            &BytesN::from_array(&env, &[1u8; 32]),
            &RoundConfig {
                model_type: ModelType::RNN,
                min_participants: 1,
                max_participants: 10,
                dp_epsilon: 10,
                dp_delta: 5,
                noise_multiplier: 100,
                clipping_threshold: 50,
                mpc_threshold: 1,
                is_rare_disease: true,
                disease_code: String::from_str(&env, "ORPHA:99999"),
                reward_per_participant: 50,
                duration_seconds: 86400,
            },
        );
        do_submit(&client, &env, &inst, round_id, 2);
        assert!(client
            .mock_all_auths()
            .begin_aggregation(&coord, &round_id));
        do_finalize(&client, &env, &coord, round_id, 4);
        // 50 * 3 (default rare multiplier) = 150
        assert_eq!(client.get_institution(&inst).unwrap().reward_balance, 150);
    }

    #[test]
    fn test_mpc_session() {
        let env = Env::default();
        let (client, admin, coord) = setup(&env);
        let inst1 = add_inst(&client, &env, &admin);
        let inst2 = add_inst(&client, &env, &admin);
        let inst3 = add_inst(&client, &env, &admin);
        let round_id = client
            .mock_all_auths()
            .start_round(&admin, &BytesN::from_array(&env, &[1u8; 32]), &default_cfg(&env, 3, 50));
        do_submit(&client, &env, &inst1, round_id, 2);
        do_submit(&client, &env, &inst2, round_id, 3);
        do_submit(&client, &env, &inst3, round_id, 4);
        let sid = client
            .mock_all_auths()
            .create_mpc_session(&coord, &round_id, &2u32);
        let sh = BytesN::from_array(&env, &[7u8; 32]);
        let cm = BytesN::from_array(&env, &[8u8; 32]);
        assert!(client
            .mock_all_auths()
            .submit_mpc_share(&inst1, &sid, &sh, &cm));
        assert!(client
            .mock_all_auths()
            .submit_mpc_share(&inst2, &sid, &sh, &cm));
        assert!(client.get_mpc_session(&sid).unwrap().is_complete);
    }

    #[test]
    fn test_marketplace() {
        let env = Env::default();
        let (client, admin, coord) = setup(&env);
        let seller = add_inst(&client, &env, &admin);
        let buyer = add_inst(&client, &env, &admin);
        let r1 = client
            .mock_all_auths()
            .start_round(&admin, &BytesN::from_array(&env, &[1u8; 32]), &default_cfg(&env, 1, 500));
        do_submit(&client, &env, &seller, r1, 2);
        assert!(client.mock_all_auths().begin_aggregation(&coord, &r1));
        let model_id = do_finalize(&client, &env, &coord, r1, 4);
        let r2 = client
            .mock_all_auths()
            .start_round(&admin, &BytesN::from_array(&env, &[1u8; 32]), &default_cfg(&env, 1, 300));
        do_submit(&client, &env, &buyer, r2, 3);
        assert!(client.mock_all_auths().begin_aggregation(&coord, &r2));
        do_finalize(&client, &env, &coord, r2, 6);
        let lid = client.mock_all_auths().list_model(
            &seller,
            &model_id,
            &200i128,
            &String::from_str(&env, "CNN cardiac model"),
            &String::from_str(&env, "CC-BY-NC"),
        );
        let before = client.get_institution(&buyer).unwrap().reward_balance;
        assert!(before >= 200);
        assert!(client.mock_all_auths().purchase_model(&buyer, &lid));
        assert_eq!(client.get_listing(&lid).unwrap().status, ListingStatus::Sold);
        assert_eq!(
            client.get_institution(&buyer).unwrap().reward_balance,
            before - 200
        );
    }
}
