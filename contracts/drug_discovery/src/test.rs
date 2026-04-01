#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use crate::{DrugDiscoveryPlatform, DrugDiscoveryPlatformClient, Error};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup(env: &Env) -> (DrugDiscoveryPlatformClient<'_>, Address, Address, Address) {
    env.mock_all_auths();
    let id = env.register_contract(None, DrugDiscoveryPlatform);
    let client = DrugDiscoveryPlatformClient::new(env, &id);

    let admin = Address::generate(env);
    let analyzer = Address::generate(env);
    let predictor = Address::generate(env);

    assert!(client.initialize(&admin, &analyzer, &predictor));
    (client, admin, analyzer, predictor)
}

#[test]
fn test_molecular_analysis_prediction_and_adverse_effects() {
    let env = Env::default();
    let (client, _admin, analyzer, predictor) = setup(&env);

    let fingerprint = Vec::from_array(&env, [13u32, 37u32, 101u32, 211u32]);
    let db_refs = Vec::from_array(
        &env,
        [
            String::from_str(&env, "pubchem:2244"),
            String::from_str(&env, "chembl:25"),
        ],
    );

    let molecule_id = client.register_molecule(
        &analyzer,
        &String::from_str(&env, "CC(=O)OC1=CC=CC=C1C(=O)O"),
        &String::from_str(&env, "BSYNRYMUTXBXSQ-UHFFFAOYSA-N"),
        &180_160u32,
        &1u32,
        &4u32,
        &3u32,
        &fingerprint,
        &db_refs,
    );

    let analysis = client.analyze_molecular_structure(&analyzer, &molecule_id);
    assert_eq!(analysis.molecule_id, molecule_id);
    assert!(analysis.qed_score_bps > 7_000);

    let prediction_id = client.predict_drug_target_interaction(
        &predictor,
        &molecule_id,
        &String::from_str(&env, "PTGS2"),
        &950u64,
        &String::from_str(&env, "ensemble-v4"),
    );
    assert_eq!(prediction_id, 1u64);

    let adverse_id = client.predict_adverse_effects(
        &predictor,
        &molecule_id,
        &String::from_str(&env, "AE-GI-001"),
        &String::from_str(&env, "cohort:adult-general"),
        &3200u32,
    );
    assert_eq!(adverse_id, 1u64);
}

#[test]
fn test_screening_campaign_benchmark_enforcement() {
    let env = Env::default();
    let (client, _admin, analyzer, predictor) = setup(&env);

    let fp = Vec::from_array(&env, [1u32, 2u32, 3u32]);
    let refs = Vec::from_array(&env, [String::from_str(&env, "db:a")]);

    let m1 = client.register_molecule(
        &analyzer,
        &String::from_str(&env, "NCC(=O)O"),
        &String::from_str(&env, "DHMQDGOQFOQNFH-UHFFFAOYSA-N"),
        &75_067u32,
        &2u32,
        &2u32,
        &1u32,
        &fp,
        &refs,
    );

    let candidates = Vec::from_array(&env, [m1, 999u64]);
    let failed = client.try_run_screening_campaign(
        &predictor,
        &String::from_str(&env, "EGFR"),
        &candidates,
        &12u32,
        &false,
    );
    assert_eq!(failed, Err(Ok(Error::BenchmarkNotMet)));

    let good_candidates = Vec::from_array(&env, [m1]);
    let campaign_id = client.run_screening_campaign(
        &predictor,
        &String::from_str(&env, "EGFR"),
        &good_candidates,
        &8u32,
        &false,
    );

    let report = client.get_campaign_report(&campaign_id);
    assert_eq!(report.candidate_accuracy_bps, 10_000);
    assert!(report.analysis_time_hours < 24);
}

#[test]
fn test_quantum_simulation_guardrails() {
    let env = Env::default();
    let (client, admin, analyzer, predictor) = setup(&env);

    let fp = Vec::from_array(&env, [7u32, 8u32, 9u32]);
    let refs = Vec::from_array(&env, [String::from_str(&env, "db:b")]);
    let molecule_id = client.register_molecule(
        &analyzer,
        &String::from_str(&env, "C1=CC=CC=C1"),
        &String::from_str(&env, "UHOVQNZJYSORNB-UHFFFAOYSA-N"),
        &78_114u32,
        &0u32,
        &0u32,
        &0u32,
        &fp,
        &refs,
    );

    let denied = client.try_request_quantum_simulation(
        &predictor,
        &molecule_id,
        &String::from_str(&env, "ALK"),
        &String::from_str(&env, "vqe"),
        &12u32,
        &1000u32,
    );
    assert_eq!(denied, Err(Ok(Error::QuantumDisabled)));

    client.configure_integrations(&admin, &None, &None, &Some(true), &Some(true));

    let sim_id = client.request_quantum_simulation(
        &predictor,
        &molecule_id,
        &String::from_str(&env, "ALK"),
        &String::from_str(&env, "vqe"),
        &12u32,
        &1000u32,
    );

    assert_eq!(sim_id, 1u64);
}
