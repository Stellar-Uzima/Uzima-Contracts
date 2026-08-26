use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, BytesN, Env, Symbol,
};

use crate::errors::{get_suggestion, Error};
use crate::{SpendingRulesContract, SpendingRulesContractClient};
use spending_categories::{SpendingCategoriesContract, SpendingCategoriesContractClient};
use spending_limits::{SpendingLimitsContract, SpendingLimitsContractClient};
use zk_verifier::{ZkVerifierContract, ZkVerifierContractClient};

struct TestContext {
    env: Env,
    rules: SpendingRulesContractClient<'static>,
    limits: SpendingLimitsContractClient<'static>,
    categories: SpendingCategoriesContractClient<'static>,
    zk: ZkVerifierContractClient<'static>,
    admin: Address,
    user: Address,
    groceries: Symbol,
}

fn setup() -> TestContext {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.timestamp = 6_048_000;
        li.sequence_number = 100;
    });

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let groceries = Symbol::new(&env, "Groceries");

    let limits_id = env.register_contract(None, SpendingLimitsContract);
    let limits = SpendingLimitsContractClient::new(&env, &limits_id);
    limits.initialize(&admin);

    let categories_id = env.register_contract(None, SpendingCategoriesContract);
    let categories = SpendingCategoriesContractClient::new(&env, &categories_id);
    categories.initialize(&admin);

    let zk_id = env.register_contract(None, ZkVerifierContract);
    let zk = ZkVerifierContractClient::new(&env, &zk_id);
    zk.initialize(&admin, &600);

    let rules_id = env.register_contract(None, SpendingRulesContract);
    let rules = SpendingRulesContractClient::new(&env, &rules_id);
    rules.initialize(&admin, &limits_id, &categories_id, &zk_id);

    TestContext {
        env,
        rules,
        limits,
        categories,
        zk,
        admin,
        user,
        groceries,
    }
}

fn setup_proof(ctx: &TestContext, proof: &Bytes) {
    let attestor = Address::generate(&ctx.env);
    let public_inputs_hash: BytesN<32> = ctx.env.crypto().sha256(proof).into();
    let proof_hash: BytesN<32> = ctx.env.crypto().sha256(proof).into();

    let vk_hash = BytesN::from_array(&ctx.env, &[4u8; 32]);
    let circuit_id = BytesN::from_array(&ctx.env, &[5u8; 32]);
    let metadata_hash = BytesN::from_array(&ctx.env, &[6u8; 32]);
    let version = ctx.zk.register_verifying_key(
        &ctx.admin,
        &vk_hash,
        &circuit_id,
        &attestor,
        &metadata_hash,
    );

    ctx.zk.submit_attestation(
        &attestor,
        &version,
        &public_inputs_hash,
        &proof_hash,
        &true,
        &300,
    );
}

#[test]
fn test_payment_under_all_thresholds_passes() {
    let ctx = setup();
    ctx.limits.set_limit(&ctx.admin, &ctx.user, &1000);
    ctx.rules
        .set_rule(&ctx.admin, &ctx.user, &ctx.groceries, &200, &100);

    let result = ctx.rules.try_evaluate(&ctx.user, &ctx.groceries, &50, &None);
    assert!(result.is_ok());
}

#[test]
fn test_payment_above_zk_threshold_fails_without_proof() {
    let ctx = setup();
    ctx.limits.set_limit(&ctx.admin, &ctx.user, &1000);
    ctx.rules
        .set_rule(&ctx.admin, &ctx.user, &ctx.groceries, &200, &100);

    let result = ctx.rules.try_evaluate(&ctx.user, &ctx.groceries, &150, &None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, soroban_sdk::Error::Contract(code) if code == Error::ZkProofRequired as u32)
    );
}

#[test]
fn test_payment_above_zk_threshold_passes_with_valid_proof() {
    let ctx = setup();
    ctx.limits.set_limit(&ctx.admin, &ctx.user, &1000);
    ctx.rules
        .set_rule(&ctx.admin, &ctx.user, &ctx.groceries, &200, &100);

    let proof = Bytes::from_slice(&ctx.env, b"valid-proof");
    setup_proof(&ctx, &proof);

    let result = ctx.rules.try_evaluate(&ctx.user, &ctx.groceries, &150, &Some(proof));
    assert!(result.is_ok());
}

#[test]
fn test_payment_exceeding_category_cap_fails_even_with_valid_proof() {
    let ctx = setup();
    ctx.limits.set_limit(&ctx.admin, &ctx.user, &1000);
    ctx.rules
        .set_rule(&ctx.admin, &ctx.user, &ctx.groceries, &200, &100);

    ctx.categories
        .record_category_spent(&ctx.user, &ctx.groceries, &80);

    let proof = Bytes::from_slice(&ctx.env, b"valid-proof-2");
    setup_proof(&ctx, &proof);

    let result = ctx.rules.try_evaluate(&ctx.user, &ctx.groceries, &170, &Some(proof));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, soroban_sdk::Error::Contract(code) if code == Error::CategoryLimitExceeded as u32)
    );
}

#[test]
fn test_overall_limit_enforced() {
    let ctx = setup();
    ctx.limits.set_limit(&ctx.admin, &ctx.user, &100);
    ctx.rules
        .set_rule(&ctx.admin, &ctx.user, &ctx.groceries, &200, &300);

    let result = ctx.rules.try_evaluate(&ctx.user, &ctx.groceries, &50, &None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, soroban_sdk::Error::Contract(code) if code == Error::OverallLimitExceeded as u32)
    );
}

#[test]
fn test_error_codes_are_stable() {
    assert_eq!(Error::Unauthorized as u32, 100);
    assert_eq!(Error::InvalidInput as u32, 200);
    assert_eq!(Error::NotInitialized as u32, 300);
    assert_eq!(Error::AlreadyInitialized as u32, 301);
    assert_eq!(Error::RuleNotFound as u32, 450);
    assert_eq!(Error::ZkProofRequired as u32, 500);
    assert_eq!(Error::CategoryLimitExceeded as u32, 510);
    assert_eq!(Error::OverallLimitExceeded as u32, 520);
    assert_eq!(Error::ZkVerificationFailed as u32, 600);
}

#[test]
fn test_get_suggestion_returns_expected_hint() {
    use soroban_sdk::symbol_short;
    assert_eq!(get_suggestion(Error::Unauthorized), symbol_short!("CHK_AUTH"));
    assert_eq!(get_suggestion(Error::NotInitialized), symbol_short!("INIT_CTR"));
    assert_eq!(get_suggestion(Error::AlreadyInitialized), symbol_short!("ALREADY"));
    assert_eq!(get_suggestion(Error::RuleNotFound), symbol_short!("SET_RULE"));
    assert_eq!(get_suggestion(Error::ZkProofRequired), symbol_short!("ADD_PROOF"));
    assert_eq!(get_suggestion(Error::CategoryLimitExceeded), symbol_short!("REDUCE"));
    assert_eq!(get_suggestion(Error::OverallLimitExceeded), symbol_short!("REDUCE"));
    assert_eq!(get_suggestion(Error::ZkVerificationFailed), symbol_short!("CONTACT"));
}
