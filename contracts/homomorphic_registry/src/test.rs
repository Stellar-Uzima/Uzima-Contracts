#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, String};

fn setup(env: &Env) -> (HomomorphicRegistryClient, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, HomomorphicRegistry);
    (HomomorphicRegistryClient::new(env, &id), id)
}

#[test]
fn context_and_submission_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let ctx_id = BytesN::from_array(&env, &[7u8; 32]);
    let params_ref = String::from_str(&env, "ipfs://he-params");
    let params_hash = BytesN::from_array(&env, &[9u8; 32]);

    client
        .register_context(&admin, &ctx_id, &HEScheme::Paillier, &params_ref, &params_hash);

    let submitter = Address::generate(&env);
    let comp_id = BytesN::from_array(&env, &[1u8; 32]);
    let c_ref = String::from_str(&env, "ipfs://ciphertext");
    let c_hash = BytesN::from_array(&env, &[2u8; 32]);
    let empty_proof_ref = String::from_str(&env, "");
    let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
    client
        .submit_encrypted_computation(&submitter, &comp_id, &ctx_id, &c_ref, &c_hash, &empty_proof_ref, &zero_hash);

    let fetched = client.get_computation(&comp_id).unwrap();
    assert_eq!(fetched.ciphertext_ref, c_ref);
}
