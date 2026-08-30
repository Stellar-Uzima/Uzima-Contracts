//! Regression tests for `explainable_ai` (issue #1430).
//!
//! ## Fixtures / cross-contract calls
//!
//! `ExplainableAiContract` makes no cross-contract calls and depends only on
//! the in-workspace `governance_commons` library (a plain Rust lib, not a
//! deployed contract) for its re-initialization guard. No mock/fixture
//! contract is required to exercise any entry point below — every test
//! deploys a single `ExplainableAiContract` instance via
//! `env.register_contract` and drives it directly through the generated
//! `ExplainableAiContractClient`.

use super::*;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::vec;

fn create_contract(env: &Env) -> ExplainableAiContractClient<'_> {
    let contract_id = env.register_contract(None, ExplainableAiContract);
    ExplainableAiContractClient::new(env, &contract_id)
}

fn sample_feature_importance(env: &Env, importance_bps: u32) -> Vec<FeatureImportance> {
    vec![
        env,
        FeatureImportance {
            feature_name: String::from_str(env, "age"),
            importance_bps,
            normalized_value: 7500u32,
        },
        FeatureImportance {
            feature_name: String::from_str(env, "bmi"),
            importance_bps: 6500u32,
            normalized_value: 8200u32,
        },
    ]
}

// ---------------------------------------------------------------------
// Initialization: single-init behavior + admin/auth setup
// ---------------------------------------------------------------------

/// `initialize` succeeds exactly once, requires the admin's authorization,
/// and seeds contract state (request/explanation/audit counters) so the
/// first subsequent call gets id `1`.
#[test]
fn test_initialize_requires_admin_auth_and_seeds_state() {
    let env = Env::default();
    env.mock_all_auths();

    let client = create_contract(&env);
    let admin = Address::generate(&env);

    let result = client.initialize(&admin);
    assert!(result, "initialize should return true on first call");

    // Prove counters were seeded to 0: the first request minted afterwards
    // must get id 1.
    let patient = Address::generate(&env);
    let request_id = client.request_explanation(&patient, &1u64);
    assert_eq!(request_id, 1u64);
}

/// `initialize` calls `admin.require_auth()`; without a mocked/authorized
/// signature from `admin`, the call must fail (proves auth is actually
/// enforced, not just documented).
#[test]
#[should_panic]
fn test_initialize_without_admin_auth_panics() {
    let env = Env::default();
    // Intentionally do NOT call env.mock_all_auths() / mock_auths here.
    let client = create_contract(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);
}

/// Re-initialization guard (`governance_commons::init_guard`) enforces
/// single-init semantics: a second `initialize` call on an already
/// initialized contract must panic rather than silently overwrite state.
#[test]
#[should_panic(expected = "already initialized")]
fn test_initialize_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let client = create_contract(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.initialize(&admin); // must panic: already initialized
}

// ---------------------------------------------------------------------
// Happy path: request -> fulfill explanation workflow
// ---------------------------------------------------------------------

/// Exercises the primary public workflow (`request_explanation` followed by
/// `fulfill_explanation_request`), verifying both the persisted result
/// (request status transition + stored `ExplanationMetadata`) and that a
/// completion event is emitted.
#[test]
fn test_request_and_fulfill_explanation_happy_path() {
    let env = Env::default();
    env.mock_all_auths();

    let client = create_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    client.initialize(&admin);

    let request_id = client.request_explanation(&patient, &123u64);
    assert_eq!(request_id, 1u64);

    let request = client.get_explanation_request(&request_id).unwrap();
    assert_eq!(request.status, ExplanationStatus::Pending);
    assert_eq!(request.patient, patient);

    let model_id = BytesN::from_array(&env, &[1; 32]);
    let explanation_method = String::from_str(&env, "SHAP");
    let feature_importance = sample_feature_importance(&env, 8000u32);
    let primary_factors = vec![
        &env,
        String::from_str(&env, "age"),
        String::from_str(&env, "bmi"),
    ];
    let explanation_ref = String::from_str(&env, "ipfs://explanation-details-123");

    let events_before = env.events().all().len();

    let fulfilled = client.fulfill_explanation_request(
        &admin,
        &request_id,
        &model_id,
        &explanation_method,
        &feature_importance,
        &primary_factors,
        &5000u32,
        &explanation_ref,
    );
    assert!(fulfilled);

    // Persisted result: request flips to Completed with a fulfilled_at
    // timestamp, and the explanation metadata is retrievable.
    let updated_request = client.get_explanation_request(&request_id).unwrap();
    assert_eq!(updated_request.status, ExplanationStatus::Completed);
    assert!(updated_request.fulfilled_at.is_some());

    let explanation = client.get_explanation(&1u64).unwrap();
    assert_eq!(explanation.model_id, model_id);
    assert_eq!(explanation.patient, patient);
    assert_eq!(explanation.explanation_method, explanation_method);
    assert_eq!(explanation.feature_importance.len(), 2);

    // Emitted event: both request_explanation ("ExpReq") and
    // fulfill_explanation_request ("ExpFull") publish an event.
    let events_after = env.events().all();
    assert!(
        events_after.len() > events_before,
        "fulfill_explanation_request must emit an event"
    );
}

// ---------------------------------------------------------------------
// Error path: invalid input is rejected without mutating state
// ---------------------------------------------------------------------

/// `fulfill_explanation_request` validates `importance_bps <= 10_000` for
/// every feature. An out-of-range value must return the documented
/// `Error::InvalidImportance` failure and must not mutate the pending
/// request.
#[test]
fn test_fulfill_with_invalid_importance_returns_documented_error() {
    let env = Env::default();
    env.mock_all_auths();

    let client = create_contract(&env);
    let admin = Address::generate(&env);
    let patient = Address::generate(&env);

    client.initialize(&admin);
    let request_id = client.request_explanation(&patient, &7u64);

    let model_id = BytesN::from_array(&env, &[9; 32]);
    let invalid_importance = sample_feature_importance(&env, 10_001u32); // > 10_000
    let primary_factors = vec![&env, String::from_str(&env, "age")];
    let explanation_ref = String::from_str(&env, "ipfs://bad-explanation");

    let result = client.try_fulfill_explanation_request(
        &admin,
        &request_id,
        &model_id,
        &String::from_str(&env, "SHAP"),
        &invalid_importance,
        &primary_factors,
        &1000u32,
        &explanation_ref,
    );
    assert_eq!(result, Err(Ok(Error::InvalidImportance)));

    // Invalid input must not mutate persisted state: the request stays Pending.
    let request = client.get_explanation_request(&request_id).unwrap();
    assert_eq!(request.status, ExplanationStatus::Pending);
}

// ---------------------------------------------------------------------
// Additional coverage: bias audit workflow (kept from prior inline tests)
// ---------------------------------------------------------------------

#[test]
fn test_bias_audit_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let client = create_contract(&env);
    let admin = Address::generate(&env);
    let model_id = BytesN::from_array(&env, &[1; 32]);

    client.initialize(&admin);

    let audit_summary = String::from_str(&env, "Initial bias audit for model v1.0");
    let recommendations = vec![
        &env,
        String::from_str(&env, "Collect more diverse training data"),
        String::from_str(&env, "Adjust model weights for underrepresented groups"),
    ];

    let audit_id =
        client.submit_bias_audit(&admin, &model_id, &audit_summary, &recommendations);
    assert_eq!(audit_id, 1u64);

    let audit = client.get_bias_audit(&model_id).unwrap();
    assert_eq!(audit.model_id, model_id);
    assert_eq!(audit.audit_summary, audit_summary);
    assert_eq!(audit.recommendations.len(), 2);

    let (dp_diff, eo_diff, cal_diff) = client.run_fairness_metrics(
        &admin,
        &model_id,
        &String::from_str(&env, "gender"),
        &String::from_str(&env, "male"),
        &String::from_str(&env, "female"),
    );

    assert!(dp_diff > 0);
    assert!(eo_diff > 0);
    assert!(cal_diff > 0);
}
