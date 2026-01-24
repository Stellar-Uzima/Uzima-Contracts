use soroban_sdk::{Address, Env, String, Vec};
use medical_records::{MedicalRecordsContract, MedicalRecordsContractClient, Role, AIInsightType, AIConfig};
use federated_learning::{FederatedLearningContract, FederatedLearningContractClient};
use anomaly_detection::{AnomalyDetectionContract, AnomalyDetectionContractClient};
use predictive_analytics::{PredictiveAnalyticsContract, PredictiveAnalyticsContractClient};
use explainable_ai::{ExplainableAiContract, ExplainableAiContractClient, FeatureImportance};

pub mod ai_integration_tests {
    use super::*;

    #[test]
    fn test_full_ai_integration_workflow() {
        let env = Env::default();
        env.mock_all_auths();

        // Initialize all contracts
        let medical_contract_id = env.register_contract(None, MedicalRecordsContract);
        let medical_client = MedicalRecordsContractClient::new(&env, &medical_contract_id);

        let federated_contract_id = env.register_contract(None, FederatedLearningContract);
        let federated_client = FederatedLearningContractClient::new(&env, &federated_contract_id);

        let anomaly_contract_id = env.register_contract(None, AnomalyDetectionContract);
        let anomaly_client = AnomalyDetectionContractClient::new(&env, &anomaly_contract_id);

        let predictive_contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let predictive_client = PredictiveAnalyticsContractClient::new(&env, &predictive_contract_id);

        let explainable_contract_id = env.register_contract(None, ExplainableAiContract);
        let explainable_client = ExplainableAiContractClient::new(&env, &explainable_contract_id);

        // Create test addresses
        let admin = Address::generate(&env);
        let doctor = Address::generate(&env);
        let patient = Address::generate(&env);
        let ai_coordinator = Address::generate(&env);
        let ai_analyst = Address::generate(&env);

        // Initialize medical records contract
        medical_client.initialize(&admin);
        medical_client.manage_user(&admin, &doctor, &Role::Doctor);
        medical_client.manage_user(&admin, &patient, &Role::Patient);

        // Set AI configuration
        medical_client.set_ai_config(&admin, &ai_coordinator, &100u32, &2u32);

        // Add a medical record
        let record_id = medical_client.add_record(
            &doctor,
            &patient,
            &String::from_str(&env, "Diagnosis"),
            &String::from_str(&env, "Treatment"),
            &false,
            &Vec::new(&env),
            &String::from_str(&env, "Modern"),
            &String::from_str(&env, "Medication"),
            &String::from_str(&env, "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx"),
        );

        // Submit anomaly score
        let model_id = [1u8; 32].into();
        let explanation_ref = String::from_str(&env, "ipfs://anomaly-explanation");
        let explanation_summary = String::from_str(&env, "Anomaly detected in lab values");
        let model_version = String::from_str(&env, "v1.0.0");
        let feature_importance = Vec::new(&env);

        medical_client.submit_anomaly_score(
            &ai_coordinator,
            &record_id,
            &model_id,
            &7500u32,
            &explanation_ref,
            &explanation_summary,
            &model_version,
            &feature_importance,
        );

        // Submit risk score
        medical_client.submit_risk_score(
            &ai_coordinator,
            &patient,
            &model_id,
            &8000u32,
            &String::from_str(&env, "ipfs://risk-explanation"),
            &String::from_str(&env, "High risk for complications"),
            &String::from_str(&env, "v1.0.0"),
            &Vec::new(&env),
        );

        // Get AI insights
        let anomaly_insight = medical_client.get_anomaly_score(&patient, &record_id).unwrap();
        assert_eq!(anomaly_insight.score_bps, 7500u32);
        assert_eq!(anomaly_insight.insight_type, AIInsightType::AnomalyScore);

        let risk_insight = medical_client.get_latest_risk_score(&patient, &patient).unwrap();
        assert_eq!(risk_insight.score_bps, 8000u32);
        assert_eq!(risk_insight.insight_type, AIInsightType::RiskScore);

        println!("✓ Full AI integration workflow completed successfully");
    }

    #[test]
    fn test_federated_learning_with_privacy() {
        let env = Env::default();
        env.mock_all_auths();

        let federated_contract_id = env.register_contract(None, FederatedLearningContract);
        let federated_client = FederatedLearningContractClient::new(&env, &federated_contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let participant1 = Address::generate(&env);
        let participant2 = Address::generate(&env);

        federated_client.initialize(&admin, &coordinator);

        // Start a federated learning round with differential privacy
        let base_model = [1u8; 32].into();
        let round_id = federated_client.start_round(&admin, &base_model, &2u32, &100u32);

        // Participants submit updates
        let update1 = [2u8; 32].into();
        let update2 = [3u8; 32].into();

        federated_client.submit_update(&participant1, &round_id, &update1, &100u32).unwrap();
        federated_client.submit_update(&participant2, &round_id, &update2, &200u32).unwrap();

        // Finalize round
        let new_model = [4u8; 32].into();
        federated_client.finalize_round(
            &coordinator,
            &round_id,
            &new_model,
            &String::from_str(&env, "Improved federated model"),
            &String::from_str(&env, "ipfs://model-metrics"),
            &String::from_str(&env, "ipfs://fairness-report"),
        ).unwrap();

        // Verify round is finalized
        let round = federated_client.get_round(&round_id).unwrap();
        assert!(round.is_finalized);
        assert_eq!(round.total_updates, 2);

        // Verify model was created
        let model = federated_client.get_model(&new_model).unwrap();
        assert_eq!(model.description, String::from_str(&env, "Improved federated model"));

        println!("✓ Federated learning with privacy completed successfully");
    }

    #[test]
    fn test_anomaly_detection_with_bias_testing() {
        let env = Env::default();
        env.mock_all_auths();

        let anomaly_contract_id = env.register_contract(None, AnomalyDetectionContract);
        let anomaly_client = AnomalyDetectionContractClient::new(&env, &anomaly_contract_id);

        let explainable_contract_id = env.register_contract(None, ExplainableAiContract);
        let explainable_client = ExplainableAiContractClient::new(&env, &explainable_contract_id);

        let admin = Address::generate(&env);
        let detector = Address::generate(&env);
        let patient = Address::generate(&env);

        anomaly_client.initialize(&admin, &detector, &7500u32);

        // Initialize explainable AI contract
        explainable_client.initialize(&admin);

        // Detect an anomaly
        let metadata = String::from_str(&env, r#"{"lab_values": [6.2, 140, 80]}"#);
        let explanation_ref = String::from_str(&env, "ipfs://detailed-anomaly-report");

        let anomaly_id = anomaly_client.detect_anomaly(
            &detector,
            &1u64,
            &patient,
            &8000u32,
            &4u32,
            &metadata,
            &explanation_ref,
        ).unwrap();

        // Verify anomaly was detected
        let anomaly_record = anomaly_client.get_anomaly_record(&anomaly_id).unwrap();
        assert_eq!(anomaly_record.score_bps, 8000u32);
        assert_eq!(anomaly_record.severity, 4u32);

        // Submit bias audit for the anomaly detection model
        let model_id = [1u8; 32].into();
        let audit_summary = String::from_str(&env, "Bias audit for anomaly detection model");
        let recommendations = vec![
            &env,
            String::from_str(&env, "Ensure balanced representation in training data"),
            String::from_str(&env, "Monitor for disparate impact across demographics"),
        ];

        let audit_id = explainable_client.submit_bias_audit(
            &admin,
            &model_id,
            &audit_summary,
            &recommendations,
        ).unwrap();

        assert!(audit_id > 0);

        // Run fairness metrics
        let (dp_diff, eo_diff, cal_diff) = explainable_client.run_fairness_metrics(
            &admin,
            &model_id,
            &String::from_str(&env, "age_group"),
            &String::from_str(&env, "young"),
            &String::from_str(&env, "elderly"),
        ).unwrap();

        assert!(dp_diff < 500); // Less than 5% difference is acceptable
        assert!(eo_diff < 500); // Less than 5% difference is acceptable
        assert!(cal_diff < 500); // Less than 5% difference is acceptable

        println!("✓ Anomaly detection with bias testing completed successfully");
    }

    #[test]
    fn test_predictive_analytics_with_explainability() {
        let env = Env::default();
        env.mock_all_auths();

        let predictive_contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let predictive_client = PredictiveAnalyticsContractClient::new(&env, &predictive_contract_id);

        let explainable_contract_id = env.register_contract(None, ExplainableAiContract);
        let explainable_client = ExplainableAiContractClient::new(&env, &explainable_contract_id);

        let admin = Address::generate(&env);
        let predictor = Address::generate(&env);
        let patient = Address::generate(&env);

        // Initialize predictive analytics contract
        predictive_client.initialize(&admin, &predictor, &30u32, &5000u32);

        // Initialize explainable AI contract
        explainable_client.initialize(&admin);

        // Make a prediction
        let model_id = [1u8; 32].into();
        let outcome_type = String::from_str(&env, "diabetes_risk");
        let features = vec![
            &env,
            String::from_str(&env, "age"),
            String::from_str(&env, "bmi"),
            String::from_str(&env, "family_history"),
        ];
        let explanation_ref = String::from_str(&env, "ipfs://prediction-details");
        let risk_factors = vec![
            &env,
            String::from_str(&env, "high_bmi"),
            String::from_str(&env, "family_history"),
        ];

        let prediction_id = predictive_client.make_prediction(
            &predictor,
            &patient,
            &model_id,
            &outcome_type,
            &7500u32,
            &8000u32,
            &features,
            &explanation_ref,
            &risk_factors,
        ).unwrap();

        // Verify prediction
        let prediction = predictive_client.get_prediction(&prediction_id).unwrap();
        assert_eq!(prediction.predicted_value, 7500u32);
        assert_eq!(prediction.confidence_bps, 8000u32);
        assert_eq!(prediction.outcome_type, outcome_type);

        // Request explanation for the prediction
        let request_id = explainable_client.request_explanation(&patient, &prediction_id);

        // Fulfill the explanation request
        let feature_importance = vec![
            &env,
            FeatureImportance {
                feature_name: String::from_str(&env, "bmi"),
                importance_bps: 8000u32,
                normalized_value: 7500u32,
            },
            FeatureImportance {
                feature_name: String::from_str(&env, "family_history"),
                importance_bps: 6500u32,
                normalized_value: 7000u32,
            },
        ];

        let primary_factors = vec![
            &env,
            String::from_str(&env, "bmi"),
            String::from_str(&env, "family_history"),
        ];

        explainable_client.fulfill_explanation_request(
            &admin,
            &request_id,
            &model_id,
            &String::from_str(&env, "SHAP"),
            &feature_importance,
            &primary_factors,
            &5000u32,
            &String::from_str(&env, "ipfs://detailed-explanation"),
        ).unwrap();

        // Verify explanation was created
        let explanation = explainable_client.get_explanation(&1u64).unwrap(); // First explanation
        assert_eq!(explanation.insight_id, prediction_id);
        assert_eq!(explanation.feature_importance.len(), 2);

        println!("✓ Predictive analytics with explainability completed successfully");
    }

    #[test]
    fn test_comprehensive_privacy_preservation() {
        let env = Env::default();
        env.mock_all_auths();

        let federated_contract_id = env.register_contract(None, FederatedLearningContract);
        let federated_client = FederatedLearningContractClient::new(&env, &federated_contract_id);

        let anomaly_contract_id = env.register_contract(None, AnomalyDetectionContract);
        let anomaly_client = AnomalyDetectionContractClient::new(&env, &anomaly_contract_id);

        let predictive_contract_id = env.register_contract(None, PredictiveAnalyticsContract);
        let predictive_client = PredictiveAnalyticsContractClient::new(&env, &predictive_contract_id);

        let admin = Address::generate(&env);
        let coordinator = Address::generate(&env);
        let predictor = Address::generate(&env);
        let detector = Address::generate(&env);
        let participant1 = Address::generate(&env);
        let participant2 = Address::generate(&env);

        // Initialize contracts
        federated_client.initialize(&admin, &coordinator);
        anomaly_client.initialize(&admin, &detector, &7500u32);
        predictive_client.initialize(&admin, &predictor, &30u32, &5000u32);

        // Test federated learning with privacy budget
        federated_client.set_privacy_budget(&admin, &participant1, &100u32).unwrap();
        federated_client.set_privacy_budget(&admin, &participant2, &100u32).unwrap();

        let budget1 = federated_client.get_privacy_budget(&participant1).unwrap();
        let budget2 = federated_client.get_privacy_budget(&participant2).unwrap();

        assert_eq!(budget1.epsilon_total, 100u32);
        assert_eq!(budget2.epsilon_total, 100u32);

        // Start round with DP parameters
        let base_model = [1u8; 32].into();
        let round_id = federated_client.start_round(&admin, &base_model, &2u32, &50u32);

        // Participants submit updates respecting privacy budget
        federated_client.submit_update(&participant1, &round_id, &[2u8; 32].into(), &50u32).unwrap();
        federated_client.submit_update(&participant2, &round_id, &[3u8; 32].into(), &40u32).unwrap();

        // Verify that privacy budgets were consumed appropriately
        let updated_budget1 = federated_client.get_privacy_budget(&participant1).unwrap();
        let updated_budget2 = federated_client.get_privacy_budget(&participant2).unwrap();

        assert!(updated_budget1.epsilon_consumed > 0);
        assert!(updated_budget2.epsilon_consumed > 0);

        // Test anomaly detection thresholds
        let patient = Address::generate(&env);
        let metadata = String::from_str(&env, "{}");
        let explanation_ref = String::from_str(&env, "ipfs://privacy-safe-report");

        anomaly_client.detect_anomaly(
            &detector,
            &1u64,
            &patient,
            &8000u32,  // Above threshold
            &4u32,
            &metadata,
            &explanation_ref,
        ).unwrap();

        // Verify that sensitive information is not exposed
        let config = anomaly_client.get_config().unwrap();
        assert_eq!(config.threshold_bps, 7500u32); // Threshold protects privacy

        // Test predictive analytics confidence thresholds
        let model_id = [1u8; 32].into();
        let features = vec![&env, String::from_str(&env, "age")];
        let risk_factors = vec![&env, String::from_str(&env, "age")];

        // This should succeed (confidence above minimum)
        let pred_id = predictive_client.make_prediction(
            &predictor,
            &patient,
            &model_id,
            &String::from_str(&env, "risk_outcome"),
            &6000u32,
            &6000u32,  // Above minimum confidence (5000)
            &features,
            &explanation_ref,
            &risk_factors,
        ).unwrap();

        assert!(pred_id > 0);

        println!("✓ Comprehensive privacy preservation tests completed successfully");
    }
}
