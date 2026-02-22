#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    DisputeStatus, Error, FeedKind, FeedPayload, HealthcareOracleNetwork,
    HealthcareOracleNetworkClient, RegulatoryAuthority, RegulatoryStatus, SourceType,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup_contract(
    env: &Env,
    min_submissions: u32,
) -> (HealthcareOracleNetworkClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, HealthcareOracleNetwork);
    let client = HealthcareOracleNetworkClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let arbiter = Address::generate(env);
    let arbiters = Vec::from_array(env, [arbiter.clone()]);

    client.initialize(&admin, &arbiters, &min_submissions);
    (client, admin, arbiter)
}

fn register_and_verify_oracle(
    env: &Env,
    client: &HealthcareOracleNetworkClient,
    admin: &Address,
    oracle: &Address,
    endpoint: &str,
) {
    let endpoint = String::from_str(env, endpoint);
    client.register_oracle(oracle, &endpoint, &SourceType::MarketAggregator);
    client.verify_oracle(admin, oracle, &true, &true);
}

#[test]
fn test_oracle_must_be_verified_before_submission() {
    let env = Env::default();
    let (client, _admin, _arbiter) = setup_contract(&env, 1);

    let oracle = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://oracle.example");
    let feed_id = String::from_str(&env, "NDC:0002-8215-01:US");
    let ndc = String::from_str(&env, "0002-8215-01");
    let currency = String::from_str(&env, "USD");

    client.register_oracle(&oracle, &endpoint, &SourceType::PharmaSupplier);

    let result = client.try_submit_drug_price(
        &oracle, &feed_id, &ndc, &currency, &1250i128, &900u32, &1u64,
    );

    assert_eq!(result, Err(Ok(Error::OracleNotVerified)));
}

#[test]
fn test_drug_feed_consensus_and_weighted_aggregation() {
    let env = Env::default();
    let (client, admin, _arbiter) = setup_contract(&env, 2);

    let oracle_1 = Address::generate(&env);
    let oracle_2 = Address::generate(&env);
    register_and_verify_oracle(&env, &client, &admin, &oracle_1, "https://o1.example");
    register_and_verify_oracle(&env, &client, &admin, &oracle_2, "https://o2.example");

    let feed_id = String::from_str(&env, "NDC:55513-1234-1:KE");
    let ndc = String::from_str(&env, "55513-1234-1");
    let currency = String::from_str(&env, "USD");

    let r1 = client.submit_drug_price(
        &oracle_1, &feed_id, &ndc, &currency, &1000i128, &200u32, &100u64,
    );
    let r2 = client.submit_drug_price(
        &oracle_2, &feed_id, &ndc, &currency, &1100i128, &220u32, &101u64,
    );

    assert_eq!(r1, 1);
    assert_eq!(r2, 1);

    let consensus = client
        .get_consensus(&FeedKind::DrugPricing, &feed_id)
        .expect("consensus should exist");
    assert_eq!(consensus.confidence_bps, 10_000);
    assert!(!consensus.disputed);
    assert_eq!(consensus.submitters.len(), 2);

    match consensus.payload {
        FeedPayload::DrugPrice(data) => {
            assert_eq!(data.ndc_code, ndc);
            assert_eq!(data.currency, currency);
            assert_eq!(data.price_minor, 1050);
            assert_eq!(data.availability_units, 210);
            assert_eq!(data.observed_at, 101);
        }
        _ => panic!("expected drug pricing payload"),
    }
}

#[test]
fn test_clinical_trial_and_regulatory_feeds() {
    let env = Env::default();
    let (client, admin, _arbiter) = setup_contract(&env, 1);

    let oracle = Address::generate(&env);
    register_and_verify_oracle(&env, &client, &admin, &oracle, "https://clinical.example");

    let trial_id = String::from_str(&env, "NCT-2026-001");
    let hash_a = String::from_str(&env, "sha256:trial-a");
    client.submit_clinical_trial(
        &oracle, &trial_id, &3u32, &450u32, &8200u32, &600u32, &hash_a, &500u64,
    );

    let clinical = client
        .get_consensus(&FeedKind::ClinicalTrial, &trial_id)
        .expect("clinical consensus should exist");
    match clinical.payload {
        FeedPayload::ClinicalTrial(data) => {
            assert_eq!(data.trial_id, trial_id);
            assert_eq!(data.phase, 3);
            assert_eq!(data.enrolled, 450);
        }
        _ => panic!("expected clinical payload"),
    }

    let regulation_id = String::from_str(&env, "FDA-2026-DRUG-UPDATE-11");
    let title = String::from_str(&env, "Updated Labeling Requirement");
    let details_hash = String::from_str(&env, "sha256:reg-update-11");
    client.submit_regulatory_update(
        &oracle,
        &regulation_id,
        &RegulatoryAuthority::FDA,
        &RegulatoryStatus::GuidelineUpdate,
        &title,
        &details_hash,
        &700u64,
    );

    let regulatory = client
        .get_consensus(&FeedKind::RegulatoryUpdate, &regulation_id)
        .expect("regulatory consensus should exist");
    match regulatory.payload {
        FeedPayload::RegulatoryUpdate(data) => {
            assert_eq!(data.regulation_id, regulation_id);
            assert_eq!(data.status, RegulatoryStatus::GuidelineUpdate);
            assert_eq!(data.authority, RegulatoryAuthority::FDA);
        }
        _ => panic!("expected regulatory payload"),
    }

    let bad = client.try_submit_clinical_trial(
        &oracle,
        &String::from_str(&env, "NCT-2026-BAD"),
        &5u32,
        &10u32,
        &5000u32,
        &300u32,
        &String::from_str(&env, "sha256:bad"),
        &701u64,
    );
    assert_eq!(bad, Err(Ok(Error::InvalidData)));
}

#[test]
fn test_treatment_outcome_feed_consensus() {
    let env = Env::default();
    let (client, admin, _arbiter) = setup_contract(&env, 2);

    let oracle_1 = Address::generate(&env);
    let oracle_2 = Address::generate(&env);
    register_and_verify_oracle(
        &env,
        &client,
        &admin,
        &oracle_1,
        "https://outcome-1.example",
    );
    register_and_verify_oracle(
        &env,
        &client,
        &admin,
        &oracle_2,
        "https://outcome-2.example",
    );

    let outcome_id = String::from_str(&env, "OUTCOME:CHF:ACEI:2026Q1");
    let condition = String::from_str(&env, "I50.9");
    let treatment = String::from_str(&env, "ACEI");

    client.submit_treatment_outcome(
        &oracle_1,
        &outcome_id,
        &condition,
        &treatment,
        &7200u32,
        &950u32,
        &180u32,
        &1200u32,
        &1000u64,
    );

    client.submit_treatment_outcome(
        &oracle_2,
        &outcome_id,
        &condition,
        &treatment,
        &7000u32,
        &1000u32,
        &200u32,
        &1100u32,
        &1010u64,
    );

    let consensus = client
        .get_consensus(&FeedKind::TreatmentOutcome, &outcome_id)
        .expect("treatment outcome consensus should exist");

    match consensus.payload {
        FeedPayload::TreatmentOutcome(data) => {
            assert_eq!(data.outcome_id, outcome_id);
            assert_eq!(data.condition_code, condition);
            assert_eq!(data.treatment_code, treatment);
            assert_eq!(data.improvement_rate_bps, 7100);
            assert_eq!(data.readmission_rate_bps, 975);
            assert_eq!(data.mortality_rate_bps, 190);
            assert_eq!(data.sample_size, 1150);
            assert_eq!(data.reported_at, 1010);
        }
        _ => panic!("expected treatment outcome payload"),
    }

    let invalid = client.try_submit_treatment_outcome(
        &oracle_1,
        &String::from_str(&env, "OUTCOME:BAD"),
        &condition,
        &treatment,
        &11_000u32,
        &100u32,
        &50u32,
        &0u32,
        &1011u64,
    );
    assert_eq!(invalid, Err(Ok(Error::InvalidData)));
}

#[test]
fn test_dispute_resolution_marks_consensus_and_penalizes_oracle() {
    let env = Env::default();
    let (client, admin, arbiter) = setup_contract(&env, 1);

    let oracle = Address::generate(&env);
    let challenger = Address::generate(&env);
    register_and_verify_oracle(&env, &client, &admin, &oracle, "https://reg-oracle.example");

    let regulation_id = String::from_str(&env, "EMA-2026-ALERT-44");
    client.submit_regulatory_update(
        &oracle,
        &regulation_id,
        &RegulatoryAuthority::EMA,
        &RegulatoryStatus::SafetyWarning,
        &String::from_str(&env, "Safety alert for adverse events"),
        &String::from_str(&env, "sha256:ema-alert-44"),
        &900u64,
    );

    let pre = client.get_oracle(&oracle).unwrap();
    let dispute_id = client.raise_dispute(
        &challenger,
        &FeedKind::RegulatoryUpdate,
        &regulation_id,
        &String::from_str(&env, "Mismatch with source bulletin"),
    );

    client.resolve_dispute(
        &arbiter,
        &dispute_id,
        &true,
        &String::from_str(&env, "Data source mismatch confirmed"),
        &Some(oracle.clone()),
    );

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.status, DisputeStatus::ResolvedValid);

    let post = client.get_oracle(&oracle).unwrap();
    assert!(post.reputation < pre.reputation);
    assert_eq!(post.disputes, pre.disputes + 1);

    let consensus = client
        .get_consensus(&FeedKind::RegulatoryUpdate, &regulation_id)
        .unwrap();
    assert!(consensus.disputed);
}
