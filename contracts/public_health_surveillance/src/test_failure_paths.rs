//! Failure-path tests for public-health aggregation (issue #1598).
//!
//! `src/test.rs` covers successful reporting and retrieval. This file covers the
//! rejection paths: uninitialized contract, inverted and degenerate time ranges,
//! privacy-budget exhaustion, missing records, and a provider that has not
//! authorized the call.
//!
//! The privacy budget is the only quota in this contract, and it is enforced in
//! exactly one place (`check_privacy_budget`), so the budget tests are the
//! load-bearing ones here.

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Bytes, BytesN, String};

/// Registered but *not* initialized, matching the existing `setup` helper.
fn setup_uninitialized(env: &Env) -> PublicHealthSurveillanceClient<'_> {
    let contract_id = env.register_contract(None, PublicHealthSurveillance {});
    PublicHealthSurveillanceClient::new(env, &contract_id)
}

fn setup(env: &Env) -> (PublicHealthSurveillanceClient<'_>, Address) {
    env.mock_all_auths();
    let client = setup_uninitialized(env);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

fn region(env: &Env) -> Bytes {
    Bytes::from_slice(env, b"encrypted_region_data")
}

fn data_id(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// A report with valid arguments, so a test can vary exactly one input.
#[allow(clippy::too_many_arguments)]
fn report(
    client: &PublicHealthSurveillanceClient<'_>,
    env: &Env,
    provider: &Address,
    id: &BytesN<32>,
    start: u64,
    end: u64,
    epsilon: u64,
) -> Result<(), Error> {
    // `try_` accessors return Result<Result<T, E>, Result<E, InvokeError>>: the
    // outer layer is a host/conversion failure, the inner one the contract's
    // own error. Flatten to Result<(), Error> so callers can assert on a code.
    match client.try_report_outbreak_data(
        provider,
        id,
        &region(env),
        &String::from_str(env, "A00.1"),
        &150u64,
        &start,
        &end,
        &AggregationMethod::DifferentialPrivacy,
        &epsilon,
        &8000u32,
    ) {
        Ok(result) => result,
        Err(conversion) => Err(conversion.unwrap()),
    }
}

const T0: u64 = 1_640_995_200; // 2022-01-01
const T1: u64 = 1_641_081_600; // 2022-01-02

// ─── Authorization ───────────────────────────────────────────────────────────

/// `report_outbreak_data` calls `provider.require_auth()` before anything else.
/// With no authorization recorded for the provider, the host rejects the call.
///
/// Bare `should_panic` on purpose: the message comes from the host's auth
/// machinery ("Error(Auth, InvalidAction)") rather than this contract, and
/// pinning that string would break on an SDK bump without testing anything
/// about this contract.
#[test]
#[should_panic]
fn test_report_outbreak_data_rejected_without_provider_auth() {
    let env = Env::default();
    // Deliberately no env.mock_all_auths().
    let client = setup_uninitialized(&env);
    let provider = Address::generate(&env);

    report(&client, &env, &provider, &data_id(&env, 1), T0, T1, 10);
}

// ─── Initialization ──────────────────────────────────────────────────────────

#[test]
fn test_report_outbreak_data_rejected_before_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_uninitialized(&env);
    let provider = Address::generate(&env);

    assert_eq!(report(&client, &env, &provider, &data_id(&env, 1), T0, T1, 10), Err(2));
}

#[test]
fn test_get_outbreak_data_rejected_before_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_uninitialized(&env);

    assert_eq!(client.try_get_outbreak_data(&data_id(&env, 1)), Err(Ok(2)));
}

#[test]
fn test_get_privacy_budget_rejected_before_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_uninitialized(&env);
    let provider = Address::generate(&env);

    assert_eq!(client.try_get_privacy_budget(&provider), Err(Ok(2)));
}

/// Re-initialization is guarded globally, not just per contract instance, so a
/// second `initialize` must not silently reset the stored admin.
#[test]
fn test_reinitialize_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_uninitialized(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let other = Address::generate(&env);
    assert_eq!(client.try_initialize(&other), Err(Ok(1)));
}

// ─── Input rejection: time range ─────────────────────────────────────────────

/// A zero-width period is rejected: the check is `>=`, not `>`.
#[test]
fn test_report_rejected_for_zero_width_time_range() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);

    assert_eq!(
        report(&client, &env, &provider, &data_id(&env, 1), T0, T0, 10),
        Err(14)
    );
}

#[test]
fn test_report_rejected_for_inverted_time_range() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);

    assert_eq!(
        report(&client, &env, &provider, &data_id(&env, 1), T1, T0, 10),
        Err(14)
    );
}

/// A rejected range must not consume budget or leave a record behind.
#[test]
fn test_rejected_time_range_has_no_side_effects() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);
    let id = data_id(&env, 1);

    assert!(report(&client, &env, &provider, &id, T1, T0, 500).is_err());

    // Budget untouched: the range check runs before the budget check.
    assert_eq!(client.get_privacy_budget(&provider), 1000);
    assert_eq!(client.try_get_outbreak_data(&id), Err(Ok(5)));
}

// ─── Quota: privacy budget ───────────────────────────────────────────────────

/// The default budget is 1000; anything above it is refused on the first call.
#[test]
fn test_report_rejected_when_epsilon_exceeds_default_budget() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);

    assert_eq!(
        report(&client, &env, &provider, &data_id(&env, 1), T0, T1, 1001),
        Err(7)
    );
    assert_eq!(client.get_privacy_budget(&provider), 1000);
}

/// Spending the budget exactly is allowed — the comparison is `>`.
#[test]
fn test_report_allowed_when_epsilon_equals_remaining_budget() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);

    assert!(report(&client, &env, &provider, &data_id(&env, 1), T0, T1, 1000).is_ok());
    assert_eq!(client.get_privacy_budget(&provider), 0);
}

/// ...and once it is zero, the next report is refused.
#[test]
fn test_report_rejected_once_budget_is_exhausted() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);

    report(&client, &env, &provider, &data_id(&env, 1), T0, T1, 1000).unwrap();
    assert_eq!(
        report(&client, &env, &provider, &data_id(&env, 2), T0, T1, 1),
        Err(7)
    );
}

/// Cumulative spend across several reports, not just a single oversized one.
#[test]
fn test_report_rejected_when_cumulative_spend_exhausts_budget() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);

    assert!(report(&client, &env, &provider, &data_id(&env, 1), T0, T1, 600).is_ok());
    assert_eq!(client.get_privacy_budget(&provider), 400);

    // 400 remaining is still allowed, and leaves nothing.
    assert!(report(&client, &env, &provider, &data_id(&env, 2), T0, T1, 400).is_ok());
    assert_eq!(client.get_privacy_budget(&provider), 0);

    assert_eq!(
        report(&client, &env, &provider, &data_id(&env, 3), T0, T1, 1),
        Err(7)
    );
}

/// Budgets are per provider, so one provider's spending cannot exhaust another's.
#[test]
fn test_privacy_budget_is_per_provider() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let spender = Address::generate(&env);
    let bystander = Address::generate(&env);

    report(&client, &env, &spender, &data_id(&env, 1), T0, T1, 1000).unwrap();

    assert_eq!(client.get_privacy_budget(&spender), 0);
    assert_eq!(client.get_privacy_budget(&bystander), 1000);
    assert!(report(&client, &env, &bystander, &data_id(&env, 2), T0, T1, 500).is_ok());
}

/// A report refused for budget must not write a record, otherwise the caller
/// loses the data without the budget being charged for it.
#[test]
fn test_budget_rejected_report_is_not_stored() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);
    let id = data_id(&env, 9);

    assert_eq!(report(&client, &env, &provider, &id, T0, T1, 1001), Err(7));
    assert_eq!(client.try_get_outbreak_data(&id), Err(Ok(5)));
}

/// `saturating_sub` means a zero-epsilon report is free and never underflows.
#[test]
fn test_zero_epsilon_report_is_free() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);

    assert!(report(&client, &env, &provider, &data_id(&env, 1), T0, T1, 0).is_ok());
    assert_eq!(client.get_privacy_budget(&provider), 1000);
}

// ─── Missing records ─────────────────────────────────────────────────────────

#[test]
fn test_get_outbreak_data_missing_key_returns_not_found() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    assert_eq!(client.try_get_outbreak_data(&data_id(&env, 7)), Err(Ok(5)));
}

/// The same `data_id` written twice overwrites rather than erroring, so a
/// provider can silently restate a period. Recorded because the issue asks for
/// input rejection and this is the one duplicate case that is *not* rejected.
#[test]
fn test_duplicate_data_id_overwrites_without_error() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let provider = Address::generate(&env);
    let id = data_id(&env, 1);

    report(&client, &env, &provider, &id, T0, T1, 100).unwrap();
    let first = client.get_outbreak_data(&id);

    report(&client, &env, &provider, &id, T0, T1, 100).unwrap();
    let second = client.get_outbreak_data(&id);

    assert_eq!(first.aggregated_cases, second.aggregated_cases);
    // Both writes were charged, so the caller paid twice for one stored record.
    assert_eq!(client.get_privacy_budget(&provider), 800);
}
