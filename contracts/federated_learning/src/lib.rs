#![no_std]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env,
    String, Vec,
};

// ─── Model Types ─────────────────────────────────────────────────────────────

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
    Validating,
    Finalized,
    Cancelled,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum InstitutionStatus {
    Pending,
    Active,
    Suspended,
    Blacklisted,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum MarketplaceListingStatus {
    Active,
    Sold,
    Withdrawn,
}

// ─── Core Structs ─────────────────────────────────────────────────────────────

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
    pub data_quality_score: u32,
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
    pub dp_delta: u32,
    pub noise_multiplier: u32,
    pub clipping_threshold: u32,
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
    pub round_id: u64,
    pub institution: Address,
    pub gradient_hash: BytesN<32>,
    pub encrypted_update_ref: String,
    pub num_samples: u32,
    pub local_loss: u32,
    pub local_accuracy: u32,
    pub noise_applied: bool,
    pub clipping_applied: bool,
    pub mpc_share_hash: BytesN<32>,
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
    pub metrics_ref: String,
    pub fairness_report_ref: String,
    pub global_loss: u32,
    pub global_accuracy: u32,
    pub num_contributors: u32,
    pub validation_score: u32,
    pub is_rare_disease_model: bool,
    pub disease_code: String,
    pub created_at: u64,
    pub version: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ModelValidation {
    pub model_id: BytesN<32>,
    pub validator: Address,
    pub accuracy_score: u32,
    pub fairness_score: u32,
    pub robustness_score: u32,
    pub bias_score: u32,
    pub overall_score: u32,
    pub passed: bool,
    pub report_ref: String,
    pub validated_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PrivacyBudget {
    pub institution: Address,
    pub epsilon_total: u32,
    pub epsilon_consumed: u32,
    pub delta_total: u32,
    pub delta_consumed: u32,
    pub last_reset: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct MPCSession {
    pub session_id: u64,
    pub round_id: u64,
    pub threshold: u32,
    pub total_shares: u32,
    pub committed_shares: u32,
    pub is_complete: bool,
    pub result_hash: BytesN<32>,
    pub created_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct MPCShare {
    pub session_id: u64,
    pub institution: Address,
    pub share_hash: BytesN<32>,
    pub commitment: BytesN<32>,
    pub submitted_at: u64,
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
    pub status: MarketplaceListingStatus,
    pub listed_at: u64,
    pub sold_at: u64,
    pub buyer: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct AuditEntry {
    pub entry_id: u64,
    pub actor: Address,
    pub action: String,
    pub target_id: String,
    pub details: String,
    pub timestamp: u64,
    pub round_id: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct RareDiseaseRegistry {
    pub disease_code: String,
    pub description: String,
    pub registered_by: Address,
    pub model_count: u32,
    pub participant_count: u32,
    pub registered_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PlatformConfig {
    pub min_reputation_to_participate: u32,
    pub base_reward_amount: i128,
    pub rare_disease_reward_multiplier: u32,
    pub validation_threshold: u32,
    pub max_epsilon_per_round: u32,
    pub min_data_quality_score: u32,
    pub marketplace_fee_bps: u32,
    pub mpc_default_threshold: u32,
}

// ─── Storage Keys ─────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Coordinator,
    Config,
    InstitutionCount,
    RoundCounter,
    MPCSessionCounter,
    ListingCounter,
    AuditCounter,
    Institution(Address),
    Round(u64),
    RoundParticipants(u64),
    ParticipantUpdate(u64, Address),
    Model(BytesN<32>),
    ModelValidation(BytesN<32>),
    PrivacyBudget(Address),
    MPCSession(u64),
    MPCShare(u64, Address),
    MarketplaceListing(u64),
    AuditEntry(u64),
    RareDiseaseRegistry(String),
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    RoundNotFound = 4,
    RoundNotOpen = 5,
    RoundNotAggregating = 6,
    RoundFinalized = 7,
    NotEnoughParticipants = 8,
    TooManyParticipants = 9,
    DuplicateUpdate = 10,
    InvalidPrivacyBudget = 11,
    PrivacyBudgetExceeded = 12,
    InvalidDPParameter = 13,
    InstitutionNotFound = 14,
    InstitutionNotActive = 15,
    InstitutionAlreadyRegistered = 16,
    LowReputation = 17,
    LowDataQuality = 18,
    ModelNotFound = 19,
    ValidationFailed = 20,
    MPCSessionNotFound = 21,
    MPCShareAlreadySubmitted = 22,
    MPCThresholdNotMet = 23,
    ListingNotFound = 24,
    ListingNotActive = 25,
    InsufficientFunds = 26,
    InvalidParameter = 27,
    RareDiseaseNotRegistered = 28,
    DeadlineExceeded = 29,
    RoundDeadlineNotPassed = 30,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct FederatedLearningContract;

#[contractimpl]
impl FederatedLearningContract {
    // ── Initialization ────────────────────────────────────────────────────

    pub fn initialize(
        env: Env,
        admin: Address,
        coordinator: Address,
        config: PlatformConfig,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Coordinator, &coordinator);
        env.storage().instance().set(&DataKey::Config, &config);
        env.storage()
            .instance()
            .set(&DataKey::RoundCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::MPCSessionCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::ListingCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::AuditCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::InstitutionCount, &0u32);

        Self::emit_audit(
            &env,
            &admin,
            String::from_str(&env, "INITIALIZE"),
            String::from_str(&env, "platform"),
            String::from_str(&env, "Platform initialized"),
            0,
        );

        Ok(true)
    }

    // ── Admin Helpers ─────────────────────────────────────────────────────

    fn get_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Not initialized"))
    }

    fn get_coordinator(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Coordinator)
            .unwrap_or_else(|| panic!("Not initialized"))
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

    fn get_config(env: &Env) -> PlatformConfig {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .unwrap_or_else(|| panic!("Config not set"))
    }

    fn next_id_u64(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(key, &next);
        next
    }

    // ── Audit ─────────────────────────────────────────────────────────────

    fn emit_audit(
        env: &Env,
        actor: &Address,
        action: String,
        target_id: String,
        details: String,
        round_id: u64,
    ) {
        let entry_id = Self::next_id_u64(env, &DataKey::AuditCounter);
        let entry = AuditEntry {
            entry_id,
            actor: actor.clone(),
            action,
            target_id,
            details,
            timestamp: env.ledger().timestamp(),
            round_id,
        };
        env.storage()
            .persistent()
            .set(&DataKey::AuditEntry(entry_id), &entry);
        env.events().publish((symbol_short!("Audit"),), entry_id);
    }

    // ── Institution Management ─────────────────────────────────────────────

    pub fn register_institution(
        env: Env,
        admin: Address,
        institution: Address,
        name: String,
        credential_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Institution(institution.clone()))
        {
            return Err(Error::InstitutionAlreadyRegistered);
        }

        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::InstitutionCount)
            .unwrap_or(0);

        let inst = Institution {
            id: institution.clone(),
            name: name.clone(),
            credential_hash,
            reputation_score: 50,
            total_contributions: 0,
            reward_balance: 0,
            status: InstitutionStatus::Active,
            registered_at: env.ledger().timestamp(),
            data_quality_score: 80,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Institution(institution.clone()), &inst);
        env.storage()
            .instance()
            .set(&DataKey::InstitutionCount, &(count + 1));

        env.events()
            .publish((symbol_short!("InstReg"),), institution.clone());

        Self::emit_audit(
            &env,
            &admin,
            String::from_str(&env, "REGISTER_INSTITUTION"),
            name,
            String::from_str(&env, "Institution registered"),
            0,
        );

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

        let mut inst: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(institution.clone()))
            .ok_or(Error::InstitutionNotFound)?;

        inst.status = status;
        env.storage()
            .persistent()
            .set(&DataKey::Institution(institution.clone()), &inst);

        Self::emit_audit(
            &env,
            &admin,
            String::from_str(&env, "UPDATE_INST_STATUS"),
            String::from_str(&env, "institution"),
            String::from_str(&env, "Status updated"),
            0,
        );

        Ok(true)
    }

    pub fn update_data_quality_score(
        env: Env,
        coordinator: Address,
        institution: Address,
        score: u32,
    ) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::ensure_coordinator(&env, &coordinator)?;

        if score > 100 {
            return Err(Error::InvalidParameter);
        }

        let mut inst: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(institution.clone()))
            .ok_or(Error::InstitutionNotFound)?;

        inst.data_quality_score = score;
        env.storage()
            .persistent()
            .set(&DataKey::Institution(institution.clone()), &inst);

        Ok(true)
    }

    pub fn get_institution(env: Env, institution: Address) -> Option<Institution> {
        env.storage()
            .persistent()
            .get(&DataKey::Institution(institution))
    }

    // ── Privacy Budget ─────────────────────────────────────────────────────

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
            return Err(Error::InvalidPrivacyBudget);
        }

        let budget = PrivacyBudget {
            institution: institution.clone(),
            epsilon_total,
            epsilon_consumed: 0,
            delta_total,
            delta_consumed: 0,
            last_reset: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::PrivacyBudget(institution), &budget);

        Ok(true)
    }

    pub fn reset_privacy_budget(
        env: Env,
        admin: Address,
        institution: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;

        let mut budget: PrivacyBudget = env
            .storage()
            .persistent()
            .get(&DataKey::PrivacyBudget(institution.clone()))
            .ok_or(Error::InvalidPrivacyBudget)?;

        budget.epsilon_consumed = 0;
        budget.delta_consumed = 0;
        budget.last_reset = env.ledger().timestamp();
        env.storage()
            .persistent()
            .set(&DataKey::PrivacyBudget(institution), &budget);

        Ok(true)
    }

    pub fn get_privacy_budget(env: Env, institution: Address) -> Option<PrivacyBudget> {
        env.storage()
            .persistent()
            .get(&DataKey::PrivacyBudget(institution))
    }

    fn check_and_consume_privacy_budget(
        env: &Env,
        institution: &Address,
        epsilon_cost: u32,
        delta_cost: u32,
        dp_epsilon: u32,
    ) -> Result<(), Error> {
        let budget_key = DataKey::PrivacyBudget(institution.clone());
        let mut budget: PrivacyBudget =
            env.storage()
                .persistent()
                .get(&budget_key)
                .unwrap_or(PrivacyBudget {
                    institution: institution.clone(),
                    epsilon_total: dp_epsilon * 10,
                    epsilon_consumed: 0,
                    delta_total: 1000,
                    delta_consumed: 0,
                    last_reset: env.ledger().timestamp(),
                });

        if budget.epsilon_consumed + epsilon_cost > budget.epsilon_total {
            return Err(Error::PrivacyBudgetExceeded);
        }
        if budget.delta_consumed + delta_cost > budget.delta_total {
            return Err(Error::PrivacyBudgetExceeded);
        }

        budget.epsilon_consumed += epsilon_cost;
        budget.delta_consumed += delta_cost;
        env.storage().persistent().set(&budget_key, &budget);

        Ok(())
    }

    // ── Rare Disease Registry ──────────────────────────────────────────────

    pub fn register_rare_disease(
        env: Env,
        admin: Address,
        disease_code: String,
        description: String,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;

        let registry = RareDiseaseRegistry {
            disease_code: disease_code.clone(),
            description,
            registered_by: admin.clone(),
            model_count: 0,
            participant_count: 0,
            registered_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::RareDiseaseRegistry(disease_code.clone()), &registry);

        env.events()
            .publish((symbol_short!("RareDis"),), disease_code.clone());

        Self::emit_audit(
            &env,
            &admin,
            String::from_str(&env, "REGISTER_RARE_DIS"),
            disease_code,
            String::from_str(&env, "Rare disease registered"),
            0,
        );

        Ok(true)
    }

    pub fn get_rare_disease(env: Env, disease_code: String) -> Option<RareDiseaseRegistry> {
        env.storage()
            .persistent()
            .get(&DataKey::RareDiseaseRegistry(disease_code))
    }

    // ── Federated Learning Rounds ──────────────────────────────────────────

    pub fn start_round(
        env: Env,
        admin: Address,
        base_model_id: BytesN<32>,
        model_type: ModelType,
        min_participants: u32,
        max_participants: u32,
        dp_epsilon: u32,
        dp_delta: u32,
        noise_multiplier: u32,
        clipping_threshold: u32,
        mpc_threshold: u32,
        is_rare_disease: bool,
        disease_code: String,
        reward_per_participant: i128,
        duration_seconds: u64,
    ) -> Result<u64, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;

        let config = Self::get_config(&env);

        if min_participants == 0 || max_participants < min_participants {
            return Err(Error::InvalidParameter);
        }

        if dp_epsilon == 0 || dp_epsilon > config.max_epsilon_per_round {
            return Err(Error::InvalidDPParameter);
        }

        if dp_delta == 0 {
            return Err(Error::InvalidDPParameter);
        }

        if is_rare_disease
            && !env
                .storage()
                .persistent()
                .has(&DataKey::RareDiseaseRegistry(disease_code.clone()))
        {
            return Err(Error::RareDiseaseNotRegistered);
        }

        let id = Self::next_id_u64(&env, &DataKey::RoundCounter);
        let now = env.ledger().timestamp();

        let round = FederatedRound {
            id,
            base_model_id,
            model_type,
            min_participants,
            max_participants,
            dp_epsilon,
            dp_delta,
            noise_multiplier,
            clipping_threshold,
            mpc_threshold,
            is_rare_disease,
            disease_code,
            reward_per_participant,
            total_updates: 0,
            status: RoundStatus::Open,
            started_at: now,
            deadline: now + duration_seconds,
            finalized_at: 0,
            aggregated_model_id: BytesN::from_array(&env, &[0u8; 32]),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Round(id), &round);

        let participants: Vec<Address> = vec![&env];
        env.storage()
            .persistent()
            .set(&DataKey::RoundParticipants(id), &participants);

        env.events().publish((symbol_short!("RndStart"),), id);

        Self::emit_audit(
            &env,
            &admin,
            String::from_str(&env, "START_ROUND"),
            String::from_str(&env, "round"),
            String::from_str(&env, "Federated round started"),
            id,
        );

        Ok(id)
    }

    pub fn submit_update(
        env: Env,
        institution: Address,
        round_id: u64,
        gradient_hash: BytesN<32>,
        encrypted_update_ref: String,
        num_samples: u32,
        local_loss: u32,
        local_accuracy: u32,
        mpc_share_hash: BytesN<32>,
        noise_applied: bool,
        clipping_applied: bool,
    ) -> Result<bool, Error> {
        institution.require_auth();

        let inst_data: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(institution.clone()))
            .ok_or(Error::InstitutionNotFound)?;

        if inst_data.status != InstitutionStatus::Active {
            return Err(Error::InstitutionNotActive);
        }

        let config = Self::get_config(&env);

        if inst_data.reputation_score < config.min_reputation_to_participate {
            return Err(Error::LowReputation);
        }

        if inst_data.data_quality_score < config.min_data_quality_score {
            return Err(Error::LowDataQuality);
        }

        let mut round: FederatedRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;

        if round.status != RoundStatus::Open {
            return Err(Error::RoundNotOpen);
        }

        let now = env.ledger().timestamp();
        if now > round.deadline {
            return Err(Error::DeadlineExceeded);
        }

        let update_key = DataKey::ParticipantUpdate(round_id, institution.clone());
        if env.storage().persistent().has(&update_key) {
            return Err(Error::DuplicateUpdate);
        }

        if round.total_updates >= round.max_participants {
            return Err(Error::TooManyParticipants);
        }

        // Gaussian mechanism: epsilon cost proportional to samples; penalty if noise skipped
        let epsilon_cost = num_samples / 1000 + if noise_applied { 0 } else { 2 };
        let delta_cost = 1u32;

        Self::check_and_consume_privacy_budget(
            &env,
            &institution,
            epsilon_cost,
            delta_cost,
            round.dp_epsilon,
        )?;

        let update = ParticipantUpdate {
            round_id,
            institution: institution.clone(),
            gradient_hash,
            encrypted_update_ref,
            num_samples,
            local_loss,
            local_accuracy,
            noise_applied,
            clipping_applied,
            mpc_share_hash,
            submitted_at: now,
        };

        env.storage().persistent().set(&update_key, &update);

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

        env.events()
            .publish((symbol_short!("UpdSub"),), (round_id, institution.clone()));

        Self::emit_audit(
            &env,
            &institution,
            String::from_str(&env, "SUBMIT_UPDATE"),
            String::from_str(&env, "round"),
            String::from_str(&env, "Gradient update submitted"),
            round_id,
        );

        Ok(true)
    }

    pub fn begin_aggregation(
        env: Env,
        coordinator: Address,
        round_id: u64,
    ) -> Result<bool, Error> {
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
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);

        env.events()
            .publish((symbol_short!("AggStart"),), round_id);

        Ok(true)
    }

    pub fn finalize_round(
        env: Env,
        coordinator: Address,
        round_id: u64,
        new_model_id: BytesN<32>,
        description: String,
        weights_ref: String,
        metrics_ref: String,
        fairness_report_ref: String,
        global_loss: u32,
        global_accuracy: u32,
        validation_score: u32,
        version: u32,
    ) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::ensure_coordinator(&env, &coordinator)?;

        let config = Self::get_config(&env);

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

        if validation_score < config.validation_threshold {
            return Err(Error::ValidationFailed);
        }

        round.status = RoundStatus::Finalized;
        round.finalized_at = env.ledger().timestamp();
        round.aggregated_model_id = new_model_id.clone();
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);

        let model = ModelMetadata {
            model_id: new_model_id.clone(),
            round_id,
            model_type: round.model_type.clone(),
            description,
            weights_ref,
            metrics_ref,
            fairness_report_ref,
            global_loss,
            global_accuracy,
            num_contributors: round.total_updates,
            validation_score,
            is_rare_disease_model: round.is_rare_disease,
            disease_code: round.disease_code.clone(),
            created_at: round.finalized_at,
            version,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Model(new_model_id.clone()), &model);

        // Distribute rewards; rare disease rounds apply multiplier
        let participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(vec![&env]);

        let reward_multiplier: i128 = if round.is_rare_disease {
            config.rare_disease_reward_multiplier as i128
        } else {
            1
        };

        for inst_addr in participants.iter() {
            let inst_key = DataKey::Institution(inst_addr.clone());
            if let Some(mut inst) = env
                .storage()
                .persistent()
                .get::<DataKey, Institution>(&inst_key)
            {
                inst.reward_balance += round.reward_per_participant * reward_multiplier;
                inst.total_contributions += 1;
                let rep_gain = if validation_score >= 90 {
                    3u32
                } else if validation_score >= 70 {
                    2u32
                } else {
                    1u32
                };
                inst.reputation_score = (inst.reputation_score + rep_gain).min(100);
                env.storage().persistent().set(&inst_key, &inst);
            }
        }

        // Update rare disease registry counters
        if round.is_rare_disease {
            let rd_key = DataKey::RareDiseaseRegistry(round.disease_code.clone());
            if let Some(mut rd) = env
                .storage()
                .persistent()
                .get::<DataKey, RareDiseaseRegistry>(&rd_key)
            {
                rd.model_count += 1;
                rd.participant_count += round.total_updates;
                env.storage().persistent().set(&rd_key, &rd);
            }
        }

        env.events()
            .publish((symbol_short!("RndFin"),), (round_id, new_model_id.clone()));

        Self::emit_audit(
            &env,
            &coordinator,
            String::from_str(&env, "FINALIZE_ROUND"),
            String::from_str(&env, "round"),
            String::from_str(&env, "Round finalized, model aggregated"),
            round_id,
        );

        Ok(true)
    }

    pub fn cancel_expired_round(
        env: Env,
        coordinator: Address,
        round_id: u64,
    ) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::ensure_coordinator(&env, &coordinator)?;

        let mut round: FederatedRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;

        if round.status == RoundStatus::Finalized || round.status == RoundStatus::Cancelled {
            return Err(Error::RoundFinalized);
        }

        let now = env.ledger().timestamp();
        if now <= round.deadline {
            return Err(Error::RoundDeadlineNotPassed);
        }

        round.status = RoundStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);

        env.events().publish((symbol_short!("RndCncl"),), round_id);

        Ok(true)
    }

    // ── Secure MPC ─────────────────────────────────────────────────────────

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

        if round.status == RoundStatus::Finalized || round.status == RoundStatus::Cancelled {
            return Err(Error::RoundFinalized);
        }

        if threshold == 0 || threshold > round.total_updates {
            return Err(Error::InvalidParameter);
        }

        let session_id = Self::next_id_u64(&env, &DataKey::MPCSessionCounter);

        let session = MPCSession {
            session_id,
            round_id,
            threshold,
            total_shares: 0,
            committed_shares: 0,
            is_complete: false,
            result_hash: BytesN::from_array(&env, &[0u8; 32]),
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::MPCSession(session_id), &session);

        env.events()
            .publish((symbol_short!("MPCNew"),), session_id);

        Ok(session_id)
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

        let share = MPCShare {
            session_id,
            institution: institution.clone(),
            share_hash,
            commitment,
            submitted_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&share_key, &share);

        session.total_shares += 1;
        session.committed_shares += 1;

        if session.committed_shares >= session.threshold {
            session.is_complete = true;
            env.events()
                .publish((symbol_short!("MPCDone"),), session_id);
        }

        env.storage()
            .persistent()
            .set(&DataKey::MPCSession(session_id), &session);

        Ok(true)
    }

    pub fn finalize_mpc_session(
        env: Env,
        coordinator: Address,
        session_id: u64,
        result_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::ensure_coordinator(&env, &coordinator)?;

        let mut session: MPCSession = env
            .storage()
            .persistent()
            .get(&DataKey::MPCSession(session_id))
            .ok_or(Error::MPCSessionNotFound)?;

        if session.committed_shares < session.threshold {
            return Err(Error::MPCThresholdNotMet);
        }

        session.result_hash = result_hash;
        session.is_complete = true;
        env.storage()
            .persistent()
            .set(&DataKey::MPCSession(session_id), &session);

        env.events()
            .publish((symbol_short!("MPCFin"),), session_id);

        Ok(true)
    }

    pub fn get_mpc_session(env: Env, session_id: u64) -> Option<MPCSession> {
        env.storage()
            .persistent()
            .get(&DataKey::MPCSession(session_id))
    }

    pub fn get_mpc_share(
        env: Env,
        session_id: u64,
        institution: Address,
    ) -> Option<MPCShare> {
        env.storage()
            .persistent()
            .get(&DataKey::MPCShare(session_id, institution))
    }

    // ── Model Validation & Quality Assessment ─────────────────────────────

    pub fn submit_model_validation(
        env: Env,
        validator: Address,
        model_id: BytesN<32>,
        accuracy_score: u32,
        fairness_score: u32,
        robustness_score: u32,
        bias_score: u32,
        report_ref: String,
    ) -> Result<bool, Error> {
        validator.require_auth();

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Model(model_id.clone()))
        {
            return Err(Error::ModelNotFound);
        }

        if accuracy_score > 100
            || fairness_score > 100
            || robustness_score > 100
            || bias_score > 100
        {
            return Err(Error::InvalidParameter);
        }

        let config = Self::get_config(&env);

        // Weighted average: accuracy 40%, fairness 25%, robustness 20%, bias 15%
        let overall =
            (accuracy_score * 40 + fairness_score * 25 + robustness_score * 20 + bias_score * 15)
                / 100;

        let passed = overall >= config.validation_threshold;

        let validation = ModelValidation {
            model_id: model_id.clone(),
            validator: validator.clone(),
            accuracy_score,
            fairness_score,
            robustness_score,
            bias_score,
            overall_score: overall,
            passed,
            report_ref,
            validated_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::ModelValidation(model_id.clone()), &validation);

        env.events()
            .publish((symbol_short!("ModVal"),), (model_id, passed));

        Self::emit_audit(
            &env,
            &validator,
            String::from_str(&env, "VALIDATE_MODEL"),
            String::from_str(&env, "model"),
            String::from_str(&env, "Model validation submitted"),
            0,
        );

        Ok(passed)
    }

    pub fn get_model_validation(env: Env, model_id: BytesN<32>) -> Option<ModelValidation> {
        env.storage()
            .persistent()
            .get(&DataKey::ModelValidation(model_id))
    }

    // ── Model Marketplace ──────────────────────────────────────────────────

    pub fn list_model(
        env: Env,
        seller: Address,
        model_id: BytesN<32>,
        price: i128,
        description: String,
        license_type: String,
    ) -> Result<u64, Error> {
        seller.require_auth();

        let _inst: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(seller.clone()))
            .ok_or(Error::InstitutionNotFound)?;

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Model(model_id.clone()))
        {
            return Err(Error::ModelNotFound);
        }

        if price < 0 {
            return Err(Error::InvalidParameter);
        }

        let listing_id = Self::next_id_u64(&env, &DataKey::ListingCounter);

        let listing = MarketplaceListing {
            listing_id,
            model_id,
            seller: seller.clone(),
            price,
            description,
            license_type,
            status: MarketplaceListingStatus::Active,
            listed_at: env.ledger().timestamp(),
            sold_at: 0,
            buyer: seller.clone(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::MarketplaceListing(listing_id), &listing);

        env.events()
            .publish((symbol_short!("MktList"),), listing_id);

        Self::emit_audit(
            &env,
            &seller,
            String::from_str(&env, "LIST_MODEL"),
            String::from_str(&env, "marketplace"),
            String::from_str(&env, "Model listed on marketplace"),
            0,
        );

        Ok(listing_id)
    }

    pub fn purchase_model(env: Env, buyer: Address, listing_id: u64) -> Result<bool, Error> {
        buyer.require_auth();

        let mut listing: MarketplaceListing = env
            .storage()
            .persistent()
            .get(&DataKey::MarketplaceListing(listing_id))
            .ok_or(Error::ListingNotFound)?;

        if listing.status != MarketplaceListingStatus::Active {
            return Err(Error::ListingNotActive);
        }

        let mut buyer_inst: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(buyer.clone()))
            .ok_or(Error::InstitutionNotFound)?;

        if buyer_inst.reward_balance < listing.price {
            return Err(Error::InsufficientFunds);
        }

        let config = Self::get_config(&env);
        let fee = listing.price * config.marketplace_fee_bps as i128 / 10000;
        let seller_proceeds = listing.price - fee;

        buyer_inst.reward_balance -= listing.price;
        env.storage()
            .persistent()
            .set(&DataKey::Institution(buyer.clone()), &buyer_inst);

        let seller_key = DataKey::Institution(listing.seller.clone());
        if let Some(mut seller_inst) = env
            .storage()
            .persistent()
            .get::<DataKey, Institution>(&seller_key)
        {
            seller_inst.reward_balance += seller_proceeds;
            env.storage().persistent().set(&seller_key, &seller_inst);
        }

        listing.status = MarketplaceListingStatus::Sold;
        listing.sold_at = env.ledger().timestamp();
        listing.buyer = buyer.clone();
        env.storage()
            .persistent()
            .set(&DataKey::MarketplaceListing(listing_id), &listing);

        env.events()
            .publish((symbol_short!("MktSold"),), (listing_id, buyer.clone()));

        Self::emit_audit(
            &env,
            &buyer,
            String::from_str(&env, "PURCHASE_MODEL"),
            String::from_str(&env, "marketplace"),
            String::from_str(&env, "Model purchased from marketplace"),
            0,
        );

        Ok(true)
    }

    pub fn withdraw_listing(
        env: Env,
        seller: Address,
        listing_id: u64,
    ) -> Result<bool, Error> {
        seller.require_auth();

        let mut listing: MarketplaceListing = env
            .storage()
            .persistent()
            .get(&DataKey::MarketplaceListing(listing_id))
            .ok_or(Error::ListingNotFound)?;

        if listing.seller != seller {
            return Err(Error::NotAuthorized);
        }

        if listing.status != MarketplaceListingStatus::Active {
            return Err(Error::ListingNotActive);
        }

        listing.status = MarketplaceListingStatus::Withdrawn;
        env.storage()
            .persistent()
            .set(&DataKey::MarketplaceListing(listing_id), &listing);

        Ok(true)
    }

    pub fn get_listing(env: Env, listing_id: u64) -> Option<MarketplaceListing> {
        env.storage()
            .persistent()
            .get(&DataKey::MarketplaceListing(listing_id))
    }

    // ── Incentive / Reward Withdrawal ──────────────────────────────────────

    pub fn withdraw_rewards(
        env: Env,
        institution: Address,
        amount: i128,
    ) -> Result<i128, Error> {
        institution.require_auth();

        let mut inst: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(institution.clone()))
            .ok_or(Error::InstitutionNotFound)?;

        if amount <= 0 || inst.reward_balance < amount {
            return Err(Error::InsufficientFunds);
        }

        inst.reward_balance -= amount;
        env.storage()
            .persistent()
            .set(&DataKey::Institution(institution.clone()), &inst);

        env.events()
            .publish((symbol_short!("Withdraw"),), (institution, amount));

        Ok(inst.reward_balance)
    }

    // ── Config Management ──────────────────────────────────────────────────

    pub fn update_config(
        env: Env,
        admin: Address,
        config: PlatformConfig,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::ensure_admin(&env, &admin)?;

        env.storage().instance().set(&DataKey::Config, &config);

        Self::emit_audit(
            &env,
            &admin,
            String::from_str(&env, "UPDATE_CONFIG"),
            String::from_str(&env, "platform"),
            String::from_str(&env, "Platform config updated"),
            0,
        );

        Ok(true)
    }

    pub fn get_config_view(env: Env) -> PlatformConfig {
        Self::get_config(&env)
    }

    // ── Read Helpers ───────────────────────────────────────────────────────

    pub fn get_round(env: Env, round_id: u64) -> Option<FederatedRound> {
        env.storage().persistent().get(&DataKey::Round(round_id))
    }

    pub fn get_model(env: Env, model_id: BytesN<32>) -> Option<ModelMetadata> {
        env.storage().persistent().get(&DataKey::Model(model_id))
    }

    pub fn get_participant_update(
        env: Env,
        round_id: u64,
        institution: Address,
    ) -> Option<ParticipantUpdate> {
        env.storage()
            .persistent()
            .get(&DataKey::ParticipantUpdate(round_id, institution))
    }

    pub fn get_round_participants(env: Env, round_id: u64) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(vec![&env])
    }

    pub fn get_audit_entry(env: Env, entry_id: u64) -> Option<AuditEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::AuditEntry(entry_id))
    }

    pub fn get_round_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RoundCounter)
            .unwrap_or(0)
    }

    pub fn get_institution_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::InstitutionCount)
            .unwrap_or(0)
    }

    pub fn get_audit_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AuditCounter)
            .unwrap_or(0)
    }

    pub fn get_listing_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ListingCounter)
            .unwrap_or(0)
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn default_config(env: &Env) -> PlatformConfig {
        PlatformConfig {
            min_reputation_to_participate: 10,
            base_reward_amount: 100,
            rare_disease_reward_multiplier: 3,
            validation_threshold: 60,
            max_epsilon_per_round: 200,
            min_data_quality_score: 50,
            marketplace_fee_bps: 200,
            mpc_default_threshold: 3,
        }
    }

    fn setup(env: &Env) -> (FederatedLearningContractClient, Address, Address) {
        let contract_id = env.register_contract(None, FederatedLearningContract);
        let client = FederatedLearningContractClient::new(env, &contract_id);
        let admin = Address::generate(env);
        let coordinator = Address::generate(env);
        client
            .mock_all_auths()
            .initialize(&admin, &coordinator, &default_config(env));
        (client, admin, coordinator)
    }

    fn register_inst(
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

    fn start_round_default(
        client: &FederatedLearningContractClient,
        env: &Env,
        admin: &Address,
        min_p: u32,
        reward: i128,
    ) -> u64 {
        client.mock_all_auths().start_round(
            admin,
            &BytesN::from_array(env, &[1u8; 32]),
            &ModelType::CNN,
            &min_p,
            &10u32,
            &10u32,
            &5u32,
            &100u32,
            &50u32,
            &3u32,
            &false,
            &String::from_str(env, ""),
            &reward,
            &86400u64,
        )
    }

    fn submit_default(
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
            &200u32,
            &80u32,
            &BytesN::from_array(env, &[5u8; 32]),
            &true,
            &true,
        );
    }

    #[test]
    fn test_full_round_lifecycle() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        let inst1 = register_inst(&client, &env, &admin);
        let inst2 = register_inst(&client, &env, &admin);

        let round_id = start_round_default(&client, &env, &admin, 2, 50);

        submit_default(&client, &env, &inst1, round_id, 2);
        submit_default(&client, &env, &inst2, round_id, 3);

        assert!(client.mock_all_auths().begin_aggregation(&coordinator, &round_id));

        let new_model = BytesN::from_array(&env, &[4u8; 32]);
        assert!(client.mock_all_auths().finalize_round(
            &coordinator,
            &round_id,
            &new_model,
            &String::from_str(&env, "FL model v1"),
            &String::from_str(&env, "ipfs://weights"),
            &String::from_str(&env, "ipfs://metrics"),
            &String::from_str(&env, "ipfs://fairness"),
            &200u32,
            &87u32,
            &80u32,
            &1u32,
        ));

        let r = client.get_round(&round_id).unwrap();
        assert_eq!(r.status, RoundStatus::Finalized);

        let m = client.get_model(&new_model).unwrap();
        assert_eq!(m.num_contributors, 2);
        assert_eq!(m.global_accuracy, 87);

        let i1 = client.get_institution(&inst1).unwrap();
        assert!(i1.reward_balance >= 50);
    }

    #[test]
    fn test_dp_budget_enforcement() {
        let env = Env::default();
        let (client, admin, _) = setup(&env);
        let inst = register_inst(&client, &env, &admin);

        // Tiny budget: 3 epsilon, 5 delta
        assert!(client
            .mock_all_auths()
            .set_privacy_budget(&admin, &inst, &3u32, &5u32));

        // Each submission of 1000 samples with no noise costs 1+2=3 epsilon
        // First round uses 3 epsilon => exhausted
        let r1 = start_round_default(&client, &env, &admin, 1, 10);
        let res1 = client.mock_all_auths().try_submit_update(
            &inst,
            &r1,
            &BytesN::from_array(&env, &[2u8; 32]),
            &String::from_str(&env, "ipfs://enc"),
            &1000u32,
            &100u32,
            &80u32,
            &BytesN::from_array(&env, &[5u8; 32]),
            &false,
            &true,
        );
        assert!(res1.unwrap().is_ok());

        // Second round should be rejected (budget exhausted)
        let r2 = start_round_default(&client, &env, &admin, 1, 10);
        let res2 = client.mock_all_auths().try_submit_update(
            &inst,
            &r2,
            &BytesN::from_array(&env, &[3u8; 32]),
            &String::from_str(&env, "ipfs://enc"),
            &1000u32,
            &100u32,
            &80u32,
            &BytesN::from_array(&env, &[5u8; 32]),
            &false,
            &true,
        );
        assert!(res2.unwrap().is_err());
    }

    #[test]
    fn test_mpc_session() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        let inst1 = register_inst(&client, &env, &admin);
        let inst2 = register_inst(&client, &env, &admin);
        let inst3 = register_inst(&client, &env, &admin);

        let round_id = start_round_default(&client, &env, &admin, 3, 50);

        submit_default(&client, &env, &inst1, round_id, 2);
        submit_default(&client, &env, &inst2, round_id, 3);
        submit_default(&client, &env, &inst3, round_id, 4);

        let session_id = client
            .mock_all_auths()
            .create_mpc_session(&coordinator, &round_id, &2u32);

        let sh = BytesN::from_array(&env, &[7u8; 32]);
        let co = BytesN::from_array(&env, &[8u8; 32]);

        assert!(client.mock_all_auths().submit_mpc_share(&inst1, &session_id, &sh, &co));
        assert!(client.mock_all_auths().submit_mpc_share(&inst2, &session_id, &sh, &co));

        let session = client.get_mpc_session(&session_id).unwrap();
        assert!(session.is_complete);
    }

    #[test]
    fn test_rare_disease_round() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        client.mock_all_auths().register_rare_disease(
            &admin,
            &String::from_str(&env, "ORPHA:99999"),
            &String::from_str(&env, "Test rare disease"),
        );

        let inst = register_inst(&client, &env, &admin);

        let round_id = client.mock_all_auths().start_round(
            &admin,
            &BytesN::from_array(&env, &[1u8; 32]),
            &ModelType::RNN,
            &1u32,
            &10u32,
            &10u32,
            &5u32,
            &100u32,
            &50u32,
            &1u32,
            &true,
            &String::from_str(&env, "ORPHA:99999"),
            &50i128,
            &86400u64,
        );

        submit_default(&client, &env, &inst, round_id, 2);

        client.mock_all_auths().begin_aggregation(&coordinator, &round_id);

        let model_id = BytesN::from_array(&env, &[4u8; 32]);
        client.mock_all_auths().finalize_round(
            &coordinator,
            &round_id,
            &model_id,
            &String::from_str(&env, "Rare disease model"),
            &String::from_str(&env, "ipfs://w"),
            &String::from_str(&env, "ipfs://m"),
            &String::from_str(&env, "ipfs://f"),
            &150u32,
            &90u32,
            &85u32,
            &1u32,
        );

        // 50 reward * 3 (rare disease multiplier) = 150
        let inst_data = client.get_institution(&inst).unwrap();
        assert_eq!(inst_data.reward_balance, 150);

        let rd = client
            .get_rare_disease(&String::from_str(&env, "ORPHA:99999"))
            .unwrap();
        assert_eq!(rd.model_count, 1);
        assert_eq!(rd.participant_count, 1);
    }

    #[test]
    fn test_marketplace_flow() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        let seller = register_inst(&client, &env, &admin);
        let buyer = register_inst(&client, &env, &admin);

        // Seller contributes to a round to earn rewards
        let r1 = start_round_default(&client, &env, &admin, 1, 500);
        submit_default(&client, &env, &seller, r1, 2);
        client.mock_all_auths().begin_aggregation(&coordinator, &r1);
        let model_id = BytesN::from_array(&env, &[4u8; 32]);
        client.mock_all_auths().finalize_round(
            &coordinator, &r1, &model_id,
            &String::from_str(&env, "v1"),
            &String::from_str(&env, "ipfs://w"),
            &String::from_str(&env, "ipfs://m"),
            &String::from_str(&env, "ipfs://f"),
            &100u32, &88u32, &80u32, &1u32,
        );

        // Buyer earns rewards from a separate round
        let r2 = start_round_default(&client, &env, &admin, 1, 300);
        submit_default(&client, &env, &buyer, r2, 3);
        client.mock_all_auths().begin_aggregation(&coordinator, &r2);
        let model_id2 = BytesN::from_array(&env, &[6u8; 32]);
        client.mock_all_auths().finalize_round(
            &coordinator, &r2, &model_id2,
            &String::from_str(&env, "v2"),
            &String::from_str(&env, "ipfs://w2"),
            &String::from_str(&env, "ipfs://m2"),
            &String::from_str(&env, "ipfs://f2"),
            &100u32, &88u32, &80u32, &1u32,
        );

        let listing_id = client.mock_all_auths().list_model(
            &seller,
            &model_id,
            &200i128,
            &String::from_str(&env, "CNN cardiac model"),
            &String::from_str(&env, "CC-BY-NC"),
        );

        let buyer_before = client.get_institution(&buyer).unwrap().reward_balance;
        assert!(buyer_before >= 200);

        assert!(client.mock_all_auths().purchase_model(&buyer, &listing_id));

        let listing = client.get_listing(&listing_id).unwrap();
        assert_eq!(listing.status, MarketplaceListingStatus::Sold);

        let buyer_after = client.get_institution(&buyer).unwrap().reward_balance;
        assert_eq!(buyer_after, buyer_before - 200);
    }

    #[test]
    fn test_model_validation() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        let inst = register_inst(&client, &env, &admin);
        let validator = Address::generate(&env);

        let round_id = start_round_default(&client, &env, &admin, 1, 50);
        submit_default(&client, &env, &inst, round_id, 2);
        client.mock_all_auths().begin_aggregation(&coordinator, &round_id);

        let model_id = BytesN::from_array(&env, &[4u8; 32]);
        client.mock_all_auths().finalize_round(
            &coordinator, &round_id, &model_id,
            &String::from_str(&env, "GNN model"),
            &String::from_str(&env, "ipfs://w"),
            &String::from_str(&env, "ipfs://m"),
            &String::from_str(&env, "ipfs://f"),
            &100u32, &90u32, &80u32, &1u32,
        );

        let passed = client.mock_all_auths().submit_model_validation(
            &validator,
            &model_id,
            &90u32,
            &85u32,
            &80u32,
            &88u32,
            &String::from_str(&env, "ipfs://val-report"),
        );

        assert!(passed);

        let val = client.get_model_validation(&model_id).unwrap();
        assert!(val.passed);
        assert!(val.overall_score >= 60);
    }

    #[test]
    fn test_audit_trail() {
        let env = Env::default();
        let (client, admin, _) = setup(&env);

        let count_before = client.get_audit_count();
        register_inst(&client, &env, &admin);
        let count_after = client.get_audit_count();

        assert!(count_after > count_before);

        let entry = client.get_audit_entry(&1u64);
        assert!(entry.is_some());
    }

    #[test]
    fn test_institution_reputation_grows() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        let inst = register_inst(&client, &env, &admin);
        let initial = client.get_institution(&inst).unwrap().reputation_score;

        let round_id = start_round_default(&client, &env, &admin, 1, 50);
        submit_default(&client, &env, &inst, round_id, 2);
        client.mock_all_auths().begin_aggregation(&coordinator, &round_id);

        let model_id = BytesN::from_array(&env, &[4u8; 32]);
        client.mock_all_auths().finalize_round(
            &coordinator, &round_id, &model_id,
            &String::from_str(&env, "v1"),
            &String::from_str(&env, "ipfs://w"),
            &String::from_str(&env, "ipfs://m"),
            &String::from_str(&env, "ipfs://f"),
            &100u32, &90u32, &95u32, &1u32,
        );

        let after = client.get_institution(&inst).unwrap().reputation_score;
        assert!(after > initial);
    }

    #[test]
    fn test_withdraw_rewards() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        let inst = register_inst(&client, &env, &admin);

        let round_id = start_round_default(&client, &env, &admin, 1, 200);
        submit_default(&client, &env, &inst, round_id, 2);
        client.mock_all_auths().begin_aggregation(&coordinator, &round_id);

        let model_id = BytesN::from_array(&env, &[4u8; 32]);
        client.mock_all_auths().finalize_round(
            &coordinator, &round_id, &model_id,
            &String::from_str(&env, "v1"),
            &String::from_str(&env, "ipfs://w"),
            &String::from_str(&env, "ipfs://m"),
            &String::from_str(&env, "ipfs://f"),
            &100u32, &88u32, &80u32, &1u32,
        );

        let balance_before = client.get_institution(&inst).unwrap().reward_balance;
        assert!(balance_before >= 100);

        let remaining = client
            .mock_all_auths()
            .withdraw_rewards(&inst, &100i128);

        assert_eq!(remaining, balance_before - 100);
    }

    #[test]
    fn test_duplicate_update_rejected() {
        let env = Env::default();
        let (client, admin, _) = setup(&env);
        let inst = register_inst(&client, &env, &admin);

        let round_id = start_round_default(&client, &env, &admin, 1, 50);
        submit_default(&client, &env, &inst, round_id, 2);

        let res = client.mock_all_auths().try_submit_update(
            &inst,
            &round_id,
            &BytesN::from_array(&env, &[2u8; 32]),
            &String::from_str(&env, "ipfs://enc"),
            &500u32, &200u32, &80u32,
            &BytesN::from_array(&env, &[5u8; 32]),
            &true, &true,
        );

        assert!(res.unwrap().is_err());
    }

    #[test]
    fn test_model_types_supported() {
        let env = Env::default();
        let (client, admin, coordinator) = setup(&env);

        let model_types = [
            ModelType::CNN,
            ModelType::RNN,
            ModelType::Transformer,
            ModelType::FeedForward,
            ModelType::GNN,
            ModelType::Hybrid,
        ];

        for (i, mt) in model_types.iter().enumerate() {
            let inst = register_inst(&client, &env, &admin);
            let round_id = client.mock_all_auths().start_round(
                &admin,
                &BytesN::from_array(&env, &[1u8; 32]),
                mt,
                &1u32, &5u32, &10u32, &5u32,
                &100u32, &50u32, &1u32,
                &false, &String::from_str(&env, ""),
                &50i128, &86400u64,
            );

            submit_default(&client, &env, &inst, round_id, (i + 2) as u8);
            client.mock_all_auths().begin_aggregation(&coordinator, &round_id);

            let model_id = BytesN::from_array(&env, &[(i + 10) as u8; 32]);
            client.mock_all_auths().finalize_round(
                &coordinator, &round_id, &model_id,
                &String::from_str(&env, "model"),
                &String::from_str(&env, "ipfs://w"),
                &String::from_str(&env, "ipfs://m"),
                &String::from_str(&env, "ipfs://f"),
                &100u32, &85u32, &80u32, &1u32,
            );

            let m = client.get_model(&model_id).unwrap();
            assert_eq!(m.model_type, *mt);
        }
    }
}
