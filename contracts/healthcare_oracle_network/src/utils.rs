use soroban_sdk::{symbol_short, Address, Env, String, Vec};

use crate::types::{
    AggregationRound, Config, ConsensusRecord, DataKey, Error, FeedKey, FeedKind, FeedPayload,
    OracleNode, RegulatoryUpdateData, TreatmentOutcomeData, DrugPriceData, ClinicalTrialData,
};

pub fn payload_feed_id_from_trial(payload: &FeedPayload) -> String {
    match payload {
        FeedPayload::ClinicalTrial(data) => data.trial_id.clone(),
        _ => unreachable!(),
    }
}

pub fn payload_feed_id_from_outcome(payload: &FeedPayload) -> String {
    match payload {
        FeedPayload::TreatmentOutcome(data) => data.outcome_id.clone(),
        _ => unreachable!(),
    }
}

pub fn require_initialized(env: &Env) -> Result<(), Error> {
    if !env.storage().instance().has(&DataKey::Config) {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

pub fn require_admin(env: &Env, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    let cfg: Config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)?;

    if cfg.admin != admin {
        return Err(Error::Unauthorized);
    }

    Ok(())
}

pub fn require_verified_oracle(env: &Env, operator: Address) -> Result<Config, Error> {
    let cfg: Config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)?;

    let node = read_oracle(env, operator)?;
    if !node.verified {
        return Err(Error::OracleNotVerified);
    }
    if !node.active {
        return Err(Error::OracleInactive);
    }

    Ok(cfg)
}

pub fn read_oracle(env: &Env, operator: Address) -> Result<OracleNode, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Oracle(operator))
        .ok_or(Error::OracleNotFound)
}

pub fn submit_payload(
    env: Env,
    operator: Address,
    kind: FeedKind,
    feed_id: String,
    payload: FeedPayload,
    cfg: Config,
) -> Result<u64, Error> {
    let key = FeedKey { kind, feed_id };
    let round_id = ensure_active_round(&env, key.clone())?;

    let submission_key = DataKey::Submission(key.clone(), round_id, operator.clone());
    if env.storage().persistent().has(&submission_key) {
        return Err(Error::SubmissionAlreadyExists);
    }

    env.storage().persistent().set(&submission_key, &payload);

    let mut round: AggregationRound = env
        .storage()
        .persistent()
        .get(&DataKey::Round(key.clone(), round_id))
        .ok_or(Error::RoundNotFound)?;
    round.submissions = round.submissions.saturating_add(1);
    env.storage()
        .persistent()
        .set(&DataKey::Round(key.clone(), round_id), &round);

    let mut node = read_oracle(&env, operator.clone())?;
    node.submissions = node.submissions.saturating_add(1);
    node.last_seen = env.ledger().timestamp();
    env.storage()
        .persistent()
        .set(&DataKey::Oracle(operator), &node);

    if round.submissions >= cfg.min_submissions {
        let _ = finalize_round(env.clone(), key.clone(), round_id)?;
    }

    Ok(round_id)
}

pub fn finalize_round(env: Env, key: FeedKey, round_id: u64) -> Result<ConsensusRecord, Error> {
    let cfg: Config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)?;

    let mut round: AggregationRound = env
        .storage()
        .persistent()
        .get(&DataKey::Round(key.clone(), round_id))
        .ok_or(Error::RoundNotFound)?;

    if round.finalized {
        return Err(Error::ConsensusAlreadyFinalized);
    }

    let all_oracles: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::OracleList)
        .unwrap_or(Vec::new(&env));

    let mut submitters = Vec::<Address>::new(&env);
    let mut payloads = Vec::<FeedPayload>::new(&env);
    let mut weights = Vec::<i128>::new(&env);

    let mut i = 0;
    while i < all_oracles.len() {
        let oracle = all_oracles.get(i).unwrap();
        let node = read_oracle(&env, oracle.clone())?;

        if node.verified && node.active && node.reputation >= cfg.min_reputation {
            let submission_key = DataKey::Submission(key.clone(), round_id, oracle.clone());
            if env.storage().persistent().has(&submission_key) {
                let payload: FeedPayload = env
                    .storage()
                    .persistent()
                    .get(&submission_key)
                    .ok_or(Error::InvalidData)?;
                submitters.push_back(oracle);
                payloads.push_back(payload);
                weights.push_back(if node.reputation > 0 { node.reputation } else { 1 });
            }
        }
        i += 1;
    }

    if submitters.len() < cfg.min_submissions {
        return Err(Error::InsufficientSubmissions);
    }

    let aggregated = aggregate_payload(
        &env,
        key.kind,
        payloads.clone(),
        weights.clone(),
        key.feed_id.clone(),
    )?;

    let confidence_bps = compute_confidence_bps(submitters.len(), all_oracles.len());
    let consensus = ConsensusRecord {
        key: key.clone(),
        payload: aggregated,
        round_id,
        finalized_at: env.ledger().timestamp(),
        submitters: submitters.clone(),
        confidence_bps,
        disputed: false,
    };

    env.storage()
        .persistent()
        .set(&DataKey::Consensus(key.clone()), &consensus);

    round.finalized = true;
    env.storage()
        .persistent()
        .set(&DataKey::Round(key, round_id), &round);

    reward_and_slash(&env, &consensus, submitters, payloads)?;

    env.events()
        .publish((symbol_short!("consens"), round_id), confidence_bps);

    Ok(consensus)
}

pub fn aggregate_payload(
    env: &Env,
    kind: FeedKind,
    payloads: Vec<FeedPayload>,
    weights: Vec<i128>,
    feed_id: String,
) -> Result<FeedPayload, Error> {
    let mut index = 0;
    let mut sum_weight = 0i128;
    while index < weights.len() {
        sum_weight = sum_weight.saturating_add(weights.get(index).unwrap_or(1));
        index += 1;
    }

    if sum_weight <= 0 {
        return Err(Error::InvalidData);
    }

    match kind {
        FeedKind::DrugPricing => {
            let mut price_weighted = 0i128;
            let mut availability_weighted = 0i128;
            let mut observed_at = 0u64;
            let mut ndc_code = String::from_str(env, "");
            let mut currency = String::from_str(env, "");

            let mut i = 0;
            while i < payloads.len() {
                let weight = weights.get(i).unwrap_or(1);
                match payloads.get(i).unwrap() {
                    FeedPayload::DrugPrice(value) => {
                        if ndc_code.len() == 0 {
                            ndc_code = value.ndc_code.clone();
                            currency = value.currency.clone();
                        }
                        price_weighted = price_weighted
                            .saturating_add(value.price_minor.saturating_mul(weight));
                        availability_weighted = availability_weighted.saturating_add(
                            (value.availability_units as i128).saturating_mul(weight),
                        );
                        if value.observed_at > observed_at {
                            observed_at = value.observed_at;
                        }
                    }
                    _ => return Err(Error::InvalidFeedType),
                }
                i += 1;
            }

            Ok(FeedPayload::DrugPrice(DrugPriceData {
                ndc_code,
                currency,
                price_minor: price_weighted / sum_weight,
                availability_units: (availability_weighted / sum_weight) as u32,
                observed_at,
            }))
        }
        FeedKind::ClinicalTrial => {
            let mut phase_weighted = 0i128;
            let mut enrolled_weighted = 0i128;
            let mut success_weighted = 0i128;
            let mut adverse_weighted = 0i128;
            let mut published_at = 0u64;
            let mut result_hash = String::from_str(env, "");

            let mut i = 0;
            while i < payloads.len() {
                let weight = weights.get(i).unwrap_or(1);
                match payloads.get(i).unwrap() {
                    FeedPayload::ClinicalTrial(value) => {
                        phase_weighted = phase_weighted
                            .saturating_add((value.phase as i128).saturating_mul(weight));
                        enrolled_weighted = enrolled_weighted
                            .saturating_add((value.enrolled as i128).saturating_mul(weight));
                        success_weighted = success_weighted.saturating_add(
                            (value.success_rate_bps as i128).saturating_mul(weight),
                        );
                        adverse_weighted = adverse_weighted.saturating_add(
                            (value.adverse_event_rate_bps as i128).saturating_mul(weight),
                        );
                        if value.published_at > published_at {
                            published_at = value.published_at;
                            result_hash = value.result_hash.clone();
                        }
                    }
                    _ => return Err(Error::InvalidFeedType),
                }
                i += 1;
            }

            Ok(FeedPayload::ClinicalTrial(ClinicalTrialData {
                trial_id: feed_id,
                phase: (phase_weighted / sum_weight) as u32,
                enrolled: (enrolled_weighted / sum_weight) as u32,
                success_rate_bps: (success_weighted / sum_weight) as u32,
                adverse_event_rate_bps: (adverse_weighted / sum_weight) as u32,
                result_hash,
                published_at,
            }))
        }
        FeedKind::RegulatoryUpdate => {
            let mut best_weight = -1i128;
            let mut picked: Option<RegulatoryUpdateData> = None;
            let mut i = 0;
            while i < payloads.len() {
                let weight = weights.get(i).unwrap_or(1);
                match payloads.get(i).unwrap() {
                    FeedPayload::RegulatoryUpdate(value) => {
                        if weight > best_weight {
                            best_weight = weight;
                            picked = Some(value.clone());
                        }
                    }
                    _ => return Err(Error::InvalidFeedType),
                }
                i += 1;
            }

            if let Some(mut value) = picked {
                value.regulation_id = feed_id;
                Ok(FeedPayload::RegulatoryUpdate(value))
            } else {
                Err(Error::InvalidData)
            }
        }
        FeedKind::TreatmentOutcome => {
            let mut improvement_weighted = 0i128;
            let mut readmission_weighted = 0i128;
            let mut mortality_weighted = 0i128;
            let mut sample_weighted = 0i128;
            let mut reported_at = 0u64;
            let mut condition_code = String::from_str(env, "");
            let mut treatment_code = String::from_str(env, "");

            let mut i = 0;
            while i < payloads.len() {
                let weight = weights.get(i).unwrap_or(1);
                match payloads.get(i).unwrap() {
                    FeedPayload::TreatmentOutcome(value) => {
                        if condition_code.len() == 0 {
                            condition_code = value.condition_code.clone();
                            treatment_code = value.treatment_code.clone();
                        }
                        improvement_weighted = improvement_weighted.saturating_add(
                            (value.improvement_rate_bps as i128).saturating_mul(weight),
                        );
                        readmission_weighted = readmission_weighted.saturating_add(
                            (value.readmission_rate_bps as i128).saturating_mul(weight),
                        );
                        mortality_weighted = mortality_weighted.saturating_add(
                            (value.mortality_rate_bps as i128).saturating_mul(weight),
                        );
                        sample_weighted = sample_weighted
                            .saturating_add((value.sample_size as i128).saturating_mul(weight));
                        if value.reported_at > reported_at {
                            reported_at = value.reported_at;
                        }
                    }
                    _ => return Err(Error::InvalidFeedType),
                }
                i += 1;
            }

            Ok(FeedPayload::TreatmentOutcome(TreatmentOutcomeData {
                outcome_id: feed_id,
                condition_code,
                treatment_code,
                improvement_rate_bps: (improvement_weighted / sum_weight) as u32,
                readmission_rate_bps: (readmission_weighted / sum_weight) as u32,
                mortality_rate_bps: (mortality_weighted / sum_weight) as u32,
                sample_size: (sample_weighted / sum_weight) as u32,
                reported_at,
            }))
        }
    }
}

pub fn reward_and_slash(
    env: &Env,
    consensus: &ConsensusRecord,
    submitters: Vec<Address>,
    payloads: Vec<FeedPayload>,
) -> Result<(), Error> {
    let mut i = 0;
    while i < submitters.len() {
        let submitter = submitters.get(i).unwrap();
        let payload = payloads.get(i).unwrap();
        let delta = reputation_delta(consensus.payload.clone(), payload);
        adjust_reputation(env, submitter, delta, delta < 0)?;
        i += 1;
    }
    Ok(())
}

pub fn reputation_delta(consensus: FeedPayload, payload: FeedPayload) -> i128 {
    match (consensus, payload) {
        (FeedPayload::DrugPrice(consensus_data), FeedPayload::DrugPrice(payload_data)) => {
            if consensus_data.price_minor <= 0 {
                return 1;
            }
            let diff = if payload_data.price_minor > consensus_data.price_minor {
                payload_data
                    .price_minor
                    .saturating_sub(consensus_data.price_minor)
            } else {
                consensus_data
                    .price_minor
                    .saturating_sub(payload_data.price_minor)
            };
            let bps = diff.saturating_mul(10_000) / consensus_data.price_minor;
            if bps <= 500 { 5 } else { -3 }
        }
        (
            FeedPayload::ClinicalTrial(consensus_data),
            FeedPayload::ClinicalTrial(payload_data),
        ) => {
            let diff = if payload_data.success_rate_bps > consensus_data.success_rate_bps {
                payload_data
                    .success_rate_bps
                    .saturating_sub(consensus_data.success_rate_bps)
            } else {
                consensus_data
                    .success_rate_bps
                    .saturating_sub(payload_data.success_rate_bps)
            };
            if diff <= 700 { 4 } else { -2 }
        }
        (
            FeedPayload::RegulatoryUpdate(consensus_data),
            FeedPayload::RegulatoryUpdate(payload_data),
        ) => {
            if consensus_data.status == payload_data.status {
                4
            } else {
                -4
            }
        }
        (
            FeedPayload::TreatmentOutcome(consensus_data),
            FeedPayload::TreatmentOutcome(payload_data),
        ) => {
            let diff = if payload_data.improvement_rate_bps > consensus_data.improvement_rate_bps {
                payload_data
                    .improvement_rate_bps
                    .saturating_sub(consensus_data.improvement_rate_bps)
            } else {
                consensus_data
                    .improvement_rate_bps
                    .saturating_sub(payload_data.improvement_rate_bps)
            };
            if diff <= 700 { 4 } else { -2 }
        }
        _ => -5,
    }
}

pub fn adjust_reputation(
    env: &Env,
    operator: Address,
    delta: i128,
    is_dispute: bool,
) -> Result<(), Error> {
    let mut node = read_oracle(env, operator.clone())?;
    node.reputation = node.reputation.saturating_add(delta);
    if node.reputation < 0 {
        node.reputation = 0;
    }
    if is_dispute {
        node.disputes = node.disputes.saturating_add(1);
    }
    node.last_seen = env.ledger().timestamp();
    env.storage()
        .persistent()
        .set(&DataKey::Oracle(operator), &node);
    Ok(())
}

pub fn ensure_active_round(env: &Env, key: FeedKey) -> Result<u64, Error> {
    if let Ok(round_id) = active_round_id(env, key.clone()) {
        return Ok(round_id);
    }

    let next_id = env
        .storage()
        .persistent()
        .get(&DataKey::RoundCounter(key.clone()))
        .unwrap_or(0u64)
        .saturating_add(1);

    let round = AggregationRound {
        id: next_id,
        started_at: env.ledger().timestamp(),
        finalized: false,
        submissions: 0,
    };

    env.storage()
        .persistent()
        .set(&DataKey::RoundCounter(key.clone()), &next_id);
    env.storage()
        .persistent()
        .set(&DataKey::Round(key, next_id), &round);

    Ok(next_id)
}

pub fn active_round_id(env: &Env, key: FeedKey) -> Result<u64, Error> {
    let latest: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::RoundCounter(key.clone()))
        .ok_or(Error::RoundNotFound)?;

    let round: AggregationRound = env
        .storage()
        .persistent()
        .get(&DataKey::Round(key, latest))
        .ok_or(Error::RoundNotFound)?;

    if round.finalized {
        return Err(Error::RoundNotFound);
    }

    Ok(latest)
}

pub fn compute_confidence_bps(submitters: u32, total_oracles: u32) -> u32 {
    if total_oracles == 0 {
        return 0;
    }

    let mut score = submitters.saturating_mul(10_000) / total_oracles;
    if score > 10_000 {
        score = 10_000;
    }
    score
}
