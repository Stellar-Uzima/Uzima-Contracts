#![no_std]
#![allow(clippy::too_many_arguments)] // FIXED: Allow too many arguments for Soroban contract functions

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, xdr::ToXdr, Address, BytesN,
    Env, Map, String, Symbol, Vec,
};

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
}

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const PREDICTOR: Symbol = symbol_short!("PREDICTOR");
const PREDICTION_HORIZON: Symbol = symbol_short!("HORIZON"); // in seconds
const MIN_CONFIDENCE: Symbol = symbol_short!("MIN_CONF"); // bps (basis points)
const PAUSED: Symbol = symbol_short!("PAUSED");
const PREDICTIONS: Symbol = symbol_short!("PREDICTS");
const MODEL_REGISTRY: Symbol = symbol_short!("MODELS");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredictionRecord {
    pub id: u64,
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub timestamp: u64,
    pub predicted_value: u32,
    pub confidence_bps: u32,
    pub horizon_seconds: u32,
    pub outcome_type: String,
    pub features_hash: BytesN<32>,
    pub explanation_ref: String,
    pub risk_factors: Vec<String>,
}

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
    /// Initialize the contract
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

        env.storage().persistent().set(&ADMIN, &admin);
        env.storage().persistent().set(&PREDICTOR, &predictor);
        env.storage()
            .persistent()
            .set(&PREDICTION_HORIZON, &horizon);
        env.storage()
            .persistent()
            .set(&MIN_CONFIDENCE, &min_confidence);
        env.storage().persistent().set(&PAUSED, &false);

        Ok(true)
    }

    /// Register a new predictive model
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

        let mut models: Map<BytesN<32>, ModelMetadata> = env
            .storage()
            .persistent()
            .get(&MODEL_REGISTRY)
            .unwrap_or(Map::new(&env));

        models.set(model_id, metadata);
        env.storage().persistent().set(&MODEL_REGISTRY, &models);

        Ok(true)
    }

    /// Update configuration
    pub fn update_config(
        env: Env,
        admin: Address,
        new_predictor: Option<Address>,
        new_horizon: Option<u32>,
        new_min_confidence: Option<u32>,
        enabled: Option<bool>,
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

        if let Some(pred) = new_predictor {
            env.storage().persistent().set(&PREDICTOR, &pred);
        }

        if let Some(hor) = new_horizon {
            env.storage().persistent().set(&PREDICTION_HORIZON, &hor);
        }

        if let Some(conf) = new_min_confidence {
            env.storage().persistent().set(&MIN_CONFIDENCE, &conf);
        }

        if let Some(is_active) = enabled {
            // If enabled is true, paused is false
            env.storage().persistent().set(&PAUSED, &(!is_active));
        }

        Ok(true)
    }

    /// Submit a prediction
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

        if env.storage().persistent().get(&PAUSED).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }

        // Verify predictor
        let stored_predictor: Address = env
            .storage()
            .persistent()
            .get(&PREDICTOR)
            .ok_or(Error::NotAuthorized)?;
        if predictor != stored_predictor {
            return Err(Error::NotAuthorized);
        }

        // Check constraints
        let min_conf: u32 = env.storage().persistent().get(&MIN_CONFIDENCE).unwrap_or(0);
        if confidence_bps < min_conf {
            return Err(Error::InvalidConfidence);
        }

        let horizon: u32 = env
            .storage()
            .persistent()
            .get(&PREDICTION_HORIZON)
            .unwrap_or(0);

        // Generate ID
        let id = env.ledger().timestamp(); // Simple ID for now

        // FIXED: Used .to_xdr(&env) instead of .into() for hashing
        let features_hash = env.crypto().sha256(&features.to_xdr(&env));

        let record = PredictionRecord {
            id,
            patient: patient.clone(),
            model_id,
            timestamp: env.ledger().timestamp(),
            predicted_value,
            confidence_bps,
            horizon_seconds: horizon,
            outcome_type,
            features_hash: features_hash.into(),
            explanation_ref,
            risk_factors,
        };

        let mut predictions: Map<u64, PredictionRecord> = env
            .storage()
            .persistent()
            .get(&PREDICTIONS)
            .unwrap_or(Map::new(&env));

        predictions.set(id, record);
        env.storage().persistent().set(&PREDICTIONS, &predictions);

        Ok(id)
    }

    /// Get a prediction by ID
    pub fn get_prediction(env: Env, id: u64) -> Result<PredictionRecord, Error> {
        let predictions: Map<u64, PredictionRecord> = env
            .storage()
            .persistent()
            .get(&PREDICTIONS)
            .ok_or(Error::InvalidPrediction)?;

        predictions.get(id).ok_or(Error::InvalidPrediction)
    }

    /// Get patient predictions
    pub fn get_patient_predictions(
        env: Env,
        patient: Address,
    ) -> Result<Vec<PredictionRecord>, Error> {
        let predictions: Map<u64, PredictionRecord> = env
            .storage()
            .persistent()
            .get(&PREDICTIONS)
            .ok_or(Error::InvalidPrediction)?;

        let mut result = Vec::new(&env);
        for (_, record) in predictions.iter() {
            if record.patient == patient {
                result.push_back(record);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec};

    #[test]
    fn test_prediction_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize
        client.initialize(&admin, &predictor, &30u32, &5000u32);

        // Register model
        let model_id = BytesN::from_array(&env, &[0u8; 32]);
        client.register_model(
            &admin,
            &model_id,
            &String::from_str(&env, "Diabetes Risk V1"),
            &String::from_str(&env, "1.0.0"),
            &String::from_str(&env, "XGBoost"),
        );

        // Make prediction
        let features = vec![&env, String::from_str(&env, "age")];
        let risk_factors = vec![&env, String::from_str(&env, "age")];

        // FIXED: Removed unwrap() because client calls return the value directly
        let prediction_id = client.make_prediction(
            &predictor,
            &patient,
            &model_id,
            &String::from_str(&env, "diabetes_risk"),
            &7500u32,
            &8000u32, // > 5000 min confidence
            &features,
            &String::from_str(&env, "ipfs://explanation"),
            &risk_factors,
        );

        // Verify
        let record = client.get_prediction(&prediction_id);
        assert_eq!(record.predicted_value, 7500u32);
        assert_eq!(record.confidence_bps, 8000u32);
    }

    #[test]
    fn test_low_confidence_rejection() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin, &predictor, &30u32, &9000u32); // High confidence required

        let model_id = BytesN::from_array(&env, &[0u8; 32]);
        let features = vec![&env, String::from_str(&env, "age")];
        let risk_factors = vec![&env, String::from_str(&env, "age")];

        // FIXED: Use try_make_prediction to catch error
        let result = client.try_make_prediction(
            &predictor,
            &patient,
            &model_id,
            &String::from_str(&env, "diabetes_risk"),
            &7500u32,
            &8000u32, // < 9000 min confidence
            &features,
            &String::from_str(&env, "ipfs://explanation"),
            &risk_factors,
        );

        // FIXED: Correct assertion for error result
        assert_eq!(result, Err(Ok(Error::InvalidConfidence)));
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

        // FIXED: Passed references to Some(...) options
        let update_result = client.update_config(
            &admin,
            &Some(Address::generate(&env)), // new predictor
            &Some(60u32),                   // new horizon
            &Some(7000u32),                 // new min confidence
            &Some(false),                   // disable
        );

        // FIXED: Client returns bool directly, not Result
        assert!(update_result);

        // Verify paused (enabled = false means paused = true)
        let model_id = BytesN::from_array(&env, &[0u8; 32]);
        let patient = Address::generate(&env);
        let features = vec![&env, String::from_str(&env, "high_bmi")];
        let risk_factors = vec![&env, String::from_str(&env, "high_bmi")];

        // Should fail because paused
        let result = client.try_make_prediction(
            &predictor, // Old predictor (even though changed, auth check might fail on stored, but here Paused check comes first)
            &patient,
            &model_id,
            &String::from_str(&env, "outcome"),
            &5000u32,
            &8000u32,
            &features,
            &String::from_str(&env, "ref"),
            &risk_factors,
        );

        // Expect ContractPaused error
        assert_eq!(result, Err(Ok(Error::ContractPaused)));
    }

    #[test]
    fn test_valid_prediction_storage() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin, &predictor, &30u32, &5000u32);

        let model_id = BytesN::from_array(&env, &[0u8; 32]);
        let features = vec![&env, String::from_str(&env, "age")];
        let risk_factors = vec![&env, String::from_str(&env, "age")];

        // FIXED: Removed unwrap()
        let prediction_id = client.make_prediction(
            &predictor,
            &patient,
            &model_id,
            &String::from_str(&env, "diabetes_risk"),
            &7500u32,
            &8000u32,
            &features,
            &String::from_str(&env, "ipfs://explanation"),
            &risk_factors,
        );

        // Verify
        let stored = client.get_prediction(&prediction_id);
        assert_eq!(stored.id, prediction_id);
    }
}
