use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{vec, Address, BytesN, Env, String};

fn setup(env: &Env) -> (MPCManagerClient<'_>, Address) {
    let id = Address::generate(env);
    env.register_contract(&id, MPCManager);
    (MPCManagerClient::new(env, &id), id)
}

#[test]
fn mpc_session_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _id) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let initiator = Address::generate(&env);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    let participants = vec![&env, p1.clone(), p2.clone()];
    let sid = BytesN::from_array(&env, &[3u8; 32]);
    let purpose = String::from_str(&env, "cohort-risk-analysis");

    client.start_session(&initiator, &sid, &participants, &2u32, &purpose, &100u64);

    client.commit_share(&p1, &sid, &BytesN::from_array(&env, &[1u8; 32]));
    client.commit_share(&p2, &sid, &BytesN::from_array(&env, &[2u8; 32]));

    client.reveal_share(
        &p1,
        &sid,
        &String::from_str(&env, "ipfs://share1"),
        &BytesN::from_array(&env, &[4u8; 32]),
    );
    client.reveal_share(
        &p2,
        &sid,
        &String::from_str(&env, "ipfs://share2"),
        &BytesN::from_array(&env, &[5u8; 32]),
    );

    client.finalize_session(
        &initiator,
        &sid,
        &String::from_str(&env, "ipfs://result"),
        &BytesN::from_array(&env, &[9u8; 32]),
        &String::from_str(&env, ""),
        &BytesN::from_array(&env, &[0u8; 32]),
    );

    let status = client.get_session(&sid).map(|s| s.status);
    assert!(matches!(status, Some(SessionStatus::Finalized)));
}
