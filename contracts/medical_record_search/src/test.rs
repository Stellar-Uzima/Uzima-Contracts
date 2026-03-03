#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, Vec};

fn setup(env: &Env) -> (MedicalRecordSearchContractClient<'_>, Address) {
    let contract_id = Address::generate(env);
    env.register_contract(&contract_id, MedicalRecordSearchContract);
    let client = MedicalRecordSearchContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

fn token(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn index_entry(
    env: &Env,
    client: &MedicalRecordSearchContractClient<'_>,
    caller: &Address,
    record_id: u64,
    network: ChainId,
    ts: u64,
    category: u8,
    required: u8,
    optional: u8,
    attr: u8,
    confidential: bool,
    quality: u32,
) {
    let mut tokens = Vec::new(env);
    tokens.push_back(token(env, required));
    if optional > 0 {
        tokens.push_back(token(env, optional));
    }
    let mut attrs = Vec::new(env);
    attrs.push_back(token(env, attr));
    client.index_record(
        caller,
        &IndexInput {
            record_id,
            patient: Address::generate(env),
            network,
            created_at: ts,
            is_confidential: confidential,
            category_hash: token(env, category),
            token_hashes: tokens,
            attribute_hashes: attrs,
            encrypted_ref_hash: token(env, record_id as u8),
            quality_score_bps: quality,
        },
    );
}

#[test]
fn search_ranking_and_cache_work() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let indexer = Address::generate(&env);
    let searcher = Address::generate(&env);
    client.assign_role(
        &admin,
        &indexer,
        &(ROLE_INDEXER | ROLE_SEARCHER | ROLE_AUDITOR),
    );
    client.assign_role(&admin, &searcher, &(ROLE_SEARCHER | ROLE_AUDITOR));

    index_entry(
        &env,
        &client,
        &indexer,
        1,
        ChainId::Stellar,
        1000,
        1,
        7,
        8,
        3,
        false,
        8000,
    );
    index_entry(
        &env,
        &client,
        &indexer,
        2,
        ChainId::Stellar,
        1000,
        1,
        7,
        0,
        3,
        false,
        6000,
    );

    let mut required = Vec::new(&env);
    required.push_back(token(&env, 7));
    let mut optional = Vec::new(&env);
    optional.push_back(token(&env, 8));

    let query = SearchQuery {
        required_tokens: required,
        optional_tokens: optional,
        category_filters: Vec::new(&env),
        attribute_filters: Vec::new(&env),
        network_filters: Vec::new(&env),
        patient_filter: None,
        from_timestamp: 0,
        to_timestamp: 0,
        include_confidential: false,
        min_quality_bps: 0,
    };

    let first = client.search(&searcher, &query, &0, &10);
    assert_eq!(first.len(), 2);
    assert_eq!(first.get(0).unwrap().record_id, 1);

    let second = client.search(&searcher, &query, &0, &10);
    assert_eq!(second.len(), 2);

    let hash = client.preview_query_hash(&query);
    let cache = client.get_cache_entry(&hash);
    assert!(cache.hit_count >= 1);
}

#[test]
fn search_requires_role() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let indexer = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    client.assign_role(&admin, &indexer, &ROLE_INDEXER);

    index_entry(
        &env,
        &client,
        &indexer,
        11,
        ChainId::Polygon,
        1000,
        2,
        4,
        0,
        9,
        false,
        7000,
    );

    let mut required = Vec::new(&env);
    required.push_back(token(&env, 4));
    let q = SearchQuery {
        required_tokens: required,
        optional_tokens: Vec::new(&env),
        category_filters: Vec::new(&env),
        attribute_filters: Vec::new(&env),
        network_filters: Vec::new(&env),
        patient_filter: None,
        from_timestamp: 0,
        to_timestamp: 0,
        include_confidential: false,
        min_quality_bps: 0,
    };
    let res = client.try_search(&unauthorized, &q, &0, &10);
    assert_eq!(res, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn confidential_records_require_confidential_role() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let indexer = Address::generate(&env);
    let basic_searcher = Address::generate(&env);
    let privileged_searcher = Address::generate(&env);
    client.assign_role(&admin, &indexer, &(ROLE_INDEXER | ROLE_SEARCHER));
    client.assign_role(&admin, &basic_searcher, &ROLE_SEARCHER);
    client.assign_role(
        &admin,
        &privileged_searcher,
        &(ROLE_SEARCHER | ROLE_CONFIDENTIAL),
    );

    index_entry(
        &env,
        &client,
        &indexer,
        33,
        ChainId::Ethereum,
        1000,
        5,
        6,
        0,
        2,
        true,
        9000,
    );

    let mut required = Vec::new(&env);
    required.push_back(token(&env, 6));
    let query = SearchQuery {
        required_tokens: required,
        optional_tokens: Vec::new(&env),
        category_filters: Vec::new(&env),
        attribute_filters: Vec::new(&env),
        network_filters: Vec::new(&env),
        patient_filter: None,
        from_timestamp: 0,
        to_timestamp: 0,
        include_confidential: true,
        min_quality_bps: 0,
    };

    let basic = client.search(&basic_searcher, &query, &0, &10);
    assert_eq!(basic.len(), 0);
    let privileged = client.search(&privileged_searcher, &query, &0, &10);
    assert_eq!(privileged.len(), 1);
}

#[test]
fn complex_filters_apply_network_and_attribute_constraints() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);
    let operator = Address::generate(&env);
    client.assign_role(&admin, &operator, &(ROLE_INDEXER | ROLE_SEARCHER));

    index_entry(
        &env,
        &client,
        &operator,
        50,
        ChainId::Polygon,
        500,
        9,
        1,
        0,
        4,
        false,
        8000,
    );
    index_entry(
        &env,
        &client,
        &operator,
        51,
        ChainId::Stellar,
        2000,
        9,
        1,
        0,
        7,
        false,
        8000,
    );

    let mut required = Vec::new(&env);
    required.push_back(token(&env, 1));
    let mut attributes = Vec::new(&env);
    attributes.push_back(token(&env, 7));
    let mut networks = Vec::new(&env);
    networks.push_back(ChainId::Stellar);

    let query = SearchQuery {
        required_tokens: required,
        optional_tokens: Vec::new(&env),
        category_filters: Vec::new(&env),
        attribute_filters: attributes,
        network_filters: networks,
        patient_filter: None,
        from_timestamp: 1000,
        to_timestamp: 3000,
        include_confidential: false,
        min_quality_bps: 0,
    };

    let results = client.search(&operator, &query, &0, &10);
    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().record_id, 51);
}
