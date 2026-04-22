use soroban_sdk::{Address, Env, String};

use crate::types::{
    ClinicalTrialData, ConsensusRecord, DataKey, DrugPriceData, Error, FeedKey, FeedKind,
    FeedPayload, RegulatoryAuthority, RegulatoryStatus, RegulatoryUpdateData,
    TreatmentOutcomeData,
};
use crate::utils;

pub fn submit_drug_price(
    env: Env,
    operator: Address,
    feed_id: String,
    ndc_code: String,
    currency: String,
    price_minor: i128,
    availability_units: u32,
    observed_at: u64,
) -> Result<u64, Error> {
    operator.require_auth();
    let config = utils::require_verified_oracle(&env, operator.clone())?;

    if feed_id.len() == 0 || ndc_code.len() == 0 || currency.len() == 0 {
        return Err(Error::InvalidData);
    }

    if price_minor <= 0
        || price_minor > config.max_drug_price_minor
        || availability_units > config.max_availability_units
    {
        return Err(Error::InvalidData);
    }

    let payload = FeedPayload::DrugPrice(DrugPriceData {
        ndc_code,
        currency,
        price_minor,
        availability_units,
        observed_at,
    });

    utils::submit_payload(env, operator, FeedKind::DrugPricing, feed_id, payload, config)
}

pub fn submit_clinical_trial(
    env: Env,
    operator: Address,
    trial_id: String,
    phase: u32,
    enrolled: u32,
    success_rate_bps: u32,
    adverse_event_rate_bps: u32,
    result_hash: String,
    published_at: u64,
) -> Result<u64, Error> {
    operator.require_auth();
    let config = utils::require_verified_oracle(&env, operator.clone())?;

    if trial_id.len() == 0 || result_hash.len() == 0 {
        return Err(Error::InvalidData);
    }

    if phase == 0
        || phase > 4
        || enrolled == 0
        || success_rate_bps > 10_000
        || adverse_event_rate_bps > 10_000
    {
        return Err(Error::InvalidData);
    }

    let payload = FeedPayload::ClinicalTrial(ClinicalTrialData {
        trial_id,
        phase,
        enrolled,
        success_rate_bps,
        adverse_event_rate_bps,
        result_hash,
        published_at,
    });

    let feed_id = utils::payload_feed_id_from_trial(&payload);
    utils::submit_payload(env, operator, FeedKind::ClinicalTrial, feed_id, payload, config)
}

pub fn submit_regulatory_update(
    env: Env,
    operator: Address,
    regulation_id: String,
    authority: RegulatoryAuthority,
    status: RegulatoryStatus,
    title: String,
    details_hash: String,
    effective_at: u64,
) -> Result<u64, Error> {
    operator.require_auth();
    let config = utils::require_verified_oracle(&env, operator.clone())?;

    if regulation_id.len() == 0 || title.len() == 0 || details_hash.len() == 0 {
        return Err(Error::InvalidData);
    }

    let payload = FeedPayload::RegulatoryUpdate(RegulatoryUpdateData {
        regulation_id: regulation_id.clone(),
        authority,
        status,
        title,
        details_hash,
        effective_at,
    });

    utils::submit_payload(
        env,
        operator,
        FeedKind::RegulatoryUpdate,
        regulation_id,
        payload,
        config,
    )
}

pub fn submit_treatment_outcome(
    env: Env,
    operator: Address,
    outcome_id: String,
    condition_code: String,
    treatment_code: String,
    improvement_rate_bps: u32,
    readmission_rate_bps: u32,
    mortality_rate_bps: u32,
    sample_size: u32,
    reported_at: u64,
) -> Result<u64, Error> {
    operator.require_auth();
    let config = utils::require_verified_oracle(&env, operator.clone())?;

    if outcome_id.len() == 0 || condition_code.len() == 0 || treatment_code.len() == 0 {
        return Err(Error::InvalidData);
    }

    if improvement_rate_bps > 10_000
        || readmission_rate_bps > 10_000
        || mortality_rate_bps > 10_000
        || sample_size == 0
    {
        return Err(Error::InvalidData);
    }

    let payload = FeedPayload::TreatmentOutcome(TreatmentOutcomeData {
        outcome_id,
        condition_code,
        treatment_code,
        improvement_rate_bps,
        readmission_rate_bps,
        mortality_rate_bps,
        sample_size,
        reported_at,
    });

    let feed_id = utils::payload_feed_id_from_outcome(&payload);
    utils::submit_payload(
        env,
        operator,
        FeedKind::TreatmentOutcome,
        feed_id,
        payload,
        config,
    )
}

pub fn finalize_feed(env: Env, kind: FeedKind, feed_id: String) -> Result<ConsensusRecord, Error> {
    utils::require_initialized(&env)?;
    if feed_id.len() == 0 {
        return Err(Error::InvalidData);
    }

    let key = FeedKey { kind, feed_id };
    let round_id = utils::active_round_id(&env, key.clone())?;
    utils::finalize_round(env, key, round_id)
}

pub fn get_consensus(env: Env, kind: FeedKind, feed_id: String) -> Option<ConsensusRecord> {
    let key = FeedKey { kind, feed_id };
    env.storage().persistent().get(&DataKey::Consensus(key))
}
