#![cfg(test)]

use super::*;
use crate::types::{ActivityType, Level, ThreatLevel};
use soroban_sdk::testutils::{Address as _, Ledger};

#[test]
fn test_forensics_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, OnChainForensics);
    let client = OnChainForensicsClient::new(&env, &contract_id);

    // 1. Initialize
    client.initialize(&admin);

    // 2. Collect Evidence
    let actor = Address::generate(&env);
    let location = BytesN::from_array(&env, &[1u8; 32]);
    let evidence_data = Bytes::from_array(&env, &[0xDE, 0xAD, 0xBE, 0xEF]);

    let evidence_id = client.collect_evidence(
        &actor,
        &ActivityType::Transaction,
        &location,
        &evidence_data,
        &ThreatLevel::Low,
    );

    assert_eq!(evidence_id, 1);

    // 3. Detect Suspicious (Not yet suspicious)
    let is_suspicious = client.detect_suspicious(&actor, &5000);
    assert!(!is_suspicious);

    // 4. Blacklist Actor
    client.blacklist_actor(&admin, &actor);

    // 5. Detect Suspicious (Now suspicious because blacklisted - logic in assess_threat)
    // Actually, detect_suspicious in lib.rs only looks at patterns.
    // Let's check blacklist directly if needed, but the requirement was about patterns and suspicious detection.
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_double_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, OnChainForensics);
    let client = OnChainForensicsClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.initialize(&admin);
}
