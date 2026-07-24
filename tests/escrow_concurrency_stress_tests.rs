//! Concurrency stress tests for escrow and payment claim settlement workflows.
//!
//! These tests simulate concurrent-like operations on escrow and payment
//! contracts to verify state consistency under rapid sequential operations.

#![cfg(all(test, feature = "testutils"))]

use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String,
};

use escrow::{EscrowContract, EscrowContractClient};

fn setup_escrow() -> (Env, EscrowContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);
    (env, client, admin)
}

/// Stress test: Rapid sequential escrow creation and release cycle.
#[test]
fn stress_test_rapid_escrow_lifecycle() {
    let (env, client, admin) = setup_escrow();

    let num_escrows = 20;
    let mut escrow_ids = Vec::new(&env);

    for i in 0..num_escrows {
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let amount = 1000 * (i as u64 + 1);
        let token = BytesN::from_array(&env, &[1u8; 32]);

        let id = client.create_escrow(
            &admin,
            &payer,
            &payee,
            &token,
            &amount,
        );
        escrow_ids.push_back(id);
    }

    for id in escrow_ids.iter() {
        client.release_escrow(&admin, id);
    }
}

/// Stress test: Multiple escrow operations across different payers.
#[test]
fn stress_test_concurrent_payer_escrows() {
    let (env, client, admin) = setup_escrow();

    let num_payers = 10;
    let escrows_per_payer = 5;
    let mut all_ids = Vec::new(&env);

    for _ in 0..num_payers {
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = BytesN::from_array(&env, &[2u8; 32]);

        for j in 0..escrows_per_payer {
            let amount = 500 * (j as u64 + 1);
            let id = client.create_escrow(
                &admin,
                &payer,
                &payee,
                &token,
                &amount,
            );
            all_ids.push_back(id);
        }
    }

    let half = all_ids.len() / 2;
    for i in 0..half {
        client.release_escrow(&admin, &all_ids.get(i).unwrap());
    }
    for i in half..all_ids.len() {
        client.cancel_escrow(&admin, &all_ids.get(i).unwrap());
    }
}

/// Stress test: Rapid create-release-refund cycle.
#[test]
fn stress_test_create_release_refund_cycle() {
    let (env, client, admin) = setup_escrow();

    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = BytesN::from_array(&env, &[3u8; 32]);

    for i in 0..15 {
        let amount = 100 * (i as u64 + 1);
        let id = client.create_escrow(
            &admin,
            &payer,
            &payee,
            &token,
            &amount,
        );

        if i % 2 == 0 {
            client.release_escrow(&admin, &id);
        } else {
            client.cancel_escrow(&admin, &id);
        }
    }
}

/// Stress test: Multiple distinct escrow pairs operating independently.
#[test]
fn stress_test_independent_escrow_pairs() {
    let (env, client, admin) = setup_escrow();

    let num_pairs = 15;
    let mut pair_ids = Vec::new(&env);

    for _ in 0..num_pairs {
        let payer = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = BytesN::from_array(&env, &[4u8; 32]);
        let amount = 2000;

        let id = client.create_escrow(
            &admin,
            &payer,
            &payee,
            &token,
            &amount,
        );
        pair_ids.push_back(id);
    }

    for i in 0..pair_ids.len() {
        if i % 3 == 0 {
            client.cancel_escrow(&admin, &pair_ids.get(i).unwrap());
        } else {
            client.release_escrow(&admin, &pair_ids.get(i).unwrap());
        }
    }
}

/// Stress test: Escrow operations with varying amounts.
#[test]
fn stress_test_varying_amounts() {
    let (env, client, admin) = setup_escrow();

    let amounts: [u64; 10] = [
        1,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        u64::MAX / 2,
        42,
        999_999,
        777_777,
    ];

    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = BytesN::from_array(&env, &[5u8; 32]);

    for amount in amounts.iter() {
        let id = client.create_escrow(
            &admin,
            &payer,
            &payee,
            &token,
            amount,
        );
        client.release_escrow(&admin, &id);
    }
}

/// Stress test: Rapid sequential refund operations.
#[test]
fn stress_test_rapid_refunds() {
    let (env, client, admin) = setup_escrow();

    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let token = BytesN::from_array(&env, &[6u8; 32]);

    for i in 0..20 {
        let amount = 50 * (i as u64 + 1);
        let id = client.create_escrow(
            &admin,
            &payer,
            &payee,
            &token,
            &amount,
        );
        client.cancel_escrow(&admin, &id);
    }
}
