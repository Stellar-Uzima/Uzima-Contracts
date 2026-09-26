//! Failure-path tests for audit log export (issue #1598).
//!
//! `src/test.rs` covers the export happy path (`test_export_logs`). This file
//! covers what happens when export is *not* entitled to succeed: an
//! uninitialized contract, a caller with no log-read grant, a grant that was
//! revoked, and malformed or sparse id ranges.
//!
//! Two of these tests document current behaviour that is arguably wrong rather
//! than asserting it is right; those are called out in comments at the point of
//! the assertion so they are easy to revisit.

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{BytesN, Map, Vec};

/// An initialized contract with three logs (ids 1..=3) and the given caller
/// left with *no* log-read grant.
fn setup_with_logs(env: &Env) -> (AuditTrailClient<'_>, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let contract_id = env.register_contract(None, AuditTrail);
    let client = AuditTrailClient::new(env, &contract_id);

    let mut enabled_types = Vec::new(env);
    enabled_types.push_back(AuditType::Event);
    enabled_types.push_back(AuditType::AdminAction);
    client.initialize(
        &admin,
        &AuditConfig {
            archive_threshold: 1000,
            enabled_types,
        },
    );

    let actor = Address::generate(env);
    let target = BytesN::from_array(env, &[0xABu8; 32]);
    let meta: Map<String, String> = Map::new(env);
    for _ in 0..3 {
        client.log_event(
            &actor,
            &ActionType::DataRead,
            &target,
            &OperationResult::Success,
            &meta,
        );
    }

    (client, admin)
}

// ─── Initialization ──────────────────────────────────────────────────────────

/// `export_logs` reaches `require_log_access`, which unwraps the admin key and
/// panics on an uninitialized contract rather than returning an error code.
#[test]
#[should_panic(expected = "Contract not initialized")]
fn test_export_logs_rejected_before_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, AuditTrail);
    let client = AuditTrailClient::new(&env, &contract_id);
    let caller = Address::generate(&env);

    client.export_logs(&caller, &1, &1);
}

/// The grant path reports this one properly, unlike export itself: the admin
/// key is read with `ok_or` instead of `expect`.
#[test]
fn test_grant_log_access_before_initialization_returns_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, AuditTrail);
    let client = AuditTrailClient::new(&env, &contract_id);
    let caller = Address::generate(&env);

    assert_eq!(client.try_grant_log_access(&caller, &caller), Err(Ok(300)));
}

// ─── Authorization ───────────────────────────────────────────────────────────

/// A caller that is neither the admin nor a granted reader is refused.
#[test]
#[should_panic(expected = "Caller does not have log-read access")]
fn test_export_logs_rejected_for_ungranted_caller() {
    let env = Env::default();
    let (client, _admin) = setup_with_logs(&env);
    let stranger = Address::generate(&env);

    client.export_logs(&stranger, &1, &3);
}

/// Control for the two tests above: the grant is what makes the difference, so
/// without this a broken grant would make the refusal tests pass vacuously.
#[test]
fn test_export_logs_succeeds_for_granted_reader() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);
    let reader = Address::generate(&env);

    client.grant_log_access(&admin, &reader);
    let bundle = client.export_logs(&reader, &1, &3);

    assert_eq!(bundle.logs.len(), 3);
    assert_eq!(bundle.exported_by, reader);
}

/// Revoking the grant must close the door again.
#[test]
#[should_panic(expected = "Caller does not have log-read access")]
fn test_export_logs_rejected_after_access_revoked() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);
    let reader = Address::generate(&env);

    client.grant_log_access(&admin, &reader);
    client.export_logs(&reader, &1, &3);
    client.revoke_log_access(&admin, &reader);

    client.export_logs(&reader, &1, &3);
}

/// A non-admin cannot mint a reader grant, so it cannot grant itself access.
#[test]
fn test_grant_log_access_rejected_for_non_admin() {
    let env = Env::default();
    let (client, _admin) = setup_with_logs(&env);
    let impostor = Address::generate(&env);
    let accomplice = Address::generate(&env);

    assert_eq!(
        client.try_grant_log_access(&impostor, &accomplice),
        Err(Ok(100))
    );
    assert!(!client.has_log_access(&accomplice));
}

/// `has_log_access` tracks grants and revocations, which is what the export
/// check reads.
#[test]
fn test_has_log_access_tracks_grant_and_revoke() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);
    let reader = Address::generate(&env);

    assert!(!client.has_log_access(&reader));
    client.grant_log_access(&admin, &reader);
    assert!(client.has_log_access(&reader));
    client.revoke_log_access(&admin, &reader);
    assert!(!client.has_log_access(&reader));

    // The admin holds implicit access and is not tracked in the reader set.
    assert!(!client.has_log_access(&admin));
}

// ─── Input rejection ─────────────────────────────────────────────────────────

/// A reversed range is accepted silently and yields an empty bundle. The range
/// `start..=end` is empty when `start > end`, and `integrity_hash` is the SHA-256
/// of an empty buffer, so the caller receives a structurally valid export with
/// no logs and no indication that the request was malformed.
///
/// This asserts current behaviour. An empty `logs` vec with a well-formed hash
/// and a published `EXPORT` event is indistinguishable from "the range was
/// empty", which is the wrong answer for a caller who passed a swapped range.
/// Worth an explicit `start_id <= end_id` check returning an error; recorded
/// rather than fixed here to keep this PR to test coverage.
#[test]
fn test_export_logs_with_reversed_range_returns_empty_bundle() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);

    let bundle = client.export_logs(&admin, &3, &1);

    assert_eq!(bundle.logs.len(), 0);
    // Hash of an empty input, not of the three logs that do exist.
    assert_eq!(
        bundle.integrity_hash,
        env.crypto().sha256(&Bytes::new(&env)).into()
    );
}

/// A single-id range is inclusive at both ends and must return exactly one log.
#[test]
fn test_export_logs_with_single_id_range_returns_one_log() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);

    assert_eq!(client.export_logs(&admin, &2, &2).logs.len(), 1);
}

/// Ids that were never allocated are skipped rather than treated as errors, so
/// an over-wide range silently under-reports. A caller cannot tell from the
/// bundle that ids 4..=10 do not exist.
#[test]
fn test_export_logs_skips_unallocated_ids() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);

    let bundle = client.export_logs(&admin, &1, &10);

    assert_eq!(bundle.logs.len(), 3);
    // The returned ids are the allocated ones, so a verifier can detect the gap
    // from the payload even though the call did not fail.
    assert_eq!(bundle.logs.get(0).unwrap().id, 1);
    assert_eq!(bundle.logs.get(2).unwrap().id, 3);
}

/// A range that starts past the last allocated id is empty, not an error.
#[test]
fn test_export_logs_past_end_of_log_stream_is_empty() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);

    assert_eq!(client.export_logs(&admin, &100, &200).logs.len(), 0);
}

// ─── Cross-cutting: no quota on export ───────────────────────────────────────

/// Export has no rate limit, page size cap, or cost bound, and a grant is
/// perpetual until revoked. A granted reader can pull the entire log stream in
/// unbounded ranges, and the integrity hash is computed over the whole result
/// set in host memory.
///
/// This test does not assert a limit exists — it records that none does, so the
/// gap is visible in the test suite if one is ever added.
#[test]
fn test_export_has_no_page_size_limit() {
    let env = Env::default();
    let (client, admin) = setup_with_logs(&env);
    let reader = Address::generate(&env);
    client.grant_log_access(&admin, &reader);

    // A range far larger than the three logs that exist is accepted without
    // error and without truncation signalling.
    let bundle = client.export_logs(&reader, &1, &1_000_000);

    assert_eq!(bundle.logs.len(), 3);
}
