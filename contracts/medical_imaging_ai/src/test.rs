#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

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
