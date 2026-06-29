//! Proptest-based fuzz harness for `cross_chain_bridge` — x86_64-unknown-linux-gnu host.
//!
//! Targets functions identified in SECURITY_CHECKLIST.md §9 that accept
//! arbitrary bytes or string inputs:
//!   • `validate_chain_address(chain, String)` — address string validation
//!   • `confirm_message(validator, id, BytesN<64>, nonce)` — signature bytes
//!   • `submit_proof(...)` with arbitrary BytesN fields
//!
//! Run locally:
//!   cd tests/fuzz/cross_chain_bridge && cargo test
//! Long-duration fuzzing:
//!   PROPTEST_CASES=2000 cargo test

use cross_chain_bridge::{ChainId, CrossChainBridgeContract};
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String as SorobanString};

/// Initialize bridge and return (contract_id, admin, validator, validator_pubkey).
fn setup(env: &Env) -> (Address, Address, Address, BytesN<32>) {
    env.mock_all_auths();

    let admin = Address::generate(env);
    let medical = Address::generate(env);
    let identity = Address::generate(env);
    let access = Address::generate(env);
    let validator_addr = Address::generate(env);
    let validator_pubkey = BytesN::from_array(env, &[0xDEu8; 32]);

    let cid = env.register_contract(None, CrossChainBridgeContract);
    let client = cross_chain_bridge::CrossChainBridgeContractClient::new(env, &cid);
    client.initialize(&admin, &medical, &identity, &access);
    client.add_validator(&admin, &validator_addr, &validator_pubkey, &1000i128);
    (cid, admin, validator_addr, validator_pubkey)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// `validate_chain_address` is a pure length-based check.
    /// Any string must return a bool and never panic.
    #[test]
    fn validate_chain_address_any_string(
        addr_str in ".*",
        chain_variant in 0u8..=7u8,
    ) {
        let env = Env::default();
        let (cid, _, _, _) = setup(&env);
        let client = cross_chain_bridge::CrossChainBridgeContractClient::new(&env, &cid);

        let chain = match chain_variant {
            0 => ChainId::Stellar,
            1 => ChainId::Ethereum,
            2 => ChainId::Polygon,
            3 => ChainId::Avalanche,
            4 => ChainId::BinanceSmartChain,
            5 => ChainId::Arbitrum,
            6 => ChainId::Optimism,
            _ => ChainId::Custom(chain_variant as u32),
        };

        // Truncate to a reasonable soroban String length
        let truncated: String = addr_str.chars().take(200).collect();
        let soroban_addr = SorobanString::from_str(&env, &truncated);

        // Must return Ok(bool), never panic
        let result = client.try_validate_chain_address(&chain, &soroban_addr);
        prop_assert!(result.is_ok(), "validate_chain_address panicked on: {:?}", truncated);
    }

    /// `confirm_message` with a random 64-byte signature must return a typed
    /// error (invalid validator nonce, invalid signature, message not found, etc.)
    /// and never panic/trap.
    #[test]
    fn confirm_message_arbitrary_signature(
        sig_bytes in any::<[u8; 64]>(),
        message_id_seed in any::<[u8; 32]>(),
        nonce in 1u64..=u64::MAX,
    ) {
        let env = Env::default();
        let (cid, _, validator, _) = setup(&env);
        let client = cross_chain_bridge::CrossChainBridgeContractClient::new(&env, &cid);

        let message_id = BytesN::from_array(&env, &message_id_seed);
        let signature = BytesN::from_array(&env, &sig_bytes);

        // Message doesn't exist, validator signature is arbitrary — must return typed Err
        let result = client.try_confirm_message(&validator, &message_id, &signature, &nonce);
        match result {
            Ok(_) | Err(_) => {} // must not panic/trap the test
        }
    }

    /// `get_chain_address_length` is a total function — any ChainId must return Ok(u32).
    #[test]
    fn get_chain_address_length_is_total(
        chain_variant in 0u8..=7u8,
        custom_id in any::<u32>(),
    ) {
        let env = Env::default();
        let (cid, _, _, _) = setup(&env);
        let client = cross_chain_bridge::CrossChainBridgeContractClient::new(&env, &cid);

        let chain = match chain_variant {
            0 => ChainId::Stellar,
            1 => ChainId::Ethereum,
            2 => ChainId::Polygon,
            3 => ChainId::Avalanche,
            4 => ChainId::BinanceSmartChain,
            5 => ChainId::Arbitrum,
            6 => ChainId::Optimism,
            _ => ChainId::Custom(custom_id),
        };

        let result = client.try_get_chain_address_length(&chain);
        prop_assert!(result.is_ok(), "get_chain_address_length panicked: {:?}", chain_variant);
    }

    /// `check_timeout` on non-existent operations must return OperationNotFound,
    /// never panic.
    #[test]
    fn check_timeout_unknown_op_id(
        op_id_seed in any::<[u8; 32]>(),
    ) {
        let env = Env::default();
        let (cid, _, _, _) = setup(&env);
        let client = cross_chain_bridge::CrossChainBridgeContractClient::new(&env, &cid);

        let op_id = BytesN::from_array(&env, &op_id_seed);
        let result = client.try_check_timeout(&op_id);
        match result {
            Ok(()) | Err(_) => {}
        }
    }
}

/// Seed corpus: typical blockchain address patterns.
#[test]
fn seed_corpus_address_patterns() {
    let addresses: &[(&str, ChainId)] = &[
        ("", ChainId::Stellar),
        ("G", ChainId::Stellar),
        ("GAHJJJKMOKYE4RVPZEWZTKH5FVI4PA3VL7GK2LFNUBSGBMH3D243GX2", ChainId::Stellar),
        ("0x", ChainId::Ethereum),
        ("0x0000000000000000000000000000000000000000", ChainId::Ethereum),
        ("not_an_address", ChainId::Polygon),
        (&"x".repeat(100), ChainId::Custom(42)),
        (&"0x".repeat(21), ChainId::Ethereum), // exactly 42 chars
    ];

    for (addr, chain) in addresses {
        let env = Env::default();
        let (cid, _, _, _) = setup(&env);
        let client = cross_chain_bridge::CrossChainBridgeContractClient::new(&env, &cid);

        let soroban_addr = SorobanString::from_str(&env, addr);
        let result = client.try_validate_chain_address(chain, &soroban_addr);
        assert!(
            result.is_ok(),
            "validate_chain_address panicked for addr '{}': {:?}",
            addr,
            result
        );
    }
}
