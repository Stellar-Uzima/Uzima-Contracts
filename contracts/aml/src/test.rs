#![cfg(test)]

use super::*;
use crate::types::RiskLevel;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{BytesN, Symbol};

#[test]
fn test_aml_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AntiMoneyLaundering);
    let client = AntiMoneyLaunderingClient::new(&env, &contract_id);

    // 1. Initialize
    client.initialize(&admin);

    // 2. Configure Rule
    client.configure_rule(
        &admin,
        &1u32, // Rule ID
        &String::from_str(&env, "Velocity Check"),
        &String::from_str(&env, "High single transaction volume"),
        &100000000i128, // 10000 XLM threshold
        &1000u32,       // 1000 bps risk (10%)
    );

    // 3. Monitor Transaction (Under threshold)
    let user = Address::generate(&env);
    let risk1 = client.monitor_transaction(&user, &50000000i128, &None);
    assert_eq!(risk1, RiskLevel::Safe);

    // 4. Monitor Transaction (Over threshold)
    let risk2 = client.monitor_transaction(&user, &200000000i128, &None);
    // Profile updated: 1000 bps = Low risk
    assert_eq!(risk2, RiskLevel::Low);

    // 5. Compliance Check
    assert!(client.is_compliant(&user));

    // 6. Blacklist
    client.update_user_status(&admin, &user, &true);
    assert!(!client.is_compliant(&user));
}

#[test]
#[allow(deprecated)]
fn test_deprecated_set_user_status_emits_warning_event() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AntiMoneyLaundering);
    let client = AntiMoneyLaunderingClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    client.initialize(&admin);

    let tracked = client.get_deprecated_functions();
    assert_eq!(tracked.len(), 1);
    let deprecation = tracked.get(0).unwrap();
    assert_eq!(deprecation.function, Symbol::new(&env, "set_user_status"));
    assert_eq!(deprecation.replacement, Some(Symbol::new(&env, "update_user_status")));

    let initial_event_count = env.events().all().len();
    client.set_user_status(&admin, &user, &true);

    assert!(!client.is_compliant(&user));

    let events = env.events().all();
    assert!(events.len() > initial_event_count);

    let deprecated_events = events
        .iter()
        .filter(|event| {
            event.topics.len() >= 2
                && event.topics[0] == Symbol::new(&env, "Deprecated")
                && event.topics[1] == Symbol::new(&env, "set_user_status")
        })
        .count();
    assert_eq!(deprecated_events, 1);
}

#[test]
fn test_validate_upgrade_reports_initialized_state() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AntiMoneyLaundering);
    let client = AntiMoneyLaunderingClient::new(&env, &contract_id);

    client.initialize(&admin);

    let validation =
        AntiMoneyLaundering::validate_upgrade(env.clone(), BytesN::from_array(&env, &[7; 32]))
            .unwrap();
    assert!(validation.state_compatible);
    assert!(validation.api_compatible);
}
