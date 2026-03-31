#![cfg(test)]

use super::*;
use crate::types::{AuthStatus, FactorType, MFAConfig};
use soroban_sdk::testutils::{Address as _, Ledger};

#[test]
fn test_mfa_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, MultiFactorAuth);
    let client = MultiFactorAuthClient::new(&env, &contract_id);

    let config = MFAConfig {
        session_ttl: 3600,
        min_factors_for_critical_op: 2,
        recovery_delay: 86400,
    };

    // 1. Initialize
    client.initialize(&admin, &config);

    // 2. Add Factors
    let user = Address::generate(&env);
    let _f1 = client.add_factor(
        &user,
        &FactorType::Password,
        &None,
        &String::from_str(&env, "Main Pass"),
    );
    let _f2 = client.add_factor(
        &user,
        &FactorType::HardwareKey,
        &None,
        &String::from_str(&env, "YubiKey"),
    );

    // 3. Start Session
    let mut req = Vec::new(&env);
    req.push_back(FactorType::Password);
    req.push_back(FactorType::HardwareKey);
    client.start_session(&user, &req);

    // 4. Verify Factors
    client.verify_mfa_factor(
        &user,
        &FactorType::Password,
        &Bytes::from_slice(&env, b"1234"),
    );
    client.verify_mfa_factor(
        &user,
        &FactorType::HardwareKey,
        &Bytes::from_slice(&env, b"sig"),
    );

    // 5. Authenticate
    assert!(client.is_authenticated(&user));

    // 6. Test Expiration
    env.ledger().set_timestamp(env.ledger().timestamp() + 4000);
    assert!(!client.is_authenticated(&user));
}
