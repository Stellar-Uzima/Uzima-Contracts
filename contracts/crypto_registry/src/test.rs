#![allow(clippy::unwrap_used)]
use super::*;

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Bytes, Env};

fn register_contract(env: &Env) -> (CryptoRegistryClient<'_>, soroban_sdk::Address) {
    let id = soroban_sdk::Address::generate(env);
    env.register_contract(&id, CryptoRegistry);
    (CryptoRegistryClient::new(env, &id), id)
}

#[test]
fn key_bundle_registration_and_rotation() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = register_contract(&env);
    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let alice = soroban_sdk::Address::generate(&env);
    let enc_key = PublicKey {
        algorithm: KeyAlgorithm::X25519,
        key: Bytes::from_slice(&env, &[1u8; 32]),
    };
    let empty = PublicKey {
        algorithm: KeyAlgorithm::Custom(0),
        key: Bytes::new(&env),
    };

    let v1 = client.register_key_bundle(&alice, &enc_key, &empty, &false, &empty, &false);
    assert_eq!(v1, 1);

    let current = client.get_current_key_bundle(&alice);
    assert_eq!(current.as_ref().map(|b| b.version), Some(1));
    assert_eq!(current.as_ref().map(|b| b.revoked), Some(false));
    assert_eq!(client.get_current_version(&alice), 1);

    // Rotate
    let enc_key2 = PublicKey {
        algorithm: KeyAlgorithm::X25519,
        key: Bytes::from_slice(&env, &[2u8; 32]),
    };
    let v2 = client.register_key_bundle(&alice, &enc_key2, &empty, &false, &empty, &false);
    assert_eq!(v2, 2);
    assert_eq!(client.get_current_version(&alice), 2);
}

#[test]
fn revoke_bundle_marks_revoked() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = register_contract(&env);
    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let alice = soroban_sdk::Address::generate(&env);
    let enc_key = PublicKey {
        algorithm: KeyAlgorithm::X25519,
        key: Bytes::from_slice(&env, &[1u8; 32]),
    };
    let empty = PublicKey {
        algorithm: KeyAlgorithm::Custom(0),
        key: Bytes::new(&env),
    };

    let v1 = client.register_key_bundle(&alice, &enc_key, &empty, &false, &empty, &false);
    client.revoke_key_bundle(&alice, &v1);

    let revoked = client.get_key_bundle(&alice, &v1).map(|b| b.revoked);
    assert_eq!(revoked, Some(true));
}
#[test]
fn post_quantum_key_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = register_contract(&env);
    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let alice = soroban_sdk::Address::generate(&env);

    // Kyber-768 public key
    let kyber_key = PublicKey {
        algorithm: KeyAlgorithm::Kyber768,
        key: Bytes::from_slice(&env, &[0u8; 1184]),
    };

    // Dilithium-3 signing key
    let dilithium_key = PublicKey {
        algorithm: KeyAlgorithm::Dilithium3,
        key: Bytes::from_slice(&env, &[0u8; 1952]),
    };

    let enc_key = PublicKey {
        algorithm: KeyAlgorithm::X25519,
        key: Bytes::from_slice(&env, &[1u8; 32]),
    };

    let v1 = client.register_key_bundle(&alice, &enc_key, &kyber_key, &true, &dilithium_key, &true);
    assert_eq!(v1, 1);

    let current = client.get_current_key_bundle(&alice).unwrap();
    assert_eq!(current.pq_encryption_key.algorithm, KeyAlgorithm::Kyber768);
    assert_eq!(current.signing_key.algorithm, KeyAlgorithm::Dilithium3);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn invalid_pq_key_length() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = register_contract(&env);
    let admin = soroban_sdk::Address::generate(&env);
    client.initialize(&admin);

    let alice = soroban_sdk::Address::generate(&env);

    // Wrong length for Kyber-768
    let kyber_key = PublicKey {
        algorithm: KeyAlgorithm::Kyber768,
        key: Bytes::from_slice(&env, &[0u8; 1000]),
    };

    let enc_key = PublicKey {
        algorithm: KeyAlgorithm::X25519,
        key: Bytes::from_slice(&env, &[1u8; 32]),
    };

    let empty = PublicKey {
        algorithm: KeyAlgorithm::Custom(0),
        key: Bytes::new(&env),
    };

    client.register_key_bundle(&alice, &enc_key, &kyber_key, &true, &empty, &false);
}
