#![cfg(test)]

use super::*;
use crate::types::RiskLevel;
use soroban_sdk::testutils::{Address as _, Ledger};

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
    client.set_user_status(&admin, &user, &true);
    assert!(!client.is_compliant(&user));
}
