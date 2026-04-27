use soroban_sdk::{symbol_short, Address, BytesN, Env, String, Vec};

use crate::{
    types::{DataKey, Error, HealthPrediction, PatientPredictionsSummary},
    utils,
};

pub fn make_prediction(
    env: Env,
    caller: Address,
    patient: Address,
    model_id: BytesN<32>,
    outcome_type: String,
    predicted_value: u32,
    confidence_bps: u32,
    features_used: Vec<String>,
    explanation_ref: String,
    risk_factors: Vec<String>,
) -> Result<u64, Error> {
    caller.require_auth();

    let config = utils::ensure_predictor(&env, &caller)?;

    utils::validate_bps(predicted_value, Error::InvalidValue)?;
    utils::validate_bps(confidence_bps, Error::InvalidConfidence)?;

    if confidence_bps < config.min_confidence_bps {
        return Err(Error::LowConfidence);
    }

    if explanation_ref.is_empty() {
        return Err(Error::EmptyInput);
    }

    let timestamp = env.ledger().timestamp();
    let horizon_start = timestamp;
    let horizon_end = timestamp.saturating_add(
        (config.prediction_horizon_days as u64).saturating_mul(utils::SECONDS_PER_DAY),
    );

    let prediction_id = utils::next_prediction_id(&env);
    let prediction = HealthPrediction {
        patient: patient.clone(),
        model_id,
        outcome_type,
        predicted_value,
        confidence_bps,
        prediction_date: timestamp,
        horizon_start,
        horizon_end,
        features_used,
        explanation_ref,
        risk_factors,
    };

    env.storage()
        .instance()
        .set(&DataKey::Prediction(prediction_id), &prediction);

    let mut summary: PatientPredictionsSummary = env
        .storage()
        .instance()
        .get(&DataKey::PatientSummary(patient.clone()))
        .unwrap_or_else(utils::empty_patient_summary);

    let previous_total = summary.total_predictions as u64;
    summary.latest_prediction_id = prediction_id;
    summary.total_predictions = summary.total_predictions.saturating_add(1);

    if predicted_value >= utils::HIGH_RISK_THRESHOLD_BPS {
        summary.high_risk_predictions = summary.high_risk_predictions.saturating_add(1);
    }

    let total_confidence = (summary.avg_confidence_bps as u64)
        .saturating_mul(previous_total)
        .saturating_add(confidence_bps as u64);
    summary.avg_confidence_bps = (total_confidence / summary.total_predictions as u64) as u32;
    summary.last_prediction_date = timestamp;

    env.storage()
        .instance()
        .set(&DataKey::PatientSummary(patient.clone()), &summary);

    env.events().publish(
        (symbol_short!("PredMade"),),
        (prediction_id, patient, predicted_value, confidence_bps),
    );

    Ok(prediction_id)
}

pub fn get_prediction(env: Env, prediction_id: u64) -> Option<HealthPrediction> {
    env.storage()
        .instance()
        .get(&DataKey::Prediction(prediction_id))
}

pub fn get_patient_summary(env: Env, patient: Address) -> Option<PatientPredictionsSummary> {
    env.storage()
        .instance()
        .get(&DataKey::PatientSummary(patient))
}

pub fn has_high_risk_prediction(env: Env, patient: Address) -> bool {
    env.storage()
        .instance()
        .get::<DataKey, PatientPredictionsSummary>(&DataKey::PatientSummary(patient))
        .map(|summary| summary.high_risk_predictions > 0)
        .unwrap_or(false)
}
