#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Map, String, Symbol};

use audit::AuditTrail;
use audit::types::{ActionType, AuditConfig, AuditType, OperationResult};

// ---------------------------------------------------------------------------
// Mock consumer that calls the audit contract for cross-contract logging
// ---------------------------------------------------------------------------

#[contract]
struct MockHealthcareConsumer;

#[contractimpl]
impl MockHealthcareConsumer {
    pub fn access_patient_data(env: Env, audit_contract: Address, actor: Address, patient_hash: BytesN<32>) -> u64 {
        actor.require_auth();
        let audit_client = audit::AuditTrailClient::new(&env, &audit_contract);
        let mut metadata: Map<String, String> = Map::new(&env);
        metadata.set(String::from_str(&env, "resource_type"), String::from_str(&env, "patient_record"));
        let log_id = audit_client.log_event(
            &actor,
            &ActionType::DataRead,
            &patient_hash,
            &OperationResult::Success,
            &metadata,
        );
        env.events().publish(
            (Symbol::new(&env, "DATA_ACC"),),
            (actor, log_id),
        );
        log_id
    }

    pub fn write_patient_record(env: Env, audit_contract: Address, actor: Address, record_hash: BytesN<32>) -> u64 {
        actor.require_auth();
        let audit_client = audit::AuditTrailClient::new(&env, &audit_contract);
        let mut metadata: Map<String, String> = Map::new(&env);
        metadata.set(String::from_str(&env, "resource_type"), String::from_str(&env, "medical_record"));
        let log_id = audit_client.log_event(
            &actor,
            &ActionType::DataWrite,
            &record_hash,
            &OperationResult::Success,
            &metadata,
        );
        env.events().publish(
            (Symbol::new(&env, "REC_NEW"),),
            (actor, log_id),
        );
        log_id
    }

    pub fn permission_change(env: Env, audit_contract: Address, actor: Address, subject_hash: BytesN<32>, grant: bool) -> u64 {
        actor.require_auth();
        let audit_client = audit::AuditTrailClient::new(&env, &audit_contract);
        let mut metadata: Map<String, String> = Map::new(&env);
        metadata.set(String::from_str(&env, "reason"), String::from_str(&env, "provider authorization"));
        let action = if grant { ActionType::PermissionGrant } else { ActionType::PermissionRevoke };
        let log_id = audit_client.log_event(
            &actor,
            &action,
            &subject_hash,
            &OperationResult::Success,
            &metadata,
        );
        env.events().publish(
            (Symbol::new(&env, "PERM_CHG"),),
            (actor, log_id),
        );
        log_id
    }
}

// ---------------------------------------------------------------------------
// Audit cross-contract integration tests
// ---------------------------------------------------------------------------

fn setup() -> (Env, audit::AuditTrailClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, AuditTrail);
    let client = audit::AuditTrailClient::new(&env, &contract_id);

    let enabled_types: soroban_sdk::Vec<AuditType> = soroban_sdk::vec![&env, AuditType::Event, AuditType::StateChange];
    let config = AuditConfig {
        archive_threshold: 1000,
        enabled_types,
    };
    client.initialize(&admin, &config);

    (env, client, admin)
}

#[test]
fn test_cross_contract_data_access_logged() {
    let (env, _audit_client, admin) = setup();
    let mock_id = env.register_contract(None, MockHealthcareConsumer);
    let mock_client = MockHealthcareConsumerClient::new(&env, &mock_id);
    let audit_id = env.register_contract(None, AuditTrail);
    let audit_client = audit::AuditTrailClient::new(&env, &audit_id);
    let enabled_types: soroban_sdk::Vec<AuditType> = soroban_sdk::vec![&env, AuditType::Event];
    let config = AuditConfig { archive_threshold: 1000, enabled_types };
    audit_client.initialize(&admin, &config);

    let patient_hash = BytesN::from_array(&env, &[1u8; 32]);
    let log_id = mock_client.access_patient_data(&audit_id, &admin, &patient_hash);

    let log = audit_client.get_log(&log_id);
    assert_eq!(log.actor, admin);
    assert_eq!(log.action, ActionType::DataRead);
    assert_eq!(log.target, patient_hash);
    assert_eq!(log.result, OperationResult::Success);

    let events = env.events().all();
    let has_audit_event = events.iter().any(|e| {
        e.topics.get(0).and_then(|v| Symbol::try_from_val(&env, &v).ok())
            == Some(symbol_short!("AUDIT"))
    });
    let has_data_acc_event = events.iter().any(|e| {
        e.topics.get(0).and_then(|v| Symbol::try_from_val(&env, &v).ok())
            == Some(Symbol::new(&env, "DATA_ACC"))
    });
    assert!(has_audit_event, "AUDIT event must be emitted");
    assert!(has_data_acc_event, "DATA_ACC event must be emitted");
}

#[test]
fn test_cross_contract_write_logged() {
    let (env, _audit_client, admin) = setup();
    let mock_id = env.register_contract(None, MockHealthcareConsumer);
    let mock_client = MockHealthcareConsumerClient::new(&env, &mock_id);
    let audit_id = env.register_contract(None, AuditTrail);
    let audit_client = audit::AuditTrailClient::new(&env, &audit_id);
    let enabled_types: soroban_sdk::Vec<AuditType> = soroban_sdk::vec![&env, AuditType::Event];
    let config = AuditConfig { archive_threshold: 1000, enabled_types };
    audit_client.initialize(&admin, &config);

    let record_hash = BytesN::from_array(&env, &[2u8; 32]);
    let log_id = mock_client.write_patient_record(&audit_id, &admin, &record_hash);

    let log = audit_client.get_log(&log_id);
    assert_eq!(log.actor, admin);
    assert_eq!(log.action, ActionType::DataWrite);
    assert_eq!(log.target, record_hash);
    assert_eq!(log.result, OperationResult::Success);
}

#[test]
fn test_cross_contract_permission_change_logged() {
    let (env, _audit_client, admin) = setup();
    let mock_id = env.register_contract(None, MockHealthcareConsumer);
    let mock_client = MockHealthcareConsumerClient::new(&env, &mock_id);
    let audit_id = env.register_contract(None, AuditTrail);
    let audit_client = audit::AuditTrailClient::new(&env, &audit_id);
    let enabled_types: soroban_sdk::Vec<AuditType> = soroban_sdk::vec![&env, AuditType::Event];
    let config = AuditConfig { archive_threshold: 1000, enabled_types };
    audit_client.initialize(&admin, &config);

    let subject_hash = BytesN::from_array(&env, &[3u8; 32]);
    let log_id = mock_client.permission_change(&audit_id, &admin, &subject_hash, &true);

    let log = audit_client.get_log(&log_id);
    assert_eq!(log.action, ActionType::PermissionGrant);
    assert_eq!(log.result, OperationResult::Success);
}

#[test]
fn test_multiple_audit_entries_from_cross_contract() {
    let (env, _audit_client, admin) = setup();
    let mock_id = env.register_contract(None, MockHealthcareConsumer);
    let mock_client = MockHealthcareConsumerClient::new(&env, &mock_id);
    let audit_id = env.register_contract(None, AuditTrail);
    let audit_client = audit::AuditTrailClient::new(&env, &audit_id);
    let enabled_types: soroban_sdk::Vec<AuditType> = soroban_sdk::vec![&env, AuditType::Event];
    let config = AuditConfig { archive_threshold: 1000, enabled_types };
    audit_client.initialize(&admin, &config);

    let h1 = BytesN::from_array(&env, &[10u8; 32]);
    let h2 = BytesN::from_array(&env, &[11u8; 32]);
    let h3 = BytesN::from_array(&env, &[12u8; 32]);

    mock_client.access_patient_data(&audit_id, &admin, &h1);
    mock_client.write_patient_record(&audit_id, &admin, &h2);
    mock_client.permission_change(&audit_id, &admin, &h3, &false);

    let logs = audit_client.get_logs_by_actor(&admin, &admin);
    assert_eq!(logs.len(), 3);
}

#[test]
fn test_failed_operation_logged() {
    let (env, _audit_client, admin) = setup();
    let mock_id = env.register_contract(None, MockHealthcareConsumer);
    let mock_client = MockHealthcareConsumerClient::new(&env, &mock_id);
    let audit_id = env.register_contract(None, AuditTrail);
    let audit_client = audit::AuditTrailClient::new(&env, &audit_id);
    let enabled_types: soroban_sdk::Vec<AuditType> = soroban_sdk::vec![&env, AuditType::Event];
    let config = AuditConfig { archive_threshold: 1000, enabled_types };
    audit_client.initialize(&admin, &config);

    let hash = BytesN::from_array(&env, &[42u8; 32]);
    let audit_client2 = audit::AuditTrailClient::new(&env, &audit_id);
    let mut metadata: Map<String, String> = Map::new(&env);
    metadata.set(String::from_str(&env, "error"), String::from_str(&env, "record_not_found"));
    let log_id = audit_client2.log_event(
        &admin,
        &ActionType::DataRead,
        &hash,
        &OperationResult::Failure,
        &metadata,
    );

    let log = audit_client.get_log(&log_id);
    assert_eq!(log.result, OperationResult::Failure);
    let error_val = log.metadata.get(String::from_str(&env, "error")).unwrap();
    assert_eq!(error_val, String::from_str(&env, "record_not_found"));
}
