use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::{
    CrisisSeverity, MentalHealthSupportContract, MentalHealthSupportContractClient, TherapyModality,
};

fn setup(env: &Env) -> (MentalHealthSupportContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MentalHealthSupportContract);
    let client = MentalHealthSupportContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    env.mock_all_auths();
    client.try_initialize(&admin).unwrap().unwrap();
    (client, admin)
}

fn queue_contains(q: &soroban_sdk::Vec<u64>, id: u64) -> bool {
    let mut i = 0u32;
    while i < q.len() {
        if q.get(i).unwrap() == id {
            return true;
        }
        i = i.saturating_add(1);
    }
    false
}

#[test]
fn enroll_mood_booking_community() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let patient = Address::generate(&env);
    client.try_enroll(&patient).unwrap().unwrap();
    assert!(client.try_is_enrolled(&patient).unwrap().unwrap());

    let mid = client
        .try_log_mood(&patient, &6u32, &BytesN::from_array(&env, &[1u8; 32]))
        .unwrap()
        .unwrap();
    let me = client.try_get_mood(&mid).unwrap().unwrap();
    assert_eq!(me.mood_score, 6);

    let sched: u64 = 500;
    let bid = client
        .try_book_teletherapy(
            &patient,
            &TherapyModality::Cbt,
            &BytesN::from_array(&env, &[2u8; 32]),
            &sched,
            &String::from_str(&env, "intro"),
        )
        .unwrap()
        .unwrap();
    let book = client.try_get_booking(&bid).unwrap().unwrap();
    assert_eq!(book.modality, TherapyModality::Cbt);

    let cid = client
        .try_create_peer_community(&admin, &String::from_str(&env, "peer-circle"))
        .unwrap()
        .unwrap();
    client
        .try_join_peer_community(&patient, &cid)
        .unwrap()
        .unwrap();
    let members = client.try_list_community_members(&cid).unwrap().unwrap();
    assert_eq!(members.len(), 1);
}

#[test]
fn crisis_is_queued() {
    let env = Env::default();
    let (mh, admin) = setup(&env);

    let tele = Address::generate(&env);
    let notif = Address::generate(&env);
    mh.try_set_integration_contracts(&admin, &tele, &notif)
        .unwrap()
        .unwrap();

    let patient = Address::generate(&env);
    mh.try_enroll(&patient).unwrap().unwrap();

    let crisis_id = mh
        .try_report_crisis(
            &patient,
            &CrisisSeverity::High,
            &BytesN::from_array(&env, &[3u8; 32]),
        )
        .unwrap()
        .unwrap();
    let c = mh.try_get_crisis(&crisis_id).unwrap().unwrap();
    assert_eq!(c.severity, CrisisSeverity::High);
    assert!(c.notification_id.is_none());

    let q = mh.try_open_crisis_queue().unwrap().unwrap();
    assert!(queue_contains(&q, crisis_id));
}
