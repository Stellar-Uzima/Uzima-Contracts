use soroban_sdk::{symbol_short, Address, BytesN, Env};

use crate::{
    types::{DataKey, Error, PredictionConfig, PredictionMetrics},
    utils,
};

pub fn initialize(
    env: Env,
    admin: Address,
    predictor: Address,
    prediction_horizon_days: u32,
    min_confidence_bps: u32,
) -> bool {
    admin.require_auth();

    if env.storage().instance().has(&DataKey::Config) {
        return false;
    }

    if min_confidence_bps > utils::MAX_BPS {
        return false;
    }

    let config = PredictionConfig {
        admin,
        predictor,
        prediction_horizon_days,
        enabled: true,
        min_confidence_bps,
    };

    env.storage().instance().set(&DataKey::Config, &config);
    env.storage()
        .instance()
        .set(&utils::PREDICTION_COUNTER, &0u64);
    true
}

pub fn update_config(
    env: Env,
    caller: Address,
    new_predictor: Option<Address>,
    new_horizon: Option<u32>,
    new_min_confidence: Option<u32>,
    enabled: Option<bool>,
) -> Result<bool, Error> {
    caller.require_auth();
    let mut config = utils::ensure_admin(&env, &caller)?;

    if let Some(predictor) = new_predictor {
        config.predictor = predictor;
    }

    if let Some(horizon) = new_horizon {
        if horizon == 0 {
            return Err(Error::InvalidHorizon);
        }
        config.prediction_horizon_days = horizon;
    }

    if let Some(min_confidence) = new_min_confidence {
        utils::validate_bps(min_confidence, Error::InvalidConfidence)?;
        config.min_confidence_bps = min_confidence;
    }

    if let Some(enable_flag) = enabled {
        config.enabled = enable_flag;
    }

    env.storage().instance().set(&DataKey::Config, &config);
    env.events().publish((symbol_short!("CfgUpdate"),), true);

    Ok(true)
}

pub fn get_config(env: Env) -> Option<PredictionConfig> {
    env.storage().instance().get(&DataKey::Config)
}

pub fn get_model_metrics(env: Env, model_id: BytesN<32>) -> Option<PredictionMetrics> {
    env.storage()
        .instance()
        .get(&DataKey::ModelMetrics(model_id))
}

pub fn update_model_metrics(
    env: Env,
    caller: Address,
    model_id: BytesN<32>,
    metrics: PredictionMetrics,
) -> Result<bool, Error> {
    caller.require_auth();
    let _config = utils::ensure_admin(&env, &caller)?;

    utils::validate_bps(metrics.accuracy_bps, Error::InvalidValue)?;
    utils::validate_bps(metrics.precision_bps, Error::InvalidValue)?;
    utils::validate_bps(metrics.recall_bps, Error::InvalidValue)?;
    utils::validate_bps(metrics.f1_score_bps, Error::InvalidValue)?;

    env.storage()
        .instance()
        .set(&DataKey::ModelMetrics(model_id.clone()), &metrics);

    env.events()
        .publish((symbol_short!("MdlMetric"),), model_id);

    Ok(true)
}

pub fn whitelist_predictor(
    env: Env,
    caller: Address,
    predictor_addr: Address,
) -> Result<bool, Error> {
    caller.require_auth();
    let _config = utils::ensure_admin(&env, &caller)?;

    env.storage()
        .instance()
        .set(&DataKey::Whitelist(predictor_addr.clone()), &true);

    env.events()
        .publish((symbol_short!("PredictWL"),), predictor_addr);

    Ok(true)
}

pub fn is_whitelisted_predictor(env: Env, predictor_addr: Address) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Whitelist(predictor_addr))
        .unwrap_or(false)
}
