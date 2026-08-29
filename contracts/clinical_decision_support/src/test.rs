// Regression tests for clinical_decision_support
// Verifies: initialization, a representative happy-path workflow, and authorization
// failures that downstream callers can observe.
//
// Run with:
//   cargo test --manifest-path contracts/clinical_decision_support/Cargo.toml

use super::*;
use soroban_sdk::testutils::Address as _;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Boot a fresh contract and return (client, admin, oracle, medical_records).
fn setup(env: &Env) -> (ClinicalDecisionSupportClient, Address, Address, Address) {
    let contract_id = env.register_contract(None, ClinicalDecisionSupport);
    let client = ClinicalDecisionSupportClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let medical_records = Address::generate(env);

    env.mock_all_auths();
    client.initialize(&admin, &oracle, &medical_records);

    (client, admin, oracle, medical_records)
}

// ── Init tests ────────────────────────────────────────────────────────────────

/// Single-initialization guard: a second call to `initialize` must panic.
#[test]
#[should_panic]
fn test_initialize_can_only_be_called_once() {
    let env = Env::default();
    let (client, admin, oracle, medical_records) = setup(&env);

    // Second call should be rejected by governance_commons::init_guard
    client.initialize(&admin, &oracle, &medical_records);
}

/// Admin address is persisted correctly after initialization.
#[test]
fn test_initialize_stores_admin() {
    let env = Env::default();
    let (client, admin, _oracle, _medical_records) = setup(&env);

    let drug_a = String::from_str(&env, "DrugA");
    let drug_b = String::from_str(&env, "DrugB");
    let severity = String::from_str(&env, "Major");

    // If admin was not stored correctly this call would panic with "Unauthorized admin"
    env.mock_all_auths();
    client.set_interaction(&admin, &drug_a, &drug_b, &severity);
}

// ── Happy-path tests ──────────────────────────────────────────────────────────

/// check_drug_interactions: returns one alert when a known interaction is present.
#[test]
fn test_drug_interaction_detected() {
    let env = Env::default();
    let (client, admin, _oracle, _medical_records) = setup(&env);

    let drug_a = String::from_str(&env, "Rx001");
    let drug_b = String::from_str(&env, "Rx002");
    let severity = String::from_str(&env, "Critical: Risk of Serotonin Syndrome");

    env.mock_all_auths();
    client.set_interaction(&admin, &drug_a, &drug_b, &severity);

    let mut current_meds = Vec::new(&env);
    current_meds.push_back(drug_a.clone());

    let alerts = client.check_drug_interactions(
        &String::from_str(&env, "PAT-001"),
        &drug_b,
        &current_meds,
    );

    assert_eq!(alerts.len(), 1);
    let alert = alerts.get(0).unwrap();
    assert_eq!(alert.rec_type, RecommendationType::DrugInteraction);
    assert_eq!(alert.content, severity);
    assert_eq!(alert.urgency, 2); // Critical
}

/// check_drug_interactions: returns empty when no interaction is registered.
#[test]
fn test_no_drug_interaction_returns_empty() {
    let env = Env::default();
    let (client, _admin, _oracle, _medical_records) = setup(&env);

    let mut current_meds = Vec::new(&env);
    current_meds.push_back(String::from_str(&env, "SafeDrug"));

    let alerts = client.check_drug_interactions(
        &String::from_str(&env, "PAT-002"),
        &String::from_str(&env, "OtherDrug"),
        &current_meds,
    );

    assert_eq!(alerts.len(), 0);
}

/// get_treatment_recommendation: returns a recommendation when a guideline is stored.
#[test]
fn test_treatment_recommendation_returned() {
    let env = Env::default();
    let (client, _admin, oracle, _medical_records) = setup(&env);

    let guideline = ClinicalGuideline {
        condition_code: String::from_str(&env, "C001"),
        recommended_action: String::from_str(&env, "Start metformin therapy"),
        evidence_level: String::from_str(&env, "A"),
        min_confidence: 9000,
    };

    env.mock_all_auths();
    client.update_guideline(&oracle, &guideline);

    let mut codes = Vec::new(&env);
    codes.push_back(String::from_str(&env, "C001"));

    let recs = client.get_treatment_recommendation(
        &String::from_str(&env, "PAT-003"),
        &codes,
    );

    assert_eq!(recs.len(), 1);
    let rec = recs.get(0).unwrap();
    assert_eq!(rec.rec_type, RecommendationType::TreatmentOptimization);
    assert_eq!(rec.content, String::from_str(&env, "Start metformin therapy"));
    assert!(rec.confidence_score >= 9500);
}

/// optimize_pathway: declining vitals trigger Critical urgency escalation.
#[test]
fn test_optimize_pathway_escalates_on_declining_vitals() {
    let env = Env::default();
    let (client, _admin, _oracle, _medical_records) = setup(&env);

    let rec = client.optimize_pathway(
        &String::from_str(&env, "PAT-004"),
        &3,
        &(-1_i32),
    );

    assert_eq!(rec.urgency, 2); // Critical
    assert_eq!(rec.rec_type, RecommendationType::PathwayAdjustment);
}

/// record_outcome: persists without panicking and learning factor influences
/// a subsequent recommendation.
#[test]
fn test_record_outcome_and_learning() {
    let env = Env::default();
    let (client, _admin, oracle, _medical_records) = setup(&env);

    let guideline = ClinicalGuideline {
        condition_code: String::from_str(&env, "C002"),
        recommended_action: String::from_str(&env, "Refer to specialist"),
        evidence_level: String::from_str(&env, "B"),
        min_confidence: 8000,
    };

    env.mock_all_auths();
    client.update_guideline(&oracle, &guideline);

    // Record 15 successful outcomes to trigger the learning factor
    for _ in 0..15 {
        client.record_outcome(&String::from_str(&env, "C002"), &true);
    }

    let mut codes = Vec::new(&env);
    codes.push_back(String::from_str(&env, "C002"));

    let recs = client.get_treatment_recommendation(
        &String::from_str(&env, "PAT-005"),
        &codes,
    );

    assert_eq!(recs.len(), 1);
    // With >10 successful outcomes the confidence should be boosted above baseline 9500
    assert!(recs.get(0).unwrap().confidence_score > 9500);
}

// ── Error-path tests ──────────────────────────────────────────────────────────

/// update_guideline: a caller that is not the stored oracle must be rejected.
#[test]
#[should_panic]
fn test_update_guideline_rejects_wrong_oracle() {
    let env = Env::default();
    let (client, _admin, _oracle, _medical_records) = setup(&env);

    let impostor = Address::generate(&env);
    env.mock_all_auths();

    let guideline = ClinicalGuideline {
        condition_code: String::from_str(&env, "C999"),
        recommended_action: String::from_str(&env, "Unauthorized"),
        evidence_level: String::from_str(&env, "X"),
        min_confidence: 0,
    };

    // Should panic: "Unauthorized oracle"
    client.update_guideline(&impostor, &guideline);
}

/// set_interaction: a caller that is not the stored admin must be rejected.
#[test]
#[should_panic]
fn test_set_interaction_rejects_wrong_admin() {
    let env = Env::default();
    let (client, _admin, _oracle, _medical_records) = setup(&env);

    let impostor = Address::generate(&env);
    env.mock_all_auths();

    // Should panic: "Unauthorized admin"
    client.set_interaction(
        &impostor,
        &String::from_str(&env, "DrugA"),
        &String::from_str(&env, "DrugB"),
        &String::from_str(&env, "Minor"),
    );
}
