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
