// Predictive Analytics Contract - Health predictions with proper validation
#![no_std]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::panic)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
    String, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct PredictionConfig {
    pub admin: Address,
    pub predictor: Address,
    pub prediction_horizon_days: u32,
    pub enabled: bool,
    pub min_confidence_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct HealthPrediction {
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub outcome_type: String,
    pub predicted_value: u32,
    pub confidence_bps: u32,
    pub prediction_date: u64,
    pub horizon_start: u64,
    pub horizon_end: u64,
    pub features_used: Vec<String>,
    pub explanation_ref: String,
    pub risk_factors: Vec<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct PredictionMetrics {
    pub accuracy_bps: u32,
    pub precision_bps: u32,
    pub recall_bps: u32,
    pub f1_score_bps: u32,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PatientPredictionsSummary {
    pub latest_prediction_id: u64,
    pub high_risk_predictions: u32,
    pub total_predictions: u32,
    pub avg_confidence_bps: u32,
    pub last_prediction_date: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Prediction(u64),
    PatientSummary(Address),
    ModelMetrics(BytesN<32>),
    PredictionCounter,
    Whitelist(Address),
}

const PREDICTION_COUNTER: Symbol = symbol_short!("PRED_CT");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ContractPaused = 2,
    ModelNotFound = 3,
    InvalidPrediction = 4,
    InvalidConfidence = 5,
    InvalidHorizon = 6,
    LimitExceeded = 7,
    ConfigError = 8,
    Disabled = 9,
    InvalidValue = 10,
    LowConfidence = 11,
}

const ADMIN: Symbol = symbol_short!("ADMIN");
const PREDICTOR: Symbol = symbol_short!("PREDICTOR");
const PREDICTION_HORIZON: Symbol = symbol_short!("HORIZON");
const MIN_CONFIDENCE: Symbol = symbol_short!("MIN_CONF");
const PAUSED: Symbol = symbol_short!("PAUSED");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelMetadata {
    pub model_id: BytesN<32>,
    pub name: String,
    pub version: String,
    pub algorithm_type: String,
    pub created_at: u64,
    pub active: bool,
}

#[contract]
pub struct PredictiveAnalyticsContract;

#[contractimpl]
impl PredictiveAnalyticsContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        predictor: Address,
        horizon: u32,
        min_confidence: u32,
    ) -> Result<bool, Error> {
        admin.require_auth();

        if env.storage().persistent().has(&ADMIN) {
            return Err(Error::ConfigError);
        }

        let config = PredictionConfig {
            admin: admin.clone(),
            predictor: predictor.clone(),
            prediction_horizon_days: horizon,
            enabled: true,
            min_confidence_bps: min_confidence,
        };

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&PREDICTOR, &predictor);
        env.storage()
            .persistent()
            .set(&PREDICTION_HORIZON, &horizon);
        env.storage()
            .persistent()
            .set(&MIN_CONFIDENCE, &min_confidence);
        env.storage().persistent().set(&PAUSED, &false);
        env.storage().instance().set(&DataKey::Config, &config);

        Ok(true)
    }

    pub fn register_model(
        env: Env,
        admin: Address,
        model_id: BytesN<32>,
        name: String,
        version: String,
        algorithm_type: String,
    ) -> Result<bool, Error> {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .persistent()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        let metadata = ModelMetadata {
            model_id: model_id.clone(),
            name,
            version,
            algorithm_type,
            created_at: env.ledger().timestamp(),
            active: true,
        };

        env.storage()
            .instance()
            .set(&DataKey::ModelMetrics(model_id), &metadata);
        Ok(true)
    }

    fn next_prediction_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .instance()
            .get(&PREDICTION_COUNTER)
            .unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&PREDICTION_COUNTER, &next);
        next
    }

    pub fn update_config(
        env: Env,
        admin: Address,
        new_predictor: Option<Address>,
        new_horizon: Option<u32>,
        new_min_confidence: Option<u32>,
        enabled: Option<bool>,
    ) -> Result<bool, Error> {
        admin.require_auth();

        let mut config: PredictionConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::ConfigError)?;

        if admin != config.admin {
            return Err(Error::NotAuthorized);
        }

        if let Some(pred) = new_predictor {
            config.predictor = pred.clone();
            env.storage().persistent().set(&PREDICTOR, &pred);
        }
        if let Some(hor) = new_horizon {
            config.prediction_horizon_days = hor;
            env.storage().persistent().set(&PREDICTION_HORIZON, &hor);
        }
        if let Some(conf) = new_min_confidence {
            config.min_confidence_bps = conf;
            env.storage().persistent().set(&MIN_CONFIDENCE, &conf);
        }
        if let Some(enable_flag) = enabled {
            config.enabled = enable_flag;
        }

        env.storage().instance().set(&DataKey::Config, &config);
        env.events().publish((symbol_short!("CfgUpdate"),), true);
        Ok(true)
    }

    pub fn make_prediction(
        env: Env,
        predictor: Address,
        patient: Address,
        model_id: BytesN<32>,
        outcome_type: String,
        predicted_value: u32,
        confidence_bps: u32,
        features: Vec<String>,
        explanation_ref: String,
        risk_factors: Vec<String>,
    ) -> Result<u64, Error> {
        predictor.require_auth();

        let config: PredictionConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::ConfigError)?;

        if !config.enabled {
            return Err(Error::Disabled);
        }
        if predicted_value > 10_000 || confidence_bps > 10_000 {
            return Err(Error::InvalidValue);
        }
        if confidence_bps < config.min_confidence_bps {
            return Err(Error::LowConfidence);
        }

        let timestamp = env.ledger().timestamp();
        let prediction_id = Self::next_prediction_id(&env);

        let prediction = HealthPrediction {
            patient: patient.clone(),
            model_id,
            outcome_type,
            predicted_value,
            confidence_bps,
            prediction_date: timestamp,
            horizon_start: timestamp,
            horizon_end: timestamp + (config.prediction_horizon_days as u64 * 86400),
            features_used: features,
            explanation_ref,
            risk_factors,
        };

        env.storage()
            .instance()
            .set(&DataKey::Prediction(prediction_id), &prediction);

        let mut summary = env
            .storage()
            .instance()
            .get(&DataKey::PatientSummary(patient.clone()))
            .unwrap_or(PatientPredictionsSummary {
                latest_prediction_id: 0,
                high_risk_predictions: 0,
                total_predictions: 0,
                avg_confidence_bps: 0,
                last_prediction_date: 0,
            });

        summary.latest_prediction_id = prediction_id;
        summary.total_predictions += 1;
        if predicted_value > 7500 {
            summary.high_risk_predictions += 1;
        }

        let total_conf = (summary.avg_confidence_bps as u64
            * (summary.total_predictions as u64 - 1)
            + confidence_bps as u64)
            / summary.total_predictions as u64;
        summary.avg_confidence_bps = total_conf as u32;
        summary.last_prediction_date = timestamp;

        env.storage()
            .instance()
            .set(&DataKey::PatientSummary(patient.clone()), &summary);
        env.events().publish(
            (symbol_short!("PredMade"),),
            (prediction_id, patient, predicted_value),
        );

        Ok(prediction_id)
    }

    pub fn get_prediction(env: Env, prediction_id: u64) -> Option<HealthPrediction> {
        env.storage()
            .instance()
            .get(&DataKey::Prediction(prediction_id))
    }

    pub fn get_config(env: Env) -> Option<PredictionConfig> {
        env.storage().instance().get(&DataKey::Config)
    }

    pub fn get_patient_summary(env: Env, patient: Address) -> Option<PatientPredictionsSummary> {
        env.storage()
            .instance()
            .get(&DataKey::PatientSummary(patient))
    }

    pub fn has_high_risk_prediction(env: Env, patient: Address) -> bool {
        let summary: Option<PatientPredictionsSummary> = env
            .storage()
            .instance()
            .get(&DataKey::PatientSummary(patient));
        summary.is_some_and(|s| s.high_risk_predictions > 0)
    }

    pub fn whitelist_predictor(
        env: Env,
        admin: Address,
        predictor: Address,
    ) -> Result<bool, Error> {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Whitelist(predictor.clone()), &true);
        env.events()
            .publish((symbol_short!("PredictWL"),), predictor);
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::vec;

    #[test]
    fn test_prediction_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin, &predictor, &30u32, &5000u32);

        let model_id = BytesN::from_array(&env, &[1; 32]);
        let outcome_type = String::from_str(&env, "diabetes_risk");
        let features = vec![&env, String::from_str(&env, "bmi")];
        let risk_factors = vec![&env, String::from_str(&env, "high_bmi")];

        let prediction_id = client.make_prediction(
            &predictor,
            &patient,
            &model_id,
            &outcome_type,
            &8000u32,
            &9000u32,
            &features,
            &String::from_str(&env, "ipfs://ref"),
            &risk_factors,
        );

        assert_eq!(prediction_id, 1u64);
        assert!(client.has_high_risk_prediction(&patient));
    }

    #[test]
    fn test_config_update() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);

        client.initialize(&admin, &predictor, &30u32, &5000u32);

        client.update_config(
            &admin,
            &Some(Address::generate(&env)),
            &Some(60u32),
            &Some(7000u32),
            &Some(false),
        );

        let config = client.get_config().unwrap();
        assert_eq!(config.prediction_horizon_days, 60u32);
        assert!(!config.enabled);
    }
}
