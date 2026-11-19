#![cfg(all(test, feature = "testutils"))]
#![allow(clippy::unwrap_used)]

use crate::{PaymentRouter, PaymentRouterClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
fn test_fee_split() {
    let env = Env::default();
    let cid = env.register_contract(None, PaymentRouter);
    let client = PaymentRouterClient::new(&env, &cid);
    let receiver = Address::generate(&env);

    client.set_fee_config(&receiver, &1000u32);

    let config = client.get_fee_config().unwrap();
    assert_eq!(config.platform_fee_bps, 1000);
    assert_eq!(config.fee_receiver, receiver);

    let (provider, fee) = client.compute_split(&1000i128);
    assert_eq!(provider, 900);
    assert_eq!(fee, 100);
}

#[test]
fn test_compute_split_requires_fee_config() {
    let env = Env::default();
    let cid = env.register_contract(None, PaymentRouter);
    let client = PaymentRouterClient::new(&env, &cid);

    assert!(client.try_compute_split(&1000i128).is_err());
}

#[test]
fn test_fee_config_rejects_bps_above_maximum() {
    let env = Env::default();
    let cid = env.register_contract(None, PaymentRouter);
    let client = PaymentRouterClient::new(&env, &cid);

    assert!(client
        .try_set_fee_config(&Address::generate(&env), &10_001u32)
        .is_err());
    assert!(client.get_fee_config().is_none());
}

#[test]
fn test_fee_split_at_maximum_bps() {
    let env = Env::default();
    let cid = env.register_contract(None, PaymentRouter);
    let client = PaymentRouterClient::new(&env, &cid);

    client.set_fee_config(&Address::generate(&env), &10_000u32);

    let (provider, fee) = client.compute_split(&1000i128);
    assert_eq!(provider, 0);
    assert_eq!(fee, 1000);
}
