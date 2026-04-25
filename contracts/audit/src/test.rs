use super::*;
use crate::types::{AuditConfig, AuditType};
use soroban_sdk::testutils::Address as _;

#[test]
fn test_audit_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AuditTrail);
    let client = AuditTrailClient::new(&env, &contract_id);

    let mut enabled_types = Vec::new(&env);
    enabled_types.push_back(AuditType::Event);
    enabled_types.push_back(AuditType::AdminAction);

    let config = AuditConfig {
        archive_threshold: 1000,
        enabled_types,
    };

    // 1. Initialize
    client.initialize(&admin, &config);

    // 2. Record Event
    let actor = Address::generate(&env);
    let target = Some(Address::generate(&env));
    let action_data = Bytes::from_slice(&env, b"User Registered");
    let current_hash = BytesN::from_array(&env, &[1u8; 32]);
    let metadata = Map::new(&env);

    let record_id = client.record_event(
        &actor,
        &AuditType::Event,
        &target,
        &action_data,
        &None, // No previous hash
        &current_hash,
        &metadata,
    );

    assert_eq!(record_id, 1);

    // 3. Verify Record
    let record = client.get_record(&1);
    assert_eq!(record.actor, actor);
    assert_eq!(record.audit_type, AuditType::Event);

    // 4. Verify Integrity (Rolling Hash should not be zero)
    let rolling = client.verify_integrity();
    assert_ne!(rolling, BytesN::from_array(&env, &[0u8; 32]));

    // 5. Generate Summary
    let end_time = env.ledger().timestamp() + 100;
    let summary = client.generate_summary(&0, &end_time);
    assert_eq!(summary.total_records, 1);
}

#[test]
fn test_error_codes_are_stable() {
    use crate::errors::Error;
    assert_eq!(Error::Unauthorized as u32, 100);
    assert_eq!(Error::NotInitialized as u32, 300);
    assert_eq!(Error::AlreadyInitialized as u32, 301);
    assert_eq!(Error::RecordNotFound as u32, 403);
}

#[test]
fn test_get_suggestion_returns_expected_hint() {
    use crate::errors::{get_suggestion, Error};
    use soroban_sdk::symbol_short;
    assert_eq!(
        get_suggestion(Error::Unauthorized),
        symbol_short!("CHK_AUTH")
    );
    assert_eq!(
        get_suggestion(Error::NotInitialized),
        symbol_short!("INIT_CTR")
    );
    assert_eq!(
        get_suggestion(Error::AlreadyInitialized),
        symbol_short!("ALREADY")
    );
    assert_eq!(
        get_suggestion(Error::RecordNotFound),
        symbol_short!("CHK_ID")
    );
}
