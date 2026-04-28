// Predictive Analytics Contract - Health predictions with proper validation
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]

mod config;
mod predictions;
#[cfg(all(test, feature = "testutils"))]
mod test;
mod types;
mod utils;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

pub use types::{
    DataKey, Error, HealthPrediction, PatientPredictionsSummary, PredictionConfig,
    PredictionMetrics,
};

#[contract]
pub struct PredictiveAnalyticsContract;

#[contractimpl]
impl PredictiveAnalyticsContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        predictor: Address,
        prediction_horizon_days: u32,
        min_confidence_bps: u32,
    ) -> bool {
        config::initialize(
            env,
            admin,
            predictor,
            prediction_horizon_days,
            min_confidence_bps,
        )
    }

    pub fn update_config(
        env: Env,
        caller: Address,
        new_predictor: Option<Address>,
        new_horizon: Option<u32>,
        new_min_confidence: Option<u32>,
        enabled: Option<bool>,
    ) -> Result<bool, Error> {
        config::update_config(
            env,
            caller,
            new_predictor,
            new_horizon,
            new_min_confidence,
            enabled,
        )
    }

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
        predictions::make_prediction(
            env,
            caller,
            patient,
            model_id,
            outcome_type,
            predicted_value,
            confidence_bps,
            features_used,
            explanation_ref,
            risk_factors,
        )
    }

    pub fn get_prediction(env: Env, prediction_id: u64) -> Option<HealthPrediction> {
        predictions::get_prediction(env, prediction_id)
    }

    pub fn get_config(env: Env) -> Option<PredictionConfig> {
        config::get_config(env)
    }

    pub fn get_patient_summary(env: Env, patient: Address) -> Option<PatientPredictionsSummary> {
        predictions::get_patient_summary(env, patient)
    }

    pub fn get_model_metrics(env: Env, model_id: BytesN<32>) -> Option<PredictionMetrics> {
        config::get_model_metrics(env, model_id)
    }

    pub fn update_model_metrics(
        env: Env,
        caller: Address,
        model_id: BytesN<32>,
        metrics: PredictionMetrics,
    ) -> Result<bool, Error> {
        config::update_model_metrics(env, caller, model_id, metrics)
    }

    pub fn has_high_risk_prediction(env: Env, patient: Address) -> bool {
        predictions::has_high_risk_prediction(env, patient)
    }

    pub fn whitelist_predictor(
        env: Env,
        caller: Address,
        predictor_addr: Address,
    ) -> Result<bool, Error> {
        config::whitelist_predictor(env, caller, predictor_addr)
    }

    pub fn is_whitelisted_predictor(env: Env, predictor_addr: Address) -> bool {
        config::is_whitelisted_predictor(env, predictor_addr)
    }
}
