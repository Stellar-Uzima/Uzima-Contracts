#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, BytesN, Env, String};

fn setup(env: &Env) -> (MedicalImagingAiContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalImagingAiContract);
    let client = MedicalImagingAiContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin, &9200, &8500, &50);
    (client, admin)
}

fn hash(env: &Env, v: u8) -> BytesN<32> {
    BytesN::from_array(env, &[v; 32])
}

#[allow(dead_code)]
fn sig(env: &Env, v: u8) -> BytesN<64> {
    BytesN::from_array(env, &[v; 64])
}

fn make_model_input(env: &Env, layer_count: u32) -> CnnModelInput {
    CnnModelInput {
        architecture_hash: hash(env, 50),
        version: 1,
        layer_count,
        input_rows: 512,
        input_cols: 512,
        input_channels: 1,
        training_samples: 100_000,
        validation_accuracy_bps: 9500,
        training_dataset_hash: hash(env, 51),
        signing_pubkey: hash(env, 52),
    }
}

// ── Task 2 Tests ────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, _) = setup(&env);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.initialize(&admin, &9200, &8500, &50);
}

#[test]
fn test_pause_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.pause(&admin);
    client.unpause(&admin);
}

#[test]
fn test_register_and_revoke_evaluator() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let evaluator = Address::generate(&env);
    client.register_evaluator(&admin, &evaluator);
    client.revoke_evaluator(&admin, &evaluator);
}

// ── Task 3 Tests ────────────────────────────────────────────────────────

fn register_test_model(
    env: &Env,
    client: &MedicalImagingAiContractClient<'_>,
    caller: &Address,
    model_id_byte: u8,
) {
    client.register_cnn_model(
        caller,
        &hash(env, model_id_byte),
        &ImagingModality::CT,
        &make_model_input(env, 152),
    );
}

#[test]
fn test_register_cnn_model() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let model = client.get_model(&hash(&env, 1));
    assert_eq!(model.version, 1);
    assert_eq!(model.layer_count, 152);
    assert_eq!(model.status, ModelStatus::Active);
    assert_eq!(model.validation_accuracy_bps, 9500);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_register_duplicate_model() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    register_test_model(&env, &client, &admin, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_register_model_zero_layers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    client.register_cnn_model(
        &admin,
        &hash(&env, 1),
        &ImagingModality::CT,
        &make_model_input(&env, 0), // zero layers — invalid
    );
}

#[test]
fn test_is_model_active() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    assert!(client.is_model_active(&hash(&env, 1)));
}

#[test]
fn test_update_model_status_retire() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    client.update_model_status(&admin, &hash(&env, 1), &ModelStatus::Retired);
    assert!(!client.is_model_active(&hash(&env, 1)));
}

#[test]
fn test_is_model_active_degraded() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    client.update_model_status(&admin, &hash(&env, 1), &ModelStatus::Degraded);
    assert!(client.is_model_active(&hash(&env, 1)));
}

#[test]
fn test_update_model_status_reactivate() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    client.update_model_status(&admin, &hash(&env, 1), &ModelStatus::Retired);
    assert!(!client.is_model_active(&hash(&env, 1)));
    client.update_model_status(&admin, &hash(&env, 1), &ModelStatus::Active);
    assert!(client.is_model_active(&hash(&env, 1)));
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_initialize_warning_lte_critical() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, MedicalImagingAiContract);
    let client = MedicalImagingAiContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    // warning_bps == critical_bps — must reject with InvalidThreshold (#15)
    client.initialize(&admin, &8500, &8500, &50);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_initialize_min_samples_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = Address::generate(&env);
    env.register_contract(&contract_id, MedicalImagingAiContract);
    let client = MedicalImagingAiContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    // min_samples == 0 — must reject with InvalidInput (#5)
    client.initialize(&admin, &9200, &8500, &0);
}

// ── Task 4 Tests ────────────────────────────────────────────────────────

fn make_finding(env: &Env, id: u32) -> Finding {
    Finding {
        finding_id: id,
        condition_hash: hash(env, id as u8),
        confidence_bps: 8500,
        severity: 3,
        region: BoundingBox {
            x_min: 10,
            y_min: 20,
            x_max: 100,
            y_max: 200,
        },
        explanation_ref: String::from_str(env, "ipfs://explanation"),
    }
}

#[test]
fn test_submit_analysis() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let caller = Address::generate(&env);
    let findings = vec![&env, make_finding(&env, 1), make_finding(&env, 2)];

    let result_id = client.submit_analysis(
        &caller,
        &1u64,
        &hash(&env, 1),
        &hash(&env, 99),
        &sig(&env, 99),
        &findings,
        &9000,
        &150,
    );
    assert_eq!(result_id, 1);

    let result = client.get_analysis(&result_id);
    assert_eq!(result.findings.len(), 2);
    assert_eq!(result.overall_confidence_bps, 9000);
    assert_eq!(result.image_id, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_submit_analysis_too_many_findings() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let caller = Address::generate(&env);
    let mut findings = Vec::new(&env);
    for i in 0..21u32 {
        findings.push_back(make_finding(&env, i));
    }

    client.submit_analysis(
        &caller,
        &1u64,
        &hash(&env, 1),
        &hash(&env, 99),
        &sig(&env, 99),
        &findings,
        &9000,
        &150,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_submit_analysis_inactive_model() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);
    client.update_model_status(&admin, &hash(&env, 1), &ModelStatus::Deactivated);

    let caller = Address::generate(&env);
    let findings = vec![&env, make_finding(&env, 1)];

    client.submit_analysis(
        &caller,
        &1u64,
        &hash(&env, 1),
        &hash(&env, 99),
        &sig(&env, 99),
        &findings,
        &9000,
        &150,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_submit_analysis_invalid_bbox() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let caller = Address::generate(&env);
    let bad_finding = Finding {
        finding_id: 1,
        condition_hash: hash(&env, 1),
        confidence_bps: 8500,
        severity: 3,
        region: BoundingBox {
            x_min: 200,
            y_min: 20,
            x_max: 100, // x_min > x_max
            y_max: 200,
        },
        explanation_ref: String::from_str(&env, "ipfs://explanation"),
    };
    let findings = vec![&env, bad_finding];

    client.submit_analysis(
        &caller,
        &1u64,
        &hash(&env, 1),
        &hash(&env, 99),
        &sig(&env, 99),
        &findings,
        &9000,
        &150,
    );
}

#[test]
fn test_get_image_analyses() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let caller = Address::generate(&env);
    let findings = vec![&env, make_finding(&env, 1)];

    client.submit_analysis(
        &caller,
        &42u64,
        &hash(&env, 1),
        &hash(&env, 99),
        &sig(&env, 99),
        &findings,
        &9000,
        &150,
    );
    client.submit_analysis(
        &caller,
        &42u64,
        &hash(&env, 1),
        &hash(&env, 98),
        &sig(&env, 98),
        &findings,
        &8500,
        &200,
    );

    let ids = client.get_image_analyses(&42u64);
    assert_eq!(ids.len(), 2);
}

// ── Task 5 Tests ────────────────────────────────────────────────────────

fn make_region(env: &Env, id: u8) -> SegmentedRegion {
    SegmentedRegion {
        label_hash: hash(env, id),
        pixel_count: 50_000,
        volume_mm3: 120_000,
        mean_intensity: 128,
        mask_ref: String::from_str(env, "ipfs://mask"),
        bounds: BoundingBox {
            x_min: 0,
            y_min: 0,
            x_max: 256,
            y_max: 256,
        },
    }
}

#[test]
fn test_submit_segmentation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let caller = Address::generate(&env);
    let regions = vec![&env, make_region(&env, 1), make_region(&env, 2)];

    let seg_id = client.submit_segmentation(
        &caller,
        &1u64,
        &hash(&env, 1),
        &hash(&env, 99),
        &sig(&env, 99),
        &regions,
        &250,
    );
    assert_eq!(seg_id, 1);

    let result = client.get_segmentation(&seg_id);
    assert_eq!(result.regions.len(), 2);
    assert_eq!(result.image_id, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_submit_segmentation_too_many_regions() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    register_test_model(&env, &client, &admin, 1);

    let caller = Address::generate(&env);
    let mut regions = Vec::new(&env);
    for i in 0..31u8 {
        regions.push_back(make_region(&env, i));
    }

    client.submit_segmentation(
        &caller,
        &1u64,
        &hash(&env, 1),
        &hash(&env, 99),
        &sig(&env, 99),
        &regions,
        &250,
    );
}
