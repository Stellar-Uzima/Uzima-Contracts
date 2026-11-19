#![cfg(all(test, feature = "testutils"))]
#![allow(clippy::unwrap_used)]

use crate::{
    InterventionRecommendation, PatientRiskStratificationContract,
    PatientRiskStratificationContractClient, RiskFactor, RiskModelType,
};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, BytesN, Env, String, Vec};

fn setup(
    env: &Env,
) -> (
    PatientRiskStratificationContractClient<'_>,
    Address,
    Address,
    BytesN<32>,
) {
    let contract_id = env.register_contract(None, PatientRiskStratificationContract);
    let client = PatientRiskStratificationContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let assessor = Address::generate(env);
    let patient = Address::generate(env);
    let model_id = BytesN::from_array(env, &[1u8; 32]);

    client.mock_all_auths().initialize(&admin);
    client.register_risk_model(
        &admin,
        &model_id,
        &RiskModelType::Readmission,
        &String::from_str(env, "cardiology"),
        &String::from_str(env, "v1.0"),
        &8000,
        &String::from_str(env, "Readmission risk model"),
    );

    (client, assessor, patient, model_id)
}

#[test]
fn test_risk_assessment_flow() {
    let env = Env::default();
    let (client, assessor, patient, model_id) = setup(&env);

    let risk_factors = vec![
        &env,
        RiskFactor {
            factor_name: String::from_str(&env, "age"),
            contribution_bps: 500,
            importance_bps: 800,
            category: String::from_str(&env, "demographic"),
            explanation: String::from_str(&env, "Age over 65 increases risk"),
        },
    ];
    let interventions = vec![
        &env,
        InterventionRecommendation {
            intervention_type: String::from_str(&env, "follow_up"),
            priority: 3,
            description: String::from_str(&env, "Schedule follow-up appointment"),
            expected_impact_bps: 300,
            timeframe_days: 7,
            resources_needed: vec![&env, String::from_str(&env, "nurse")],
        },
    ];

    let assessment_id = client.mock_all_auths().perform_risk_assessment(
        &assessor,
        &patient,
        &model_id,
        &6500,
        &8500,
        &30,
        &risk_factors,
        &interventions,
        &8700,
    );

    assert_eq!(assessment_id, 1);
    let assessment = client.get_risk_assessment(&assessment_id).unwrap();
    assert_eq!(assessment.patient, patient);
    assert_eq!(assessment.risk_score_bps, 6500);
    assert_eq!(assessment.confidence_bps, 8500);

    let profile = client.get_patient_risk_profile(&patient).unwrap();
    assert_eq!(profile.current_risk_level, String::from_str(&env, "high"));
    assert_eq!(profile.total_assessments, 1);
}

#[test]
fn test_risk_level_boundaries_and_monotonicity() {
    let env = Env::default();
    let (client, assessor, patient, model_id) = setup(&env);
    let risk_factors: Vec<RiskFactor> = Vec::new(&env);
    let interventions: Vec<InterventionRecommendation> = Vec::new(&env);
    let cases = [
        (2499u32, "low"),
        (2500u32, "medium"),
        (4999u32, "medium"),
        (5000u32, "high"),
        (7499u32, "high"),
        (7500u32, "critical"),
        (10000u32, "critical"),
    ];

    for (index, (score, expected_level)) in cases.iter().enumerate() {
        client.mock_all_auths().perform_risk_assessment(
            &assessor,
            &patient,
            &model_id,
            score,
            &9500,
            &30,
            &risk_factors,
            &interventions,
            &9000,
        );

        let profile = client.get_patient_risk_profile(&patient).unwrap();
        assert_eq!(
            profile.current_risk_level,
            String::from_str(&env, expected_level)
        );
        assert_eq!(profile.total_assessments, (index + 1) as u32);
    }
}

#[test]
fn test_invalid_and_missing_assessment_inputs() {
    let env = Env::default();
    let (client, assessor, patient, model_id) = setup(&env);
    let risk_factors: Vec<RiskFactor> = Vec::new(&env);
    let interventions: Vec<InterventionRecommendation> = Vec::new(&env);
    let missing_model = BytesN::from_array(&env, &[9u8; 32]);

    assert!(client
        .try_perform_risk_assessment(
            &assessor,
            &patient,
            &missing_model,
            &5000,
            &9500,
            &30,
            &risk_factors,
            &interventions,
            &9000,
        )
        .is_err());

    assert!(client
        .try_perform_risk_assessment(
            &assessor,
            &patient,
            &model_id,
            &5000,
            &7999,
            &30,
            &risk_factors,
            &interventions,
            &9000,
        )
        .is_err());

    assert!(client
        .try_perform_risk_assessment(
            &assessor,
            &patient,
            &model_id,
            &10_001,
            &9500,
            &30,
            &risk_factors,
            &interventions,
            &9000,
        )
        .is_err());
}

#[test]
fn test_multiple_models() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PatientRiskStratificationContract);
    let client = PatientRiskStratificationContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.mock_all_auths().initialize(&admin);

    let readmission_model = BytesN::from_array(&env, &[1u8; 32]);
    let mortality_model = BytesN::from_array(&env, &[2u8; 32]);
    let complications_model = BytesN::from_array(&env, &[3u8; 32]);

    client.register_risk_model(
        &admin,
        &readmission_model,
        &RiskModelType::Readmission,
        &String::from_str(&env, "general"),
        &String::from_str(&env, "v1.0"),
        &500,
        &String::from_str(&env, "Readmission model"),
    );
    client.register_risk_model(
        &admin,
        &mortality_model,
        &RiskModelType::Mortality,
        &String::from_str(&env, "cardiology"),
        &String::from_str(&env, "v1.0"),
        &500,
        &String::from_str(&env, "Mortality model"),
    );
    client.register_risk_model(
        &admin,
        &complications_model,
        &RiskModelType::Complications,
        &String::from_str(&env, "surgery"),
        &String::from_str(&env, "v1.0"),
        &500,
        &String::from_str(&env, "Complications model"),
    );

    assert_eq!(
        client.get_risk_model(&readmission_model).unwrap().model_type,
        RiskModelType::Readmission
    );
    assert_eq!(
        client.get_risk_model(&mortality_model).unwrap().model_type,
        RiskModelType::Mortality
    );
    assert_eq!(
        client
            .get_risk_model(&complications_model)
            .unwrap()
            .model_type,
        RiskModelType::Complications
    );
}
