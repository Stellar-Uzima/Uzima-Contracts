use crate::{
    types::{HealthPrediction, PatientPredictionsSummary, PredictionConfig},
    PredictiveAnalyticsContract, PredictiveAnalyticsContractClient,
};
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env, String};

#[test]
fn test_prediction_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
    let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let predictor = Address::generate(&env);
    let patient = Address::generate(&env);

    client
        .mock_all_auths()
        .initialize(&admin, &predictor, &30u32, &5000u32);

    let config: PredictionConfig = client.get_config().unwrap();
    assert_eq!(config.admin, admin);
    assert_eq!(config.predictor, predictor);
    assert_eq!(config.prediction_horizon_days, 30u32);
    assert_eq!(config.min_confidence_bps, 5000u32);
    assert!(config.enabled);

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
        &7500u32,
        &8000u32,
        &features,
        &explanation_ref,
        &risk_factors,
    );

    assert_eq!(prediction_id, 1u64);

    let prediction: HealthPrediction = client.get_prediction(&prediction_id).unwrap();
    assert_eq!(prediction.patient, patient);
    assert_eq!(prediction.predicted_value, 7500u32);
    assert_eq!(prediction.confidence_bps, 8000u32);
    assert_eq!(prediction.outcome_type, outcome_type);

    let summary: PatientPredictionsSummary = client.get_patient_summary(&patient).unwrap();
    assert_eq!(summary.latest_prediction_id, 1u64);
    assert_eq!(summary.total_predictions, 1u32);
    assert_eq!(summary.high_risk_predictions, 1u32);
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

    client
        .mock_all_auths()
        .initialize(&admin, &predictor, &30u32, &9000u32);

    let model_id = BytesN::from_array(&env, &[1; 32]);
    let outcome_type = String::from_str(&env, "diabetes_risk");
    let features = vec![&env, String::from_str(&env, "age")];
    let explanation_ref = String::from_str(&env, "ipfs://low-confidence-prediction");
    let risk_factors = vec![&env, String::from_str(&env, "age")];

    let result = client.mock_all_auths().try_make_prediction(
        &predictor,
        &patient,
        &model_id,
        &outcome_type,
        &5000u32,
        &4000u32,
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

    client
        .mock_all_auths()
        .initialize(&admin, &predictor, &30u32, &5000u32);

    assert!(client.mock_all_auths().update_config(
        &admin,
        &Some(Address::generate(&env)),
        &Some(60u32),
        &Some(7000u32),
        &Some(false),
    ));

    let config: PredictionConfig = client.get_config().unwrap();
    assert_eq!(config.prediction_horizon_days, 60u32);
    assert_eq!(config.min_confidence_bps, 7000u32);
    assert!(!config.enabled);
}

#[test]
fn test_has_high_risk_prediction_helper() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PredictiveAnalyticsContract);
    let client = PredictiveAnalyticsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let predictor = Address::generate(&env);
    let patient = Address::generate(&env);

    client
        .mock_all_auths()
        .initialize(&admin, &predictor, &30u32, &5000u32);

    assert!(!client.has_high_risk_prediction(&patient));

    let model_id = BytesN::from_array(&env, &[1; 32]);
    let outcome_type = String::from_str(&env, "diabetes_risk");
    let features = vec![&env, String::from_str(&env, "age")];
    let explanation_ref = String::from_str(&env, "ipfs://prediction-explanation");
    let risk_factors = vec![&env, String::from_str(&env, "high_bmi")];

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
    );

    assert!(client.has_high_risk_prediction(&patient));
}
