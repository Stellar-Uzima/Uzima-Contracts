#![cfg(test)]

use super::*;
use crate::types::{ActionType, AuditConfig, AuditType, OperationResult, RetentionPolicy};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{BytesN, Map, Vec};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (AuditTrailClient<'_>, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = env.register_contract(None, AuditTrail);
    let client = AuditTrailClient::new(env, &contract_id);

    let mut enabled_types = Vec::new(env);
    enabled_types.push_back(AuditType::Event);
    enabled_types.push_back(AuditType::AdminAction);

    let config = AuditConfig {
        archive_threshold: 1000,
        enabled_types,
    };
    client.initialize(&admin, &config);
    (client, admin)
}

fn dummy_target(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0xABu8; 32])
}

fn empty_meta(env: &Env) -> Map<String, String> {
    Map::new(env)
}

// ─── Issue #399: log_event ────────────────────────────────────────────────────

#[test]
fn test_log_event_data_access() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    let id = client.log_event(
        &actor,
        &ActionType::DataRead,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);

    let log = client.get_log(&1);
    assert_eq!(log.actor, actor);
    assert_eq!(log.action, ActionType::DataRead);
    assert_eq!(log.result, OperationResult::Success);
    assert_eq!(log.target, target);
}

#[test]
fn test_log_event_permission_change() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_event(
        &actor,
        &ActionType::PermissionGrant,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);
    assert_eq!(client.get_log(&1).action, ActionType::PermissionGrant);
}

#[test]
fn test_log_event_record_modification() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_event(
        &actor,
        &ActionType::RecordUpdate,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);
    assert_eq!(client.get_log(&1).action, ActionType::RecordUpdate);
}

#[test]
fn test_log_event_auth_attempts() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    let id_ok = client.log_event(
        &actor,
        &ActionType::AuthSuccess,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );
    let id_fail = client.log_event(
        &actor,
        &ActionType::AuthFailure,
        &target,
        &OperationResult::Failure,
        &empty_meta(&env),
    );

    assert_eq!(id_ok, 1);
    assert_eq!(id_fail, 2);
    assert_eq!(client.get_log(&2).result, OperationResult::Failure);
}

#[test]
fn test_log_event_cross_chain_transfer() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_event(
        &actor,
        &ActionType::CrossChainTransferInitiated,
        &dummy_target(&env),
        &OperationResult::Pending,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);
    assert_eq!(
        client.get_log(&1).action,
        ActionType::CrossChainTransferInitiated
    );
    assert_eq!(client.get_log(&1).result, OperationResult::Pending);
}

// ─── Convenience wrappers ─────────────────────────────────────────────────────

#[test]
fn test_log_data_access_convenience() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_data_access(
        &actor,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);
    assert_eq!(client.get_log(&1).action, ActionType::DataRead);
}

#[test]
fn test_log_permission_change_convenience() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_permission_change(
        &actor,
        &ActionType::RoleAssign,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);
    assert_eq!(client.get_log(&1).action, ActionType::RoleAssign);
}

#[test]
fn test_log_auth_attempt_convenience() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_auth_attempt(
        &actor,
        &ActionType::AuthLogout,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);
    assert_eq!(client.get_log(&1).action, ActionType::AuthLogout);
}

#[test]
fn test_log_cross_chain_transfer_convenience() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_cross_chain_transfer(
        &actor,
        &ActionType::CrossChainTransferCompleted,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    assert_eq!(id, 1);
    assert_eq!(
        client.get_log(&1).action,
        ActionType::CrossChainTransferCompleted
    );
}

// ─── Querying ─────────────────────────────────────────────────────────────────

#[test]
fn test_get_logs_by_actor() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    client.log_event(
        &actor,
        &ActionType::DataRead,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );
    client.log_event(
        &actor,
        &ActionType::DataWrite,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );

    let logs = client.get_logs_by_actor(&admin, &actor);
    assert_eq!(logs.len(), 2);
}

#[test]
fn test_get_logs_by_action() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    client.log_event(
        &actor,
        &ActionType::AuthFailure,
        &target,
        &OperationResult::Failure,
        &empty_meta(&env),
    );
    client.log_event(
        &actor,
        &ActionType::AuthFailure,
        &target,
        &OperationResult::Failure,
        &empty_meta(&env),
    );
    client.log_event(
        &actor,
        &ActionType::AuthSuccess,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );

    assert_eq!(
        client
            .get_logs_by_action(&admin, &ActionType::AuthFailure)
            .len(),
        2
    );
    assert_eq!(
        client
            .get_logs_by_action(&admin, &ActionType::AuthSuccess)
            .len(),
        1
    );
}

#[test]
fn test_get_logs_by_timeframe() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let actor = Address::generate(&env);
    let t0 = env.ledger().timestamp();

    client.log_event(
        &actor,
        &ActionType::DataRead,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );

    assert_eq!(
        client
            .get_logs_by_timeframe(&admin, &t0, &(t0 + 1000))
            .len(),
        1
    );
    assert_eq!(
        client
            .get_logs_by_timeframe(&admin, &(t0 + 5000), &(t0 + 9999))
            .len(),
        0
    );
}

// ─── Access Control ───────────────────────────────────────────────────────────

#[test]
fn test_grant_and_revoke_log_access() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let reader = Address::generate(&env);

    assert!(!client.has_log_access(&reader));
    client.grant_log_access(&admin, &reader);
    assert!(client.has_log_access(&reader));
    client.revoke_log_access(&admin, &reader);
    assert!(!client.has_log_access(&reader));
}

#[test]
fn test_granted_reader_can_query() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let actor = Address::generate(&env);
    let reader = Address::generate(&env);

    client.log_event(
        &actor,
        &ActionType::DataRead,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    client.grant_log_access(&admin, &reader);

    let logs = client.get_logs_by_actor(&reader, &actor);
    assert_eq!(logs.len(), 1);
}

// ─── Retention Policy ─────────────────────────────────────────────────────────

#[test]
fn test_set_and_get_retention_policy() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let policy = RetentionPolicy {
        min_retention_seconds: 86_400,
        max_retention_seconds: 315_360_000,
    };
    client.set_retention_policy(&admin, &policy);

    let stored = client.get_retention_policy();
    assert_eq!(stored.min_retention_seconds, 86_400);
    assert_eq!(stored.max_retention_seconds, 315_360_000);
}

#[test]
fn test_verify_retention_within_window() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let id = client.log_event(
        &actor,
        &ActionType::DataRead,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );
    // max_retention_seconds = 0 means no upper bound → always valid
    assert!(client.verify_retention(&id));
}

// ─── Export Capability ────────────────────────────────────────────────────────

#[test]
fn test_export_logs() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    for _ in 0..3 {
        client.log_event(
            &actor,
            &ActionType::DataRead,
            &target,
            &OperationResult::Success,
            &empty_meta(&env),
        );
    }

    let bundle = client.export_logs(&admin, &1, &3);
    assert_eq!(bundle.logs.len(), 3);
    assert_eq!(bundle.exported_by, admin);
    assert_ne!(bundle.integrity_hash, BytesN::from_array(&env, &[0u8; 32]));
}

// ─── Integrity / Tamper-evidence ─────────────────────────────────────────────

#[test]
fn test_rolling_hash_changes_with_each_entry() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    let h0 = client.get_log_rolling_hash();
    client.log_event(
        &actor,
        &ActionType::DataRead,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );
    let h1 = client.get_log_rolling_hash();
    assert_ne!(h0, h1);

    client.log_event(
        &actor,
        &ActionType::DataWrite,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );
    let h2 = client.get_log_rolling_hash();
    assert_ne!(h1, h2);
}

#[test]
fn test_verify_log_integrity_matches_stored_hash() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    client.log_event(
        &actor,
        &ActionType::DataRead,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );
    client.log_event(
        &actor,
        &ActionType::AuthSuccess,
        &target,
        &OperationResult::Success,
        &empty_meta(&env),
    );

    let stored = client.get_log_rolling_hash();
    let recomputed = client.verify_log_integrity();
    assert_eq!(stored, recomputed);
    assert!(!client.is_log_tampered(&recomputed));
}

#[test]
fn test_is_log_tampered_detects_wrong_hash() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    client.log_event(
        &actor,
        &ActionType::DataRead,
        &dummy_target(&env),
        &OperationResult::Success,
        &empty_meta(&env),
    );

    let wrong_hash = BytesN::from_array(&env, &[0xFFu8; 32]);
    assert!(client.is_log_tampered(&wrong_hash));
}

// ─── Metadata ─────────────────────────────────────────────────────────────────

#[test]
fn test_log_event_with_metadata() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let mut meta = Map::new(&env);
    meta.set(
        String::from_str(&env, "ip_address"),
        String::from_str(&env, "192.168.1.1"),
    );
    meta.set(
        String::from_str(&env, "session_id"),
        String::from_str(&env, "sess_abc123"),
    );

    let id = client.log_event(
        &actor,
        &ActionType::DataRead,
        &dummy_target(&env),
        &OperationResult::Success,
        &meta,
    );

    let log = client.get_log(&id);
    assert_eq!(
        log.metadata.get(String::from_str(&env, "ip_address")),
        Some(String::from_str(&env, "192.168.1.1"))
    );
    assert_eq!(
        log.metadata.get(String::from_str(&env, "session_id")),
        Some(String::from_str(&env, "sess_abc123"))
    );
}

// ─── All 24 required event categories ────────────────────────────────────────

#[test]
fn test_all_required_event_categories() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let actor = Address::generate(&env);
    let target = dummy_target(&env);

    let actions = [
        ActionType::DataRead,
        ActionType::DataWrite,
        ActionType::DataDelete,
        ActionType::DataExport,
        ActionType::PermissionGrant,
        ActionType::PermissionRevoke,
        ActionType::RoleAssign,
        ActionType::RoleRevoke,
        ActionType::RecordCreate,
        ActionType::RecordUpdate,
        ActionType::RecordArchive,
        ActionType::RecordRestore,
        ActionType::AuthSuccess,
        ActionType::AuthFailure,
        ActionType::AuthLogout,
        ActionType::AuthTokenRefresh,
        ActionType::CrossChainTransferInitiated,
        ActionType::CrossChainTransferCompleted,
        ActionType::CrossChainTransferFailed,
        ActionType::CrossChainTransferReverted,
        ActionType::ConsentGranted,
        ActionType::ConsentRevoked,
        ActionType::DataBreach,
        ActionType::RetentionViolation,
    ];

    for (i, action) in actions.iter().enumerate() {
        let id = client.log_event(
            &actor,
            action,
            &target,
            &OperationResult::Success,
            &empty_meta(&env),
        );
        assert_eq!(id, (i as u64) + 1);
    }

    // Chain integrity holds after all 24 entries
    let stored = client.get_log_rolling_hash();
    let recomputed = client.verify_log_integrity();
    assert_eq!(stored, recomputed);
}
