use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::types::{DataKey, Error, PatientPredictionsSummary, PredictionConfig};

pub const PREDICTION_COUNTER: Symbol = symbol_short!("PRED_CT");
pub const MAX_BPS: u32 = 10_000;
pub const HIGH_RISK_THRESHOLD_BPS: u32 = 7_500;
pub const SECONDS_PER_DAY: u64 = 24 * 3600;

pub fn load_config(env: &Env) -> Result<PredictionConfig, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::ConfigNotSet)
}

pub fn ensure_admin(env: &Env, caller: &Address) -> Result<PredictionConfig, Error> {
    let config = load_config(env)?;
    if config.admin != *caller {
        return Err(Error::NotAuthorized);
    }
    Ok(config)
}

pub fn ensure_predictor(env: &Env, caller: &Address) -> Result<PredictionConfig, Error> {
    let config = load_config(env)?;
    if config.predictor != *caller {
        return Err(Error::NotAuthorized);
    }
    if !config.enabled {
        return Err(Error::Disabled);
    }
    Ok(config)
}

pub fn next_prediction_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&PREDICTION_COUNTER)
        .unwrap_or(0);
    let next = current.saturating_add(1);
    env.storage().instance().set(&PREDICTION_COUNTER, &next);
    next
}

pub fn validate_bps(value: u32, error: Error) -> Result<(), Error> {
    if value > MAX_BPS {
        return Err(error);
    }
    Ok(())
}

pub fn empty_patient_summary() -> PatientPredictionsSummary {
    PatientPredictionsSummary {
        latest_prediction_id: 0,
        high_risk_predictions: 0,
        total_predictions: 0,
        avg_confidence_bps: 0,
        last_prediction_date: 0,
    }
}
