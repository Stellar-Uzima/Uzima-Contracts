#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map,
    String, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct PredictionConfig {
    pub admin: Address,
    pub predictor: Address,
    pub prediction_horizon_days: u32,  // How far ahead to predict
    pub enabled: bool,
    pub min_confidence_bps: u32,  // Minimum confidence in basis points (0-10000)
}

#[derive(Clone)]
#[contracttype]
pub struct HealthPrediction {
    pub patient: Address,
    pub model_id: BytesN<32>,
    pub outcome_type: String,  // e.g., "diabetes_risk", "heart_attack_prob", "readmission_likelihood"
    pub predicted_value: u32,  // Predicted value in basis points (0-10000)
    pub confidence_bps: u32,   // Confidence in basis points (0-10000)
    pub prediction_date: u64,  // Date of prediction
    pub horizon_start: u64,    // Start date for prediction horizon
    pub horizon_end: u64,      // End date for prediction horizon
    pub features_used: Vec<String>,  // Features used in prediction
    pub explanation_ref: String,     // Off-chain reference to detailed explanation
    pub risk_factors: Vec<String>,   // Key risk factors identified
}

#[derive(Clone)]
#[contracttype]
pub struct PredictionMetrics {
    pub accuracy_bps: u32,      // Accuracy in basis points
    pub precision_bps: u32,     // Precision in basis points
    pub recall_bps: u32,        // Recall in basis points
    pub f1_score_bps: u32,      // F1 score in basis points
    pub last_updated: u64,      // Last time metrics were updated
}

#[derive(Clone)]
#[contracttype]
pub struct PatientPredictionsSummary {
    pub latest_prediction_id: u64,
    pub high_risk_predictions: u32,  // Count of high-risk predictions (>7500 bps)
    pub total_predictions: u32,
    pub avg_confidence_bps: u32,
    pub last_prediction_date: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
    Prediction(u64),  // Prediction ID -> HealthPrediction
    PatientSummary(Address),  // Patient -> PatientPredictionsSummary
    ModelMetrics(BytesN<32>),  // Model ID -> PredictionMetrics
    PredictionCounter,
    Whitelist(Address),
}

const PREDICTION_COUNTER: Symbol = symbol_short!("PRED_CT");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    ConfigNotSet = 2,
    Disabled = 3,
    InvalidValue = 4,
    InvalidConfidence = 5,
    RecordNotFound = 6,
    LowConfidence = 7,
    InvalidHorizon = 8,
}

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
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            panic!("Already initialized");
        }

        if min_confidence_bps > 10_000 {
            panic!("min_confidence_bps must be <= 10000");
        }

        let config = PredictionConfig {
            admin,
            predictor,
            prediction_horizon_days,
            enabled: true,
            min_confidence_bps,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&PREDICTION_COUNTER, &0u64);
        true
    }

    fn load_config(env: &Env) -> Result<PredictionConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(Error::ConfigNotSet)
    }

    fn ensure_admin(env: &Env, caller: &Address) -> Result<PredictionConfig, Error> {
        let config = Self::load_config(env)?;
        if config.admin != *caller {
            return Err(Error::NotAuthorized);
        }
        Ok(config)
    }

    fn ensure_predictor(env: &Env, caller: &Address) -> Result<PredictionConfig, Error> {
        let config = Self::load_config(env)?;
        if config.predictor != *caller {
            return Err(Error::NotAuthorized);
        }
        if !config.enabled {
            return Err(Error::Disabled);
        }
        Ok(config)
    }

    fn ensure_enabled(env: &Env) -> Result<PredictionConfig, Error> {
        let config = Self::load_config(env)?;
        if !config.enabled {
            return Err(Error::Disabled);
        }
        Ok(config)
    }

    fn next_prediction_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .instance()
            .get(&PREDICTION_COUNTER)
            .unwrap_or(0);
        let next = current + 1;
        env.storage()
            .instance()
            .set(&PREDICTION_COUNTER, &next);
        next
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
        let mut config = Self::ensure_admin(&env, &caller)?;

        if let Some(predictor) = new_predictor {
            config.predictor = predictor;
        }

        if let Some(horizon) = new_horizon {
            if horizon == 0 {
                return Err(Error::InvalidHorizon);
            }
            config.prediction_horizon_days = horizon;
        }

        if let Some(min_conf) = new_min_confidence {
            if min_conf > 10_000 {
                return Err(Error::InvalidConfidence);
            }
            config.min_confidence_bps = min_conf;
        }

        if let Some(enable_flag) = enabled {
            config.enabled = enable_flag;
        }

        env.storage().instance().set(&DataKey::Config, &config);
        env.events()
            .publish((symbol_short!("CfgUpdate"),), true);

        Ok(true)
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
        caller.require_auth();
        
        let config = Self::ensure_predictor(&env, &caller)?;
        
        // Validate inputs
        if predicted_value > 10_000 {
            return Err(Error::InvalidValue);
        }
        
        if confidence_bps > 10_000 {
            return Err(Error::InvalidConfidence);
        }
        
        if confidence_bps < config.min_confidence_bps {
            return Err(Error::LowConfidence);
        }
        
        if explanation_ref.is_empty() {
            panic!("explanation_ref cannot be empty");
        }

        let timestamp = env.ledger().timestamp();
        let horizon_start = timestamp;
        let horizon_end = timestamp + (config.prediction_horizon_days as u64 * 24 * 3600); // Days to seconds

        // Create prediction record
        let prediction_id = Self::next_prediction_id(&env);
        
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

        // Update patient's prediction summary
        let mut summary: PatientPredictionsSummary = env
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
        
        // Count high-risk predictions (values > 7500 bps)
        if predicted_value > 7500 {
            summary.high_risk_predictions += 1;
        }
        
        // Calculate new average confidence
        let total_conf = (summary.avg_confidence_bps as u64 * (summary.total_predictions as u64 - 1) + confidence_bps as u64) / summary.total_predictions as u64;
        summary.avg_confidence_bps = total_conf as u32;
        summary.last_prediction_date = timestamp;

        env.storage()
            .instance()
            .set(&DataKey::PatientSummary(patient.clone()), &summary);

        // Emit event
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

    pub fn get_config(env: Env) -> Option<PredictionConfig> {
        env.storage().instance().get(&DataKey::Config)
    }

    pub fn get_patient_summary(env: Env, patient: Address) -> Option<PatientPredictionsSummary> {
        env.storage()
            .instance()
            .get(&DataKey::PatientSummary(patient))
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
        let config = Self::ensure_admin(&env, &caller)?;

        // Validate metrics
        if metrics.accuracy_bps > 10_000 
            || metrics.precision_bps > 10_000 
            || metrics.recall_bps > 10_000 
            || metrics.f1_score_bps > 10_000 {
            return Err(Error::InvalidValue);
        }

        env.storage()
            .instance()
            .set(&DataKey::ModelMetrics(model_id.clone()), &metrics);

        env.events()
            .publish((symbol_short!("MdlMetric"),), model_id);

        Ok(true)
    }

    pub fn has_high_risk_prediction(env: Env, patient: Address) -> bool {
        let summary: Option<PatientPredictionsSummary> = env
            .storage()
            .instance()
            .get(&DataKey::PatientSummary(patient));

        match summary {
            Some(s) => s.high_risk_predictions > 0,
            None => false,
        }
    }

    pub fn whitelist_predictor(env: Env, caller: Address, predictor_addr: Address) -> Result<bool, Error> {
        caller.require_auth();
        let config = Self::ensure_admin(&env, &caller)?;

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
}

#[cfg(all(test, feature = "testutils"))]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_prediction_flow() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize contract with 30-day prediction horizon and 5000 bps (50%) min confidence
        client.mock_all_auths().initialize(&admin, &predictor, &30u32, &5000u32);

        // Verify config
        let config = client.get_config().unwrap();
        assert_eq!(config.admin, admin);
        assert_eq!(config.predictor, predictor);
        assert_eq!(config.prediction_horizon_days, 30u32);
        assert_eq!(config.min_confidence_bps, 5000u32);
        assert_eq!(config.enabled, true);

        // Make a prediction
        let model_id = BytesN::from_array(&env, &[1; 32]);
        let outcome_type = String::from_str(&env, "diabetes_risk");
        let features = vec![
            &env,
            String::from_str(&env, "age"),
            String::from_str(&env, "bmi"),
            String::from_str(&env, "family_history"),
        ];
        let explanation_ref = String::from_str(&env, "ipfs://prediction-explanation-123");
        let risk_factors = vec![
            &env,
            String::from_str(&env, "high_bmi"),
            String::from_str(&env, "family_history"),
        ];

        let prediction_id = client.mock_all_auths().make_prediction(
            &predictor,
            &patient,
            &model_id,
            &outcome_type,
            &7500u32,  // High risk (75%)
            &8000u32,  // High confidence (80%)
            &features,
            &explanation_ref,
            &risk_factors,
        ).unwrap();

        assert_eq!(prediction_id, 1u64);

        // Get the prediction record
        let prediction = client.get_prediction(&prediction_id).unwrap();
        assert_eq!(prediction.patient, patient);
        assert_eq!(prediction.predicted_value, 7500u32);
        assert_eq!(prediction.confidence_bps, 8000u32);
        assert_eq!(prediction.outcome_type, outcome_type);

        // Check patient summary
        let summary = client.get_patient_summary(&patient).unwrap();
        assert_eq!(summary.latest_prediction_id, 1u64);
        assert_eq!(summary.total_predictions, 1u32);
        assert_eq!(summary.high_risk_predictions, 1u32);  // Since 7500 > 7500 threshold
        assert_eq!(summary.avg_confidence_bps, 8000u32);
    }

    #[test]
    fn test_low_confidence_rejection() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize with high minimum confidence
        client.mock_all_auths().initialize(&admin, &predictor, &30u32, &9000u32);

        // Attempt to make a prediction with low confidence - should fail
        let model_id = BytesN::from_array(&env, &[1; 32]);
        let outcome_type = String::from_str(&env, "diabetes_risk");
        let features = vec![&env, String::from_str(&env, "age")];
        let explanation_ref = String::from_str(&env, "ipfs://low-confidence-prediction");
        let risk_factors = vec![&env, String::from_str(&env, "age")];

        let result = client.mock_all_auths().make_prediction(
            &predictor,
            &patient,
            &model_id,
            &outcome_type,
            &5000u32,
            &4000u32,  // Below minimum confidence of 9000
            &features,
            &explanation_ref,
            &risk_factors,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_config_updates() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);

        // Initialize contract
        client.mock_all_auths().initialize(&admin, &predictor, &30u32, &5000u32);

        // Update config
        assert!(client.mock_all_auths().update_config(
            &admin,
            Some(Address::generate(&env)),  // new predictor
            Some(60u32),                   // new horizon
            Some(7000u32),                 // new min confidence
            Some(false),                   // disable
        ).is_ok());

        let config = client.get_config().unwrap();
        assert_eq!(config.prediction_horizon_days, 60u32);
        assert_eq!(config.min_confidence_bps, 7000u32);
        assert_eq!(config.enabled, false);
    }

    #[test]
    fn test_has_high_risk_prediction_helper() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        client.mock_all_auths().initialize(&admin, &predictor, &30u32, &5000u32);

        // Initially there should be no high-risk predictions
        assert_eq!(client.has_high_risk_prediction(&patient), false);

        let model_id = BytesN::from_array(&env, &[1; 32]);
        let outcome_type = String::from_str(&env, "diabetes_risk");
        let features = vec![&env, String::from_str(&env, "age")];
        let explanation_ref = String::from_str(&env, "ipfs://prediction-explanation");
        let risk_factors = vec![&env, String::from_str(&env, "high_bmi")];

        // Create a high-risk prediction (>7500 bps)
        client.mock_all_auths().make_prediction(
            &predictor,
            &patient,
            &model_id,
            &outcome_type,
            &8000u32,
            &9000u32,
            &features,
            &explanation_ref,
            &risk_factors,
        ).unwrap();

        assert_eq!(client.has_high_risk_prediction(&patient), true);
    }
}
