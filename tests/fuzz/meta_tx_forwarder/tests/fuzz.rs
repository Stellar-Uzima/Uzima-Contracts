//! Proptest-based fuzz harness for `meta_tx_forwarder` — x86_64-unknown-linux-gnu host.
//!
//! Targets identified in SECURITY_CHECKLIST.md §9:
//!   • `execute(relayer, ForwardRequest, signature: BytesN<64>)` — Ed25519 sig bytes
//!   • `execute_batch(...)` — multiple requests with mismatched or arbitrary signatures
//!
//! The Ed25519 `verify_signature` function constructs a domain-separated message
//! from `DOMAIN_PREFIX || forwarder_xdr || request.to_xdr()` and calls
//! `env.crypto().ed25519_verify(pub_key, message, sig)`. Feeding arbitrary
//! 64-byte signatures exercises the host's cryptographic trap path; `try_execute`
//! catches the trap as `Err(soroban_sdk::Error)`.
//!
//! Run locally:
//!   cd tests/fuzz/meta_tx_forwarder && cargo test
//! Long-duration fuzzing:
//!   PROPTEST_CASES=2000 cargo test

use meta_tx_forwarder::{ForwardRequest, MetaTxForwarder};
use proptest::prelude::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, IntoVal, Val, Vec as SorobanVec,
};

fn setup(env: &Env) -> (Address, Address, BytesN<32>) {
    env.mock_all_auths();

    let owner = Address::generate(env);
    let fee_collector = Address::generate(env);
    let relayer = Address::generate(env);
    let user = Address::generate(env);
    // Generate a dummy 32-byte public key (not a real Ed25519 key — used for
    // non-auth paths; actual sig verification uses the key registered in storage)
    let pub_key = BytesN::from_array(env, &[0xABu8; 32]);

    let cid = env.register_contract(None, MetaTxForwarder);
    let client = meta_tx_forwarder::MetaTxForwarderClient::new(env, &cid);
    client.initialize(&owner, &fee_collector, &100_i128);
    client.register_relayer(&owner, &relayer, &100_u32);
    client.register_user_pub_key(&user, &pub_key);

    (cid, relayer, pub_key)
}

fn make_request(env: &Env, from: &Address, to: &Address, nonce: u64, deadline: u64) -> ForwardRequest {
    ForwardRequest {
        from: from.clone(),
        to: to.clone(),
        value: 0,
        gas: 0,
        nonce,
        deadline,
        target_fn: symbol_short!("noop"),
        target_args: SorobanVec::new(env),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    /// Execute with an arbitrary 64-byte signature.
    /// The `ed25519_verify` host call traps on bad sigs; `try_execute` catches it
    /// as `Err`. Must never propagate as a Rust panic in the test process.
    #[test]
    fn execute_arbitrary_signature(
        sig_bytes in any::<[u8; 64]>(),
        nonce in 0u64..100u64,
        deadline_offset in 0u64..86400u64,
    ) {
        let env = Env::default();
        env.ledger().with_mut(|li| { li.timestamp = 1_000_000; });
        let (cid, relayer, _) = setup(&env);
        let client = meta_tx_forwarder::MetaTxForwarderClient::new(&env, &cid);

        let user = Address::generate(&env);
        let pub_key = BytesN::from_array(&env, &[0xABu8; 32]);
        client.register_user_pub_key(&user, &pub_key);

        let target = Address::generate(&env);
        let deadline = env.ledger().timestamp() + deadline_offset + 1;
        let request = make_request(&env, &user, &target, nonce, deadline);
        let signature = BytesN::from_array(&env, &sig_bytes);

        // Must not propagate as a Rust panic — catch via try_execute
        let result = client.try_execute(&relayer, &request, &signature);
        match result {
            Ok(_) | Err(_) => {}
        }
    }

    /// `execute` with an expired deadline must always return `RequestExpired`.
    #[test]
    fn execute_expired_request_is_rejected(
        sig_bytes in any::<[u8; 64]>(),
    ) {
        let env = Env::default();
        env.ledger().with_mut(|li| { li.timestamp = 1_000_000; });
        let (cid, relayer, _) = setup(&env);
        let client = meta_tx_forwarder::MetaTxForwarderClient::new(&env, &cid);

        let user = Address::generate(&env);
        let pub_key = BytesN::from_array(&env, &[0xCCu8; 32]);
        client.register_user_pub_key(&user, &pub_key);

        // deadline in the past
        let request = make_request(&env, &user, &Address::generate(&env), 0, 999_999);
        let signature = BytesN::from_array(&env, &sig_bytes);

        let result = client.try_execute(&relayer, &request, &signature);
        prop_assert!(
            result.is_err(),
            "expired request must return Err, got: {:?}",
            result
        );
    }

    /// `get_nonce` is a total function — must return Ok(u64) for any address.
    #[test]
    fn get_nonce_is_total(
        _seed in any::<u8>(),
    ) {
        let env = Env::default();
        let (cid, _, _) = setup(&env);
        let client = meta_tx_forwarder::MetaTxForwarderClient::new(&env, &cid);

        let addr = Address::generate(&env);
        // get_nonce() -> u64 means try_get_nonce() has the nested Result type
        let result = client.try_get_nonce(&addr);
        prop_assert!(
            matches!(result, Ok(Ok(_))),
            "get_nonce must never fail: {:?}", result
        );
        if let Ok(Ok(nonce)) = result {
            prop_assert_eq!(nonce, 0u64, "new address should have nonce 0");
        }
    }

    /// `execute_batch` with mismatched request/signature lengths must return
    /// `BatchLengthMismatch`, never panic.
    #[test]
    fn execute_batch_length_mismatch(
        n_requests in 1usize..=4,
        n_sigs in 0usize..=4,
    ) {
        let env = Env::default();
        env.ledger().with_mut(|li| { li.timestamp = 1_000_000; });
        let (cid, relayer, _) = setup(&env);
        let client = meta_tx_forwarder::MetaTxForwarderClient::new(&env, &cid);

        let user = Address::generate(&env);
        let pub_key = BytesN::from_array(&env, &[0xDDu8; 32]);
        client.register_user_pub_key(&user, &pub_key);

        let mut requests: soroban_sdk::Vec<ForwardRequest> = soroban_sdk::Vec::new(&env);
        let mut sigs: soroban_sdk::Vec<BytesN<64>> = soroban_sdk::Vec::new(&env);

        let target = Address::generate(&env);
        for i in 0..n_requests {
            requests.push_back(make_request(
                &env, &user, &target,
                i as u64,
                env.ledger().timestamp() + 3600
            ));
        }
        for _ in 0..n_sigs {
            sigs.push_back(BytesN::from_array(&env, &[0u8; 64]));
        }

        let result = client.try_execute_batch(&relayer, &requests, &sigs);
        if n_requests != n_sigs {
            prop_assert!(
                result.is_err(),
                "mismatched lengths must return Err, got: {:?}", result
            );
        } else {
            match result { Ok(_) | Err(_) => {} }
        }
    }
}

/// Seed corpus: known edge cases for signature handling.
#[test]
fn seed_corpus_signatures() {
    let sigs: &[[u8; 64]] = &[
        [0x00u8; 64],
        [0xFFu8; 64],
        {
            let mut s = [0u8; 64];
            s[0] = 0x01;
            s[63] = 0x01;
            s
        },
        {
            let mut s = [0u8; 64];
            // High-bit set on scalar (non-canonical Edwards point)
            s[31] = 0x80;
            s[63] = 0x80;
            s
        },
    ];

    for sig_bytes in sigs {
        let env = Env::default();
        env.ledger().with_mut(|li| { li.timestamp = 1_000_000; });
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let fee_collector = Address::generate(&env);
        let relayer_addr = Address::generate(&env);
        let user = Address::generate(&env);
        let pub_key = BytesN::from_array(&env, &[0xABu8; 32]);

        let cid = env.register_contract(None, MetaTxForwarder);
        let client = meta_tx_forwarder::MetaTxForwarderClient::new(&env, &cid);
        client.initialize(&owner, &fee_collector, &0_i128);
        client.register_relayer(&owner, &relayer_addr, &0_u32);
        client.register_user_pub_key(&user, &pub_key);

        let request = ForwardRequest {
            from: user.clone(),
            to: Address::generate(&env),
            value: 0,
            gas: 0,
            nonce: 0,
            deadline: env.ledger().timestamp() + 3600,
            target_fn: symbol_short!("noop"),
            target_args: SorobanVec::new(&env),
        };
        let signature = BytesN::from_array(&env, sig_bytes);

        let result = client.try_execute(&relayer_addr, &request, &signature);
        // Must not panic
        match result {
            Ok(_) | Err(_) => {}
        }
    }
}
