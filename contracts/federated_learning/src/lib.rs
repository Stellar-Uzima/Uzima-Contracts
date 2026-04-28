#![no_std]
#![allow(clippy::arithmetic_side_effects, clippy::panic, clippy::unwrap_used)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    Map, String, Vec,
};

// Removed unused 'max' to prevent compiler warnings
use core::cmp::min;

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
pub enum Framework {
    TensorFlow,
    PyTorch,
    JAX,
    Custom,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum AggregationMethod {
    FedAvg,
    FedProx,
    SecureAgg,
    Krum,
    MultiKrum,
    TrimmedMean,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum RoundStatus {
    Open,
    Aggregating,
    Finalized,
    Failed,
    Verification,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum InstitutionStatus {
    Active,
    Suspended,
    Blacklisted,
    UnderReview,
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
    pub last_contribution: u64,
    pub privacy_budget_used: u32,
    pub contribution_quality_score: u32,
    pub framework_preference: Framework,
}

#[derive(Clone)]
#[contracttype]
pub struct RoundConfig {
    pub model_type: ModelType,
    pub framework: Framework,
    pub aggregation_method: AggregationMethod,
    pub min_participants: u32,
    pub max_participants: u32,
    pub dp_epsilon: u32,
    pub dp_delta: u32,
    pub reward_per_participant: i128,
    pub duration_seconds: u64,
    pub verification_threshold: u32,
    pub poisoning_detection_threshold: u32,
    pub communication_budget: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct FederatedRound {
    pub id: u64,
    pub base_model_id: BytesN<32>,
    pub model_type: ModelType,
    pub framework: Framework,
    pub aggregation_method: AggregationMethod,
    pub min_participants: u32,
    pub max_participants: u32,
    pub reward_per_participant: i128,
    pub total_updates: u32,
    pub status: RoundStatus,
    pub started_at: u64,
    pub deadline: u64,
    pub finalized_at: u64,
    pub aggregated_model_id: BytesN<32>,
    pub verification_score: u32,
    pub poisoning_detected: bool,
    pub communication_overhead: u32,
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
    pub convergence_metrics: Map<String, u32>,
    pub privacy_loss: u32,
    pub communication_cost: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ModelMetadata {
    pub model_id: BytesN<32>,
    pub round_id: u64,
    pub model_type: ModelType,
    pub framework: Framework,
    pub num_contributors: u32,
    pub validation_score: u32,
    pub version: u32,
    pub aggregation_method: AggregationMethod,
    pub privacy_guarantee: u32,
    pub robustness_score: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ContributionVerification {
    pub institution: Address,
    pub round_id: u64,
    pub gradient_hash: BytesN<32>,
    pub quality_score: u32,
    pub similarity_score: u32,
    pub privacy_compliance: bool,
    pub anomaly_detected: bool,
    pub contribution_weight: u32,
    pub verification_timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PrivacyMetrics {
    pub epsilon_used: u32,
    pub delta_used: u32,
    pub noise_scale: u32,
    pub clipping_bound: u32,
    pub privacy_budget_remaining: u32,
    pub cumulative_privacy_loss: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct AttackDetection {
    pub round_id: u64,
    pub detected_attacks: Vec<String>,
    pub suspicious_participants: Vec<Address>,
    pub attack_confidence: u32,
    pub mitigation_applied: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct CommunicationMetrics {
    pub total_bytes_sent: u32,
    pub total_bytes_received: u32,
    pub compression_ratio: u32,
    pub latency_ms: u32,
    pub protocol_efficiency: u32,
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
    ContributionVerification(u64, Address),
    PrivacyMetrics(u64),
    AttackDetection(u64),
    CommunicationMetrics(u64),
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
    PrivacyBudgetExceeded = 18,
    PoisoningAttackDetected = 19,
    CommunicationBudgetExceeded = 20,
    VerificationFailed = 21,
    FrameworkNotSupported = 22,
    ContributionQualityLow = 23,
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
        framework_preference: Framework,
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
                last_contribution: 0,
                privacy_budget_used: 0,
                contribution_quality_score: 75,
                framework_preference,
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
        if cfg.max_participants > 100 {
            return Err(Error::InvalidParameter);
        }

        let id = Self::next_round_id(&env);
        let now = env.ledger().timestamp();

        env.storage().persistent().set(
            &DataKey::Round(id),
            &FederatedRound {
                id,
                base_model_id,
                model_type: cfg.model_type.clone(),
                framework: cfg.framework.clone(),
                aggregation_method: cfg.aggregation_method.clone(),
                min_participants: cfg.min_participants,
                max_participants: cfg.max_participants,
                reward_per_participant: cfg.reward_per_participant,
                total_updates: 0,
                status: RoundStatus::Open,
                started_at: now,
                deadline: now + cfg.duration_seconds,
                finalized_at: 0,
                aggregated_model_id: BytesN::from_array(&env, &[0u8; 32]),
                verification_score: 0,
                poisoning_detected: false,
                communication_overhead: 0,
            },
        );

        let empty: Vec<Address> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::RoundParticipants(id), &empty);

        env.storage().persistent().set(
            &DataKey::PrivacyMetrics(id),
            &PrivacyMetrics {
                epsilon_used: 0,
                delta_used: 0,
                noise_scale: cfg.dp_epsilon,
                clipping_bound: cfg.dp_delta,
                privacy_budget_remaining: cfg.dp_epsilon * cfg.max_participants,
                cumulative_privacy_loss: 0,
            },
        );

        env.storage().persistent().set(
            &DataKey::CommunicationMetrics(id),
            &CommunicationMetrics {
                total_bytes_sent: 0,
                total_bytes_received: 0,
                compression_ratio: 100,
                latency_ms: 0,
                protocol_efficiency: 100,
            },
        );

        env.events().publish((symbol_short!("RndStart"),), id);
        Ok(id)
    }

    pub fn submit_update(
        env: Env,
        institution: Address,
        round_id: u64,
        gradient_hash: BytesN<32>,
        quality_metrics: Map<String, u32>,
        privacy_proof: BytesN<32>,
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

        let mut privacy_metrics: PrivacyMetrics = env
            .storage()
            .persistent()
            .get(&DataKey::PrivacyMetrics(round_id))
            .unwrap_or(PrivacyMetrics {
                epsilon_used: 0,
                delta_used: 0,
                noise_scale: 1,
                clipping_bound: 1,
                privacy_budget_remaining: 100,
                cumulative_privacy_loss: 0,
            });

        if inst.privacy_budget_used >= privacy_metrics.privacy_budget_remaining {
            return Err(Error::PrivacyBudgetExceeded);
        }

        let quality_score = Self::evaluate_contribution_quality(&env, &quality_metrics);
        let similarity_score = Self::compute_similarity_score(&gradient_hash, &round.base_model_id);
        let anomaly_detected = Self::detect_anomaly(&env, &quality_metrics, similarity_score);

        if anomaly_detected && inst.reputation_score < 30 {
            inst.status = InstitutionStatus::UnderReview;
            env.storage().persistent().set(&inst_key, &inst);
            return Err(Error::VerificationFailed);
        }

        let verification = ContributionVerification {
            institution: institution.clone(),
            round_id,
            gradient_hash: gradient_hash.clone(),
            quality_score,
            similarity_score,
            privacy_compliance: true,
            anomaly_detected,
            contribution_weight: Self::calculate_contribution_weight(
                quality_score,
                inst.reputation_score,
            ),
            verification_timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&upd_key, &gradient_hash);
        env.storage().persistent().set(
            &DataKey::ContributionVerification(round_id, institution.clone()),
            &verification,
        );

        let mut participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(Vec::new(&env));
        participants.push_back(institution.clone());
        env.storage()
            .persistent()
            .set(&DataKey::RoundParticipants(round_id), &participants);

        round.total_updates += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);

        inst.total_contributions += 1;
        inst.last_contribution = env.ledger().timestamp();
        inst.contribution_quality_score = (inst.contribution_quality_score * 3 + quality_score) / 4;
        inst.privacy_budget_used += privacy_metrics.noise_scale;
        env.storage().persistent().set(&inst_key, &inst);

        privacy_metrics.epsilon_used += privacy_metrics.noise_scale;
        privacy_metrics.privacy_budget_remaining -= privacy_metrics.noise_scale;
        privacy_metrics.cumulative_privacy_loss += privacy_metrics.noise_scale;
        env.storage()
            .persistent()
            .set(&DataKey::PrivacyMetrics(round_id), &privacy_metrics);

        env.events().publish(
            (symbol_short!("UpdSub"),),
            (round_id, institution, quality_score),
        );
        Ok(true)
    }

    fn evaluate_contribution_quality(env: &Env, metrics: &Map<String, u32>) -> u32 {
        let mut score = 50u32;

        if let Some(loss) = metrics.get(String::from_str(env, "loss")) {
            if loss < 5 {
                score += 20;
            } else if loss < 10 {
                score += 10;
            } else if loss > 50 {
                score -= 20;
            }
        }

        if let Some(accuracy) = metrics.get(String::from_str(env, "accuracy")) {
            score += accuracy / 5;
        }

        if let Some(convergence) = metrics.get(String::from_str(env, "convergence")) {
            if convergence > 80 {
                score += 15;
            } else if convergence > 60 {
                score += 10;
            }
        }

        min(score, 100)
    }

    fn compute_similarity_score(gradient: &BytesN<32>, base_model: &BytesN<32>) -> u32 {
        let mut similarity = 50u32;
        for i in 0..32 {
            if gradient[i] == base_model[i] {
                similarity += 1;
            }
        }
        min(similarity, 100)
    }

    fn detect_anomaly(env: &Env, metrics: &Map<String, u32>, similarity: u32) -> bool {
        if let Some(loss) = metrics.get(String::from_str(env, "loss")) {
            if loss > 100 {
                return true;
            }
        }

        if similarity < 20 {
            return true;
        }

        false
    }

    fn calculate_contribution_weight(quality: u32, reputation: u32) -> u32 {
        ((quality as u64 * reputation as u64) / 10000) as u32
    }

    fn detect_poisoning_attacks(
        env: &Env,
        round_id: u64,
        participants: &Vec<Address>,
    ) -> Result<AttackDetection, Error> {
        let mut suspicious_participants: Vec<Address> = Vec::new(env);
        let mut detected_attacks: Vec<String> = Vec::new(env);
        let mut total_anomaly_score = 0u32;
        let mut participant_count = 0u32;

        for addr in participants.iter() {
            if let Some(verification) = env
                .storage()
                .persistent()
                .get::<DataKey, ContributionVerification>(&DataKey::ContributionVerification(
                    round_id,
                    addr.clone(),
                ))
            {
                participant_count += 1;

                if verification.anomaly_detected {
                    suspicious_participants.push_back(addr.clone());
                    total_anomaly_score += 1;
                }

                if verification.quality_score < 25 {
                    detected_attacks.push_back(String::from_str(env, "low_quality_attack"));
                    suspicious_participants.push_back(addr.clone());
                }

                if verification.similarity_score < 15 {
                    detected_attacks.push_back(String::from_str(env, "divergent_model_attack"));
                    suspicious_participants.push_back(addr.clone());
                }
            }
        }

        let attack_confidence = if participant_count > 0 {
            (total_anomaly_score * 100) / participant_count
        } else {
            0
        };

        Ok(AttackDetection {
            round_id,
            detected_attacks,
            suspicious_participants,
            attack_confidence,
            mitigation_applied: false,
        })
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

        let participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(Vec::new(&env));

        let attack_detection = Self::detect_poisoning_attacks(&env, round_id, &participants)?;

        if attack_detection.attack_confidence > 30 {
            round.status = RoundStatus::Failed;
            round.poisoning_detected = true;
            env.storage()
                .persistent()
                .set(&DataKey::Round(round_id), &round);
            env.storage()
                .persistent()
                .set(&DataKey::AttackDetection(round_id), &attack_detection);
            return Err(Error::PoisoningAttackDetected);
        }

        round.status = RoundStatus::Verification;
        round.verification_score = 100 - attack_detection.attack_confidence;
        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);
        env.storage()
            .persistent()
            .set(&DataKey::AttackDetection(round_id), &attack_detection);

        env.events().publish(
            (symbol_short!("AggStart"),),
            (round_id, round.verification_score),
        );
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
        if round.status != RoundStatus::Verification {
            return Err(Error::RoundNotAggregating);
        }

        if out.validation_score < 60 {
            return Err(Error::ValidationFailed);
        }

        let accuracy_diff = if out.global_accuracy > 80 {
            0
        } else {
            80 - out.global_accuracy
        };
        if accuracy_diff > 5 {
            return Err(Error::ValidationFailed);
        }

        let mut comm_metrics: CommunicationMetrics = env
            .storage()
            .persistent()
            .get(&DataKey::CommunicationMetrics(round_id))
            .unwrap_or(CommunicationMetrics {
                total_bytes_sent: 0,
                total_bytes_received: 0,
                compression_ratio: 100,
                latency_ms: 0,
                protocol_efficiency: 100,
            });

        let expected_comm_cost = round.total_updates * 1024;
        if comm_metrics.total_bytes_sent > expected_comm_cost * 3 {
            return Err(Error::CommunicationBudgetExceeded);
        }

        let vscore = out.validation_score;
        let vid = out.model_id.clone();
        round.status = RoundStatus::Finalized;
        round.finalized_at = env.ledger().timestamp();
        round.aggregated_model_id = vid.clone();
        round.communication_overhead = comm_metrics.total_bytes_sent;

        env.storage()
            .persistent()
            .set(&DataKey::Round(round_id), &round);

        let privacy_metrics: PrivacyMetrics = env
            .storage()
            .persistent()
            .get(&DataKey::PrivacyMetrics(round_id))
            .unwrap_or(PrivacyMetrics {
                epsilon_used: 0,
                delta_used: 0,
                noise_scale: 1,
                clipping_bound: 1,
                privacy_budget_remaining: 100,
                cumulative_privacy_loss: 0,
            });

        env.storage().persistent().set(
            &DataKey::Model(vid.clone()),
            &ModelMetadata {
                model_id: vid.clone(),
                round_id,
                model_type: round.model_type.clone(),
                framework: round.framework.clone(),
                num_contributors: round.total_updates,
                validation_score: vscore,
                version: out.version,
                aggregation_method: round.aggregation_method.clone(),
                privacy_guarantee: privacy_metrics.cumulative_privacy_loss,
                robustness_score: round.verification_score,
            },
        );

        let participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RoundParticipants(round_id))
            .unwrap_or(Vec::new(&env));

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

                if let Some(verification) =
                    env.storage()
                        .persistent()
                        .get::<DataKey, ContributionVerification>(
                            &DataKey::ContributionVerification(round_id, addr.clone()),
                        )
                {
                    if verification.quality_score > 80 {
                        inst.reputation_score = (inst.reputation_score + 1).min(100);
                    }
                }

                env.storage().persistent().set(&k, &inst);
            }
        }

        env.events().publish(
            (symbol_short!("RndFin"),),
            (round_id, vid, vscore, round.communication_overhead),
        );
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

    pub fn get_privacy_metrics(env: Env, round_id: u64) -> Option<PrivacyMetrics> {
        env.storage()
            .persistent()
            .get(&DataKey::PrivacyMetrics(round_id))
    }

    pub fn get_attack_detection(env: Env, round_id: u64) -> Option<AttackDetection> {
        env.storage()
            .persistent()
            .get(&DataKey::AttackDetection(round_id))
    }

    pub fn get_communication_metrics(env: Env, round_id: u64) -> Option<CommunicationMetrics> {
        env.storage()
            .persistent()
            .get(&DataKey::CommunicationMetrics(round_id))
    }

    pub fn get_contribution_verification(
        env: Env,
        round_id: u64,
        institution: Address,
    ) -> Option<ContributionVerification> {
        env.storage()
            .persistent()
            .get(&DataKey::ContributionVerification(round_id, institution))
    }

    pub fn update_communication_metrics(
        env: Env,
        coordinator: Address,
        round_id: u64,
        metrics: CommunicationMetrics,
    ) -> Result<bool, Error> {
        coordinator.require_auth();
        Self::check_auth(&env, &coordinator, &DataKey::Coordinator)?;

        let round: FederatedRound = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .ok_or(Error::RoundNotFound)?;

        if round.status != RoundStatus::Open && round.status != RoundStatus::Verification {
            return Err(Error::RoundNotOpen);
        }

        env.storage()
            .persistent()
            .set(&DataKey::CommunicationMetrics(round_id), &metrics);
        Ok(true)
    }

    pub fn blacklist_institution(
        env: Env,
        admin: Address,
        institution: Address,
        reason: String,
    ) -> Result<bool, Error> {
        admin.require_auth();
        Self::check_auth(&env, &admin, &DataKey::Admin)?;

        let mut inst: Institution = env
            .storage()
            .persistent()
            .get(&DataKey::Institution(institution.clone()))
            .ok_or(Error::InstitutionNotFound)?;

        inst.status = InstitutionStatus::Blacklisted;
        env.storage()
            .persistent()
            .set(&DataKey::Institution(institution.clone()), &inst);

        env.events()
            .publish((symbol_short!("InstBlack"),), (institution, reason));
        Ok(true)
    }
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup(env: &Env) -> (FederatedLearningContractClient<'_>, Address, Address) {
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
            &Framework::TensorFlow,
        );
        inst
    }

    fn default_cfg(_env: &Env, min_p: u32, reward: i128) -> RoundConfig {
        RoundConfig {
            model_type: ModelType::CNN,
            framework: Framework::TensorFlow,
            aggregation_method: AggregationMethod::FedAvg,
            min_participants: min_p,
            max_participants: 10,
            dp_epsilon: 10,
            dp_delta: 5,
            reward_per_participant: reward,
            duration_seconds: 86400,
            verification_threshold: 80,
            poisoning_detection_threshold: 30,
            communication_budget: 3072,
        }
    }

    #[test]
    fn test_enhanced_round_lifecycle() {
        let env = Env::default();
        let (client, admin, coord) = setup(&env);
        let inst1 = add_inst(&client, &env, &admin);
        let inst2 = add_inst(&client, &env, &admin);

        let round_id = client.mock_all_auths().start_round(
            &admin,
            &BytesN::from_array(&env, &[1u8; 32]),
            &default_cfg(&env, 2, 50),
        );

        let mut quality_metrics: Map<String, u32> = Map::new(&env);
        quality_metrics.set(String::from_str(&env, "loss"), &5u32);
        quality_metrics.set(String::from_str(&env, "accuracy"), &85u32);
        quality_metrics.set(String::from_str(&env, "convergence"), &90u32);

        client.mock_all_auths().submit_update(
            &inst1,
            &round_id,
            &BytesN::from_array(&env, &[2u8; 32]),
            &quality_metrics,
            &BytesN::from_array(&env, &[7u8; 32]),
        );

        let mut quality_metrics2: Map<String, u32> = Map::new(&env);
        quality_metrics2.set(String::from_str(&env, "loss"), &8u32);
        quality_metrics2.set(String::from_str(&env, "accuracy"), &82u32);
        quality_metrics2.set(String::from_str(&env, "convergence"), &85u32);

        client.mock_all_auths().submit_update(
            &inst2,
            &round_id,
            &BytesN::from_array(&env, &[3u8; 32]),
            &quality_metrics2,
            &BytesN::from_array(&env, &[8u8; 32]),
        );

        assert!(client.mock_all_auths().begin_aggregation(&coord, &round_id));

        let mut convergence_metrics: Map<String, u32> = Map::new(&env);
        convergence_metrics.set(String::from_str(&env, "training_loss"), &3u32);
        convergence_metrics.set(String::from_str(&env, "validation_loss"), &4u32);

        let mid = BytesN::from_array(&env, &[4u8; 32]);
        client.mock_all_auths().finalize_round(
            &coord,
            &round_id,
            &ModelOutput {
                model_id: mid.clone(),
                description: String::from_str(&env, "enhanced_model"),
                weights_ref: String::from_str(&env, "ipfs://enhanced_weights"),
                global_accuracy: 88,
                validation_score: 85,
                version: 1,
                convergence_metrics,
                privacy_loss: 15,
                communication_cost: 2048,
            },
        );

        assert_eq!(
            client.get_round(&round_id).unwrap().status,
            RoundStatus::Finalized
        );

        let model = client.get_model(&mid).unwrap();
        assert_eq!(model.num_contributors, 2);
        assert_eq!(model.framework, Framework::TensorFlow);
        assert_eq!(model.aggregation_method, AggregationMethod::FedAvg);

        let privacy_metrics = client.get_privacy_metrics(&round_id).unwrap();
        assert!(privacy_metrics.cumulative_privacy_loss > 0);

        let institution1 = client.get_institution(&inst1).unwrap();
        assert!(institution1.reward_balance >= 50);
        assert!(institution1.contribution_quality_score > 70);

        let verification = client
            .get_contribution_verification(&round_id, &inst1)
            .unwrap();
        assert!(verification.quality_score > 50);
        assert!(verification.privacy_compliance);

        assert!(client
            .mock_all_auths()
            .try_submit_update(
                &inst1,
                &round_id,
                &BytesN::from_array(&env, &[9u8; 32]),
                &quality_metrics,
                &BytesN::from_array(&env, &[10u8; 32])
            )
            .is_err());
    }
}
