use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, BytesN, Env,
};

// ── Boundary condition tests ──────────────────────────────────────────

/// Test: execute exactly at eta boundary
#[test]
fn test_execute_exactly_at_eta() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    env.as_contract(&contract_id, || {
        Timelock::initialize(env.clone(), admin, 10).unwrap();
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        // Advance EXACTLY to eta (initial ts + 10)
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 10,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1).unwrap();
    });
}

/// Test: execute 1 second BEFORE eta should fail
#[test]
fn test_execute_one_second_before_eta_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    env.as_contract(&contract_id, || {
        Timelock::initialize(env.clone(), admin, 10).unwrap();
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        // Advance to 1 second before eta
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 9,
            ..Default::default()
        });
        let result = Timelock::execute(env.clone(), 1);
        assert_eq!(result, Err(Error::NotReady));
    });
}

/// Test: execute 1 second AFTER eta should succeed
#[test]
fn test_execute_one_second_after_eta_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    env.as_contract(&contract_id, || {
        Timelock::initialize(env.clone(), admin, 10).unwrap();
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        // Advance to 1 second after eta
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 11,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1).unwrap();
    });
}

// ── Time manipulation attack tests ────────────────────────────────────

/// Test: setting timestamp to far future should allow execution
#[test]
fn test_far_future_timestamp() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    env.as_contract(&contract_id, || {
        Timelock::initialize(env.clone(), admin, 10).unwrap();
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        // Set to very far future
        env.ledger().set(LedgerInfo {
            timestamp: u64::MAX / 2,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1).unwrap();
    });
}

/// Test: large timestamp jump (epoch overflow edge case)
#[test]
fn test_large_timestamp_jump() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    env.as_contract(&contract_id, || {
        Timelock::initialize(env.clone(), admin, 10).unwrap();
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        // Massive jump forward
        env.ledger().set(LedgerInfo {
            timestamp: 1_000_000_000_000_000u64,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1).unwrap();
    });
}

/// Test: near u64::MAX timestamp
#[test]
fn test_max_timestamp_edge() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    env.as_contract(&contract_id, || {
        Timelock::initialize(env.clone(), admin, 10).unwrap();
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        env.ledger().set(LedgerInfo {
            timestamp: u64::MAX - 1,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1).unwrap();
    });
}

// ── Sequential time progression ──────────────────────────────────────

/// Test: sequential time progression, verify execution fails until ready
#[test]
fn test_sequential_time_progression() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    let admin = Address::generate(&env);
    let target = Address::generate(&env);
    let call = BytesN::from_array(&env, &[0u8; 32]);
    env.as_contract(&contract_id, || {
        let initial_ts = 1_000_000u64;
        env.ledger().set(LedgerInfo {
            timestamp: initial_ts,
            ..Default::default()
        });
        Timelock::initialize(env.clone(), admin, 10).unwrap();
        Timelock::queue(env.clone(), 1, target, call).unwrap();

        for offset in &[1u64, 5, 9] {
            env.ledger().set(LedgerInfo {
                timestamp: initial_ts + offset,
                ..Default::default()
            });
            let result = Timelock::execute(env.clone(), 1);
            assert_eq!(result, Err(Error::NotReady));
        }

        // At offset 10 (eta), execution should succeed
        env.ledger().set(LedgerInfo {
            timestamp: initial_ts + 10,
            ..Default::default()
        });
        Timelock::execute(env.clone(), 1).unwrap();
    });
}

// Time-dependent assumptions documented:
//
// 1. `queue()` sets `eta = now + delay_seconds`. The transaction is
//    executable when `env.ledger().timestamp() >= eta`.
// 2. The contract also requires `current_sequence >= queued_at_sequence + min_sequence_advance`
//    to prevent execution in the same ledger.
// 3. Timestamps are assumed to be monotonically non-decreasing (Soroban
//    Ledger guarantees this).
// 4. No upper bound is placed on timestamps — the contract handles
//    large values via saturating arithmetic.
// 5. The `eta` computation uses `saturating_add` to avoid overflow.
