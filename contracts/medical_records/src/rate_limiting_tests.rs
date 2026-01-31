#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
// We need the main contract struct to register it for storage context
use crate::rate_limiting::test_helpers::*;
use crate::rate_limiting::*;
use crate::MedicalRecordsContract;

#[test]
fn test_regular_user_within_limit() {
    let env = Env::default();
    env.mock_all_auths();

    // Register the contract to create a storage context
    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let user = Address::generate(&env);

    // Execute logic AS the contract
    env.as_contract(&contract_id, || {
        // Regular user should be able to make 5 operations
        for i in 0..5 {
            let result = enforce_rate_limit(&env, &user, UserRole::RegularUser);
            assert!(result.is_ok(), "Operation {} should succeed", i + 1);
        }

        // 6th operation should fail
        let result = enforce_rate_limit(&env, &user, UserRole::RegularUser);
        assert_eq!(result, Err(RateLimitError::LimitExceeded));
    });
}

#[test]
fn test_doctor_higher_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let doctor = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Doctor should be able to make 20 operations
        for i in 0..20 {
            let result = enforce_rate_limit(&env, &doctor, UserRole::Doctor);
            assert!(result.is_ok(), "Doctor operation {} should succeed", i + 1);
        }

        // 21st operation should fail
        let result = enforce_rate_limit(&env, &doctor, UserRole::Doctor);
        assert_eq!(result, Err(RateLimitError::LimitExceeded));
    });
}

#[test]
fn test_admin_unlimited_access() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let admin = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Admin should be able to make > 20 operations (unlimited)
        for i in 0..50 {
            let result = enforce_rate_limit(&env, &admin, UserRole::Admin);
            assert!(result.is_ok(), "Admin operation {} should succeed", i + 1);
        }
    });
}

#[test]
fn test_window_reset_after_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MedicalRecordsContract);
    let user = Address::generate(&env);

    // 1. Hit the limit
    env.as_contract(&contract_id, || {
        for _ in 0..5 {
            enforce_rate_limit(&env, &user, UserRole::RegularUser).unwrap();
        }

        // Verify blocked
        assert_eq!(
            enforce_rate_limit(&env, &user, UserRole::RegularUser),
            Err(RateLimitError::LimitExceeded)
        );
    });

    // 2. Advance past the window (100 ledgers)
    // Ledger time is global, so we can do this outside
    advance_ledger(&env, 100);

    // 3. Try again in new window
    env.as_contract(&contract_id, || {
        let result = enforce_rate_limit(&env, &user, UserRole::RegularUser);
        assert!(result.is_ok(), "Should succeed in new window");
    });
}
