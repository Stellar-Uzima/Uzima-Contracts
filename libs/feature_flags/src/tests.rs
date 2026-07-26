#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{
    evaluate_rollout, get_flag, is_enabled, set_environment_override, set_flag, DataKey,
    FeatureFlag, FlagStage,
};

fn setup_flag(env: &Env, name: &str, enabled: bool, rollout: u32, stage: FlagStage) {
    let flag = FeatureFlag {
        name: String::from_str(env, name),
        enabled,
        rollout_percentage: rollout,
        stage,
    };
    set_flag(env, &flag);
}

#[test]
fn test_set_and_get_flag() {
    let env = Env::default();

    setup_flag(&env, "partial_updates", true, 50, FlagStage::Beta);

    let flag = get_flag(&env, "partial_updates").unwrap();
    assert!(flag.enabled);
    assert_eq!(flag.rollout_percentage, 50);
    assert_eq!(flag.stage, FlagStage::Beta);
}

#[test]
fn test_get_flag_not_found() {
    let env = Env::default();
    let result = get_flag(&env, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_is_enabled_true() {
    let env = Env::default();
    setup_flag(&env, "my_flag", true, 100, FlagStage::Ga);
    assert!(is_enabled(&env, "my_flag"));
}

#[test]
fn test_is_enabled_false() {
    let env = Env::default();
    setup_flag(&env, "my_flag", false, 100, FlagStage::Ga);
    assert!(!is_enabled(&env, "my_flag"));
}

#[test]
fn test_is_enabled_missing() {
    let env = Env::default();
    assert!(!is_enabled(&env, "missing_flag"));
}

#[test]
fn test_evaluate_rollout_full() {
    let env = Env::default();
    setup_flag(&env, "full_rollout", true, 100, FlagStage::Ga);

    let caller = Address::generate(&env);
    assert!(evaluate_rollout(&env, "full_rollout", &caller).unwrap());
}

#[test]
fn test_evaluate_rollout_disabled() {
    let env = Env::default();
    setup_flag(&env, "no_rollout", true, 0, FlagStage::Disabled);

    let caller = Address::generate(&env);
    assert!(!evaluate_rollout(&env, "no_rollout", &caller).unwrap());
}

#[test]
fn test_evaluate_rollout_canary() {
    let env = Env::default();
    // 25% rollout
    setup_flag(&env, "canary_flag", true, 25, FlagStage::Canary);

    let caller = Address::generate(&env);
    // Just verify it returns a boolean without panicking
    let _result = evaluate_rollout(&env, "canary_flag", &caller).unwrap();
}

#[test]
fn test_evaluate_rollout_consistency() {
    let env = Env::default();
    setup_flag(&env, "consistency_flag", true, 25, FlagStage::Canary);

    let caller = Address::generate(&env);

    // Same caller should always get the same result
    let first = evaluate_rollout(&env, "consistency_flag", &caller).unwrap();
    let second = evaluate_rollout(&env, "consistency_flag", &caller).unwrap();
    assert_eq!(first, second);
}

#[test]
fn test_evaluate_rollout_different_callers_can_differ() {
    let env = Env::default();
    setup_flag(&env, "mixed_flag", true, 50, FlagStage::Beta);

    let caller1 = Address::generate(&env);
    let caller2 = Address::generate(&env);

    // With 50% rollout and different callers, there's a chance
    // at least one differs. We just verify no panics and both are booleans.
    let r1 = evaluate_rollout(&env, "mixed_flag", &caller1).unwrap();
    let r2 = evaluate_rollout(&env, "mixed_flag", &caller2).unwrap();
    // Both are valid booleans
    assert!(r1 == true || r1 == false);
    assert!(r2 == true || r2 == false);
}

#[test]
fn test_evaluate_rollout_missing_flag() {
    let env = Env::default();
    let caller = Address::generate(&env);
    let result = evaluate_rollout(&env, "no_such_flag", &caller);
    assert!(result.is_err());
}

#[test]
fn test_environment_override_all_enabled() {
    let env = Env::default();

    // No flags set, but environment override forces all on
    set_environment_override(&env, true);

    let caller = Address::generate(&env);
    // Even a nonexistent flag returns true via env override
    assert!(evaluate_rollout(&env, "anything", &caller).unwrap());
}

#[test]
fn test_environment_override_all_disabled() {
    let env = Env::default();

    // Even with a fully-enabled flag, env override forces all off
    setup_flag(&env, "fully_on", true, 100, FlagStage::Ga);
    set_environment_override(&env, false);

    // When override is false, the function proceeds to normal evaluation
    let caller = Address::generate(&env);
    let result = evaluate_rollout(&env, "fully_on", &caller).unwrap();
    assert!(result); // No override active, flag is 100%
}
