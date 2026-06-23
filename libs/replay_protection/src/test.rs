use super::*;
use soroban_sdk::testutils::Ledger;
use soroban_sdk::{contract, contractimpl, BytesN, Env};

/// Minimal contract providing a registered storage context for testing.
#[contract]
struct TestContext;

#[contractimpl]
impl TestContext {
    pub fn __stub() {}
}

fn with_contract<R>(env: &Env, f: impl FnOnce() -> R) -> R {
    let id = env.register_contract(None, TestContext);
    env.as_contract(&id, f)
}

#[test]
fn test_successful_verification() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender = BytesN::from_array(&env, &[2u8; 32]);
    with_contract(&env, || {
        let r = verify_replay_protection(
            &env, &msg_hash, &sender, 1, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r, Ok(()));
        assert!(is_message_seen(&env, &msg_hash));
    });
}

#[test]
fn test_nonce_reuse_rejected() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender = BytesN::from_array(&env, &[2u8; 32]);
    with_contract(&env, || {
        let r1 = verify_replay_protection(
            &env, &msg_hash, &sender, 1, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r1, Ok(()));
        let r2 = verify_replay_protection(
            &env, &msg_hash, &sender, 1, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r2, Err(ReplayError::NonceReused));
    });
}

#[test]
fn test_expired_message_rejected() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender = BytesN::from_array(&env, &[2u8; 32]);
    with_contract(&env, || {
        let r = verify_replay_protection(
            &env, &msg_hash, &sender, 1, 1000, 1000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r, Err(ReplayError::MessageExpired));
    });
}

#[test]
fn test_chain_mismatch_rejected() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender = BytesN::from_array(&env, &[2u8; 32]);
    with_contract(&env, || {
        let r = verify_replay_protection(
            &env, &msg_hash, &sender, 1, 4000, 2000, &ChainId::Stellar, &ChainId::Ethereum,
        );
        assert_eq!(r, Err(ReplayError::ChainMismatch));
    });
}

#[test]
fn test_is_message_seen_false_for_unseen() {
    let env = Env::default();
    let msg_hash = BytesN::from_array(&env, &[99u8; 32]);
    with_contract(&env, || {
        assert!(!is_message_seen(&env, &msg_hash));
    });
}

#[test]
fn test_check_message_expired_within_window() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    assert_eq!(check_message_expired(&env, 4000, 2000), Ok(()));
}

#[test]
fn test_check_message_expired_past_deadline() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 7000; });
    assert_eq!(
        check_message_expired(&env, 4000, 2000),
        Err(ReplayError::MessageExpired)
    );
}

#[test]
fn test_check_message_expired_overflow() {
    let env = Env::default();
    assert_eq!(
        check_message_expired(&env, u64::MAX, 1),
        Err(ReplayError::ExpiryOverflow)
    );
}

#[test]
fn test_nonce_must_increase() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender = BytesN::from_array(&env, &[2u8; 32]);
    with_contract(&env, || {
        let r1 = verify_replay_protection(
            &env, &msg_hash, &sender, 5, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r1, Ok(()));
        let r2 = verify_replay_protection(
            &env, &msg_hash, &sender, 3, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r2, Err(ReplayError::NonceReused));
    });
}

#[test]
fn test_is_message_seen_independent_per_hash() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender = BytesN::from_array(&env, &[2u8; 32]);
    with_contract(&env, || {
        let other_hash = BytesN::from_array(&env, &[42u8; 32]);
        let _ = verify_replay_protection(
            &env, &msg_hash, &sender, 1, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert!(is_message_seen(&env, &msg_hash));
        assert!(!is_message_seen(&env, &other_hash));
    });
}

#[test]
fn test_separate_sender_nonces_independent() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender_a = BytesN::from_array(&env, &[10u8; 32]);
    let sender_b = BytesN::from_array(&env, &[20u8; 32]);
    with_contract(&env, || {
        let r = verify_replay_protection(
            &env, &msg_hash, &sender_a, 1, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r, Ok(()));
        let r = verify_replay_protection(
            &env, &msg_hash, &sender_b, 1, 4000, 2000, &ChainId::Stellar, &ChainId::Stellar,
        );
        assert_eq!(r, Ok(()));
    });
}

#[test]
fn test_custom_chain_match() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let msg_hash = BytesN::from_array(&env, &[1u8; 32]);
    let sender = BytesN::from_array(&env, &[2u8; 32]);
    with_contract(&env, || {
        let r = verify_replay_protection(
            &env, &msg_hash, &sender, 1, 4000, 2000, &ChainId::Custom(42), &ChainId::Custom(42),
        );
        assert_eq!(r, Ok(()));
    });
}

#[test]
fn test_all_chain_variants() {
    let env = Env::default();
    env.ledger().with_mut(|li| { li.timestamp = 5000; });
    let chains = [
        ChainId::Stellar,
        ChainId::Ethereum,
        ChainId::Polygon,
        ChainId::Avalanche,
        ChainId::BinanceSmartChain,
        ChainId::Arbitrum,
        ChainId::Optimism,
        ChainId::Custom(0),
        ChainId::Custom(u32::MAX),
    ];
    with_contract(&env, || {
        for (i, chain) in chains.iter().enumerate() {
            let mut h = [0u8; 32];
            h[0] = i as u8;
            let msg_hash = BytesN::from_array(&env, &h);
            let mut s = [0u8; 32];
            s[0] = (i + 100) as u8;
            let sender = BytesN::from_array(&env, &s);
            let r = verify_replay_protection(
                &env, &msg_hash, &sender, 1, 4000, 2000, chain, chain,
            );
            assert_eq!(r, Ok(()), "chain {:?} should pass", chain);
        }
    });
}
