#![cfg(all(test, feature = "testutils"))]

use crate::{QueuedTx, Timelock, TimelockConfig, CFG, QUEUE};
use soroban_sdk::{
    map, testutils::{Address as _, Ledger, LedgerInfo},
    Address, BytesN, Env, Map,
};

#[test]
fn execute_before_min_delay_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        Timelock::initialize(env.clone(), admin.clone(), 100).unwrap();
        let target = Address::generate(&env);
        let call = BytesN::from_array(&env, &[0u8; 32]);
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        // Try executing before the min delay has elapsed
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 50,
            ..Default::default()
        });
        let result = Timelock::execute(env.clone(), 1);
        assert!(result.is_err());
    });
}

#[test]
fn execute_after_min_delay_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        Timelock::initialize(env.clone(), admin.clone(), 100).unwrap();
        let target = Address::generate(&env);
        let call = BytesN::from_array(&env, &[0u8; 32]);
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        // Advance past the full delay window
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + 200,
            ..Default::default()
        });
        let result = Timelock::execute(env.clone(), 1);
        assert!(result.is_ok());
    });
}

#[test]
fn execute_exactly_at_delay_boundary() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Timelock);
    env.as_contract(&contract_id, || {
        let admin = Address::generate(&env);
        Timelock::initialize(env.clone(), admin.clone(), 100).unwrap();
        let target = Address::generate(&env);
        let call = BytesN::from_array(&env, &[0u8; 32]);
        Timelock::queue(env.clone(), 1, target.clone(), call.clone()).unwrap();

        let cfg: TimelockConfig = env.storage().instance().get(&CFG).unwrap();
        env.ledger().set(LedgerInfo {
            timestamp: env.ledger().timestamp() + cfg.delay_seconds,
            ..Default::default()
        });
        let result = Timelock::execute(env.clone(), 1);
        assert!(result.is_ok());
    });
}