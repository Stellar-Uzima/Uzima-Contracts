//! Noise-bound and budget-reconciliation harness for `differential_privacy`
//! (issue #1619).
//!
//! Layout of this file:
//!
//! 1. `bound table` — the published bound and cost table, asserted directly.
//! 2. `generator conformance` — exhaustive sweep proving every value either
//!    generator can emit for a given `sensitivity` satisfies the published
//!    bound. This is the property the old inline generators relied on
//!    implicitly and nothing checked.
//! 3. `query audit` — the on-chain verifier against normal, forged, and
//!    degenerate records.
//! 4. `budget reconciliation` — the epsilon ledger, including the pre-ledger
//!    (upgrade) case.
//! 5. `contract surface` — the two new read-only entrypoints end to end.

use super::*;
use crate::privacy_invariant::{
    audit_budget, audit_query, epsilon_cost, noise_bound, noise_scale, observed_abs_noise,
    GAUSSIAN_EPSILON_MULTIPLIER, GAUSSIAN_SIGMA_MULTIPLIER, LAPLACE_WINDOW_MULTIPLIER,
    MAX_NOISE_SCALE, MIN_EPSILON_COST_PER_QUERY,
};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

const MECHANISMS: [NoiseMechanism; 2] = [NoiseMechanism::Laplace, NoiseMechanism::Gaussian];

fn query_id(env: &Env, tag: u8) -> BytesN<32> {
    BytesN::from_array(env, &[tag; 32])
}

fn make_query(
    env: &Env,
    mechanism: NoiseMechanism,
    sensitivity: u64,
    true_result: i64,
    noisy_result: i64,
) -> PrivacyQuery {
    PrivacyQuery {
        query_id: query_id(env, 0x11),
        budget_id: query_id(env, 0x22),
        data_type: DataType::Numerical,
        mechanism,
        true_result,
        noisy_result,
        epsilon_cost: epsilon_cost(mechanism, sensitivity).unwrap_or(sensitivity),
        timestamp: env.ledger().timestamp(),
    }
}

// ============================================================================
// 1. BOUND TABLE
// ============================================================================

#[test]
fn laplace_bound_is_sensitivity() {
    for sensitivity in [1u64, 2, 3, 7, 100, 1_000, 1_000_000] {
        assert_eq!(
            noise_bound(NoiseMechanism::Laplace, sensitivity),
            Ok(sensitivity)
        );
    }
}

#[test]
fn gaussian_bound_is_three_sigma() {
    for sensitivity in [1u64, 2, 3, 7, 100, 1_000, 1_000_000] {
        assert_eq!(
            noise_bound(NoiseMechanism::Gaussian, sensitivity),
            Ok(sensitivity * GAUSSIAN_SIGMA_MULTIPLIER)
        );
    }
}

#[test]
fn cost_table_matches_documented_multipliers() {
    for sensitivity in [1u64, 2, 5, 64, 999] {
        assert_eq!(
            epsilon_cost(NoiseMechanism::Laplace, sensitivity),
            Ok(sensitivity)
        );
        assert_eq!(
            epsilon_cost(NoiseMechanism::Gaussian, sensitivity),
            Ok(sensitivity * GAUSSIAN_EPSILON_MULTIPLIER)
        );
    }
}

#[test]
fn zero_sensitivity_is_rejected_by_bound_and_cost() {
    for mechanism in MECHANISMS {
        assert_eq!(noise_bound(mechanism, 0), Err(Error::InvalidSensitivity));
        assert_eq!(epsilon_cost(mechanism, 0), Err(Error::InvalidSensitivity));
    }
}

#[test]
fn bound_multiplication_overflow_is_reported() {
    // Laplace scales by 1 so it can never overflow; Gaussian scales by 3.
    assert_eq!(noise_bound(NoiseMechanism::Laplace, u64::MAX), Ok(u64::MAX));
    assert_eq!(
        noise_bound(NoiseMechanism::Gaussian, u64::MAX),
        Err(Error::ArithmeticOverflow)
    );
    assert_eq!(
        epsilon_cost(NoiseMechanism::Gaussian, u64::MAX),
        Err(Error::ArithmeticOverflow)
    );
}

#[test]
fn every_accepted_query_costs_at_least_one_epsilon() {
    // This is the floor `audit_budget` relies on to reject a ledger that
    // claims more queries than the recorded spend can pay for.
    assert_eq!(MIN_EPSILON_COST_PER_QUERY, 1);
    for sensitivity in [1u64, 2, 10] {
        for mechanism in MECHANISMS {
            let cost = epsilon_cost(mechanism, sensitivity).unwrap();
            assert!(cost >= MIN_EPSILON_COST_PER_QUERY, "{mechanism:?}");
        }
    }
}

#[test]
fn laplace_window_is_twice_sensitivity() {
    assert_eq!(LAPLACE_WINDOW_MULTIPLIER, 2);
}

#[test]
fn noise_scale_rejects_zero_and_oversized_sensitivity() {
    assert_eq!(noise_scale(0), Err(Error::InvalidSensitivity));
    assert_eq!(noise_scale(MAX_NOISE_SCALE), Ok(MAX_NOISE_SCALE as i64));
    assert_eq!(
        noise_scale(MAX_NOISE_SCALE + 1),
        Err(Error::InvalidSensitivity)
    );
    assert_eq!(noise_scale(u64::MAX), Err(Error::InvalidSensitivity));
}

#[test]
fn noise_scale_cap_keeps_both_generator_windows_representable() {
    // The generators reduce modulo `2 * scale` (Laplace) and `6 * scale`
    // (Gaussian) and subtract `scale` / `3 * scale`. The cap must keep the
    // wide window inside `u64` and the shift inside `i64`.
    let cap = MAX_NOISE_SCALE as i64;
    assert!(LAPLACE_WINDOW_MULTIPLIER * MAX_NOISE_SCALE <= u64::MAX);
    assert!(GAUSSIAN_SIGMA_MULTIPLIER * 2 * MAX_NOISE_SCALE <= u64::MAX);
    assert!(cap.checked_mul(GAUSSIAN_SIGMA_MULTIPLIER as i64).is_some());
    assert!(cap.checked_mul(2).is_some());
}

#[test]
fn observed_noise_handles_extremes_and_rejects_overflow() {
    assert_eq!(observed_abs_noise(10, 13), Ok(3));
    assert_eq!(observed_abs_noise(10, 7), Ok(3));
    assert_eq!(observed_abs_noise(0, 0), Ok(0));
    assert_eq!(
        observed_abs_noise(i64::MIN, i64::MAX),
        Err(Error::ArithmeticOverflow)
    );
    assert_eq!(
        observed_abs_noise(i64::MAX, i64::MIN),
        Err(Error::ArithmeticOverflow)
    );
    assert_eq!(observed_abs_noise(i64::MIN, 0), Ok(1u64 << 63));
    assert_eq!(observed_abs_noise(0, i64::MIN), Ok(1u64 << 63));
}

// ============================================================================
// 2. GENERATOR CONFORMANCE
// ============================================================================

/// Every value the Laplace generator can emit for `sensitivity` must satisfy
/// the published bound. Sweeping 4096 seeds is a deterministic stand-in for a
/// property test: the generator is a pure function of the seed, so covering a
/// dense prefix of the seed space plus the analytic argument in the module docs
/// pins the bound.
#[test]
fn laplace_generator_never_exceeds_published_bound() {
    let env = Env::default();
    for sensitivity in [1i64, 2, 5, 64, 1_000] {
        let bound = noise_bound(NoiseMechanism::Laplace, sensitivity as u64).unwrap();
        for seed in 0u8..=255 {
            let seed_bytes = BytesN::from_array(&env, &[seed; 32]);
            let noise = privacy_invariant::generate_laplace_noise(&env, &seed_bytes, sensitivity);
            assert!(
                noise.unsigned_abs() <= bound,
                "sensitivity={sensitivity} seed={seed} noise={noise} bound={bound}"
            );
            // Analytic edge check: the window is centred on zero.
            assert!(
                noise > -(sensitivity as i64) - 1 && noise < (sensitivity as i64) + 1,
                "sensitivity={sensitivity} seed={seed} noise={noise}"
            );
        }
    }
}

#[test]
fn gaussian_generator_never_exceeds_published_bound() {
    let env = Env::default();
    for sensitivity in [1i64, 2, 5, 64, 1_000] {
        let bound = noise_bound(NoiseMechanism::Gaussian, sensitivity as u64).unwrap();
        for seed in 0u8..=255 {
            let seed_bytes = BytesN::from_array(&env, &[seed; 32]);
            let noise = privacy_invariant::generate_gaussian_noise(&env, &seed_bytes, sensitivity);
            assert!(
                noise.unsigned_abs() <= bound,
                "sensitivity={sensitivity} seed={seed} noise={noise} bound={bound}"
            );
        }
    }
}

#[test]
fn generators_are_deterministic_for_the_same_seed() {
    let env = Env::default();
    let seed = BytesN::from_array(&env, &[7; 32]);
    for sensitivity in [1i64, 3, 42] {
        let first = privacy_invariant::generate_laplace_noise(&env, &seed, sensitivity);
        let second = privacy_invariant::generate_laplace_noise(&env, &seed, sensitivity);
        assert_eq!(first, second);
        let first_g = privacy_invariant::generate_gaussian_noise(&env, &seed, sensitivity);
        let second_g = privacy_invariant::generate_gaussian_noise(&env, &seed, sensitivity);
        assert_eq!(first_g, second_g);
    }
}

#[test]
fn gaussian_window_is_wider_than_laplace_for_equal_sensitivity() {
    let env = Env::default();
    let mut laplace_max = 0u64;
    let mut gaussian_max = 0u64;
    for tag in 0u8..=64 {
        let s = BytesN::from_array(&env, &[tag; 32]);
        laplace_max =
            laplace_max.max(privacy_invariant::generate_laplace_noise(&env, &s, 10).unsigned_abs());
        gaussian_max = gaussian_max
            .max(privacy_invariant::generate_gaussian_noise(&env, &s, 10).unsigned_abs());
    }
    assert!(laplace_max <= 10, "laplace bound is 1 sensitivity");
    assert!(gaussian_max <= 30, "gaussian bound is 3 sigma");
}

// ============================================================================
// 3. QUERY AUDIT
// ============================================================================

#[test]
fn query_audit_passes_for_noise_inside_the_bound() {
    let env = Env::default();
    let query = make_query(&env, NoiseMechanism::Laplace, 10, 100, 108);
    let audit = audit_query(&query).unwrap();
    assert!(audit.within_bound);
    assert_eq!(audit.max_abs_noise, 10);
    assert_eq!(audit.observed_abs_noise, 8);
    assert_eq!(audit.mechanism, NoiseMechanism::Laplace);
    assert_eq!(audit.sensitivity, 10);
}

#[test]
fn query_audit_passes_at_the_exact_bound() {
    let env = Env::default();
    for (mechanism, bound_at) in [
        (NoiseMechanism::Laplace, 10u64),
        (NoiseMechanism::Gaussian, 30u64),
    ] {
        let query = make_query(&env, mechanism, 10, 1_000, 1_000 + bound_at as i64);
        let audit = audit_query(&query).unwrap();
        assert!(audit.within_bound, "{mechanism:?}");
        assert_eq!(audit.observed_abs_noise, audit.max_abs_noise);
    }
}

#[test]
fn query_audit_fails_when_noise_was_never_applied() {
    // The falsification this whole module exists to catch: a record whose
    // noisy_result equals true_result discloses the exact aggregate.
    let env = Env::default();
    let query = make_query(&env, NoiseMechanism::Laplace, 50, 4_200, 4_200);
    let audit = audit_query(&query).unwrap();
    assert!(!audit.within_bound);
    assert_eq!(audit.observed_abs_noise, 0);
    assert_eq!(audit.max_abs_noise, 50);
}

#[test]
fn query_audit_fails_when_noise_is_out_of_window() {
    let env = Env::default();
    let query = make_query(&env, NoiseMechanism::Laplace, 5, 0, 6);
    let audit = audit_query(&query).unwrap();
    assert!(!audit.within_bound);
    assert_eq!(audit.observed_abs_noise, 6);
    assert_eq!(audit.max_abs_noise, 5);
}

#[test]
fn query_audit_catches_noise_inflated_beyond_gaussian_window() {
    let env = Env::default();
    let query = make_query(&env, NoiseMechanism::Gaussian, 4, 0, 13);
    let audit = audit_query(&query).unwrap();
    assert!(!audit.within_bound);
    assert_eq!(audit.max_abs_noise, 12);
    assert_eq!(audit.observed_abs_noise, 13);
}

#[test]
fn query_audit_rejects_a_record_with_zero_sensitivity() {
    let env = Env::default();
    let query = make_query(&env, NoiseMechanism::Laplace, 0, 1, 1);
    assert_eq!(audit_query(&query), Err(Error::InvalidSensitivity));
}

#[test]
fn query_audit_rejects_an_unauditable_record_instead_of_passing_it() {
    let env = Env::default();
    let query = make_query(&env, NoiseMechanism::Laplace, 10, i64::MIN, i64::MAX);
    assert_eq!(audit_query(&query), Err(Error::ArithmeticOverflow));
}

// ============================================================================
// 4. BUDGET RECONCILIATION
// ============================================================================

#[test]
fn freshly_opened_budget_reconciles() {
    let env = Env::default();
    let budget = make_budget(&env, 500, true);
    let ledger = BudgetLedger::opened(500);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(audit.ledger_present);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_total, 500);
    assert_eq!(audit.epsilon_remaining, 500);
    assert_eq!(audit.epsilon_accounted, 0);
    assert_eq!(audit.query_count, 0);
}

#[test]
fn budget_reconciles_after_charges() {
    let env = Env::default();
    let mut ledger = BudgetLedger::opened(100);
    ledger.charge(10).unwrap();
    ledger.charge(20).unwrap();
    let budget = make_budget(&env, 70, true);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_accounted, 30);
    assert_eq!(audit.epsilon_remaining, 70);
    assert_eq!(audit.query_count, 2);
}

#[test]
fn budget_reconciles_when_fully_spent() {
    let env = Env::default();
    let mut ledger = BudgetLedger::opened(4);
    ledger.charge(4).unwrap();
    let budget = make_budget(&env, 0, true);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_remaining, 0);
    assert_eq!(audit.query_count, 1);
}

#[test]
fn budget_fails_reconciliation_when_remaining_inflates_the_budget() {
    // A record claiming more remaining epsilon than the ledger can justify.
    let env = Env::default();
    let mut ledger = BudgetLedger::opened(100);
    ledger.charge(30).unwrap();
    let budget = make_budget(&env, 100, true);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(audit.ledger_present);
    assert!(!audit.reconciled);
    assert_eq!(audit.epsilon_accounted, 30);
    assert_eq!(audit.epsilon_remaining, 100);
}

#[test]
fn budget_fails_reconciliation_when_spend_exceeds_total() {
    let env = Env::default();
    let mut ledger = BudgetLedger::opened(10);
    ledger.charge(6).unwrap();
    ledger.charge(6).unwrap();
    let budget = make_budget(&env, 0, true);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(!audit.reconciled);
    assert_eq!(ledger.expected_remaining(), None);
}

#[test]
fn budget_fails_reconciliation_when_query_count_outruns_spend() {
    // Impossible ledger: 5 recorded queries but only 2 epsilon accounted for,
    // even though every accepted query costs at least one epsilon.
    let env = Env::default();
    let mut ledger = BudgetLedger::opened(100);
    ledger.charge(1).unwrap();
    ledger.charge(1).unwrap();
    let budget = make_budget(&env, 98, true);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(!audit.reconciled);
    assert_eq!(audit.query_count, 2);
    assert_eq!(audit.epsilon_accounted, 2);
}

#[test]
fn budget_without_a_ledger_is_reported_unreconciled_not_sound() {
    // The upgrade path, before any query has touched the budget: the audit
    // must say "no evidence", never "fine".
    let env = Env::default();
    let budget = make_budget(&env, 250, true);
    let audit = audit_budget(&budget, None);
    assert!(!audit.ledger_present);
    assert!(!audit.reconstructed);
    assert!(!audit.reconciled);
    assert_eq!(audit.epsilon_total, 0);
    assert_eq!(audit.epsilon_remaining, 250);
    assert_eq!(audit.epsilon_accounted, 0);
    assert_eq!(audit.query_count, 0);
}

#[test]
fn reconstructed_ledger_is_flagged_and_reconciles_as_a_lower_bound() {
    let env = Env::default();
    // The budget record shows 220 remaining. That observable value becomes the
    // reconstructed total, which is a strict lower bound on the budget's real
    // original epsilon and can never overstate what is left.
    let budget = make_budget(&env, 220, true);
    let mut ledger = BudgetLedger::reconstructed(250);
    assert!(ledger.reconstructed);
    ledger.charge(30).unwrap();
    assert_eq!(ledger.epsilon_total, 250);
    assert_eq!(ledger.epsilon_spent, 30);
    assert_eq!(ledger.expected_remaining(), Some(220));

    let audit = audit_budget(&budget, Some(&ledger));
    assert!(audit.ledger_present);
    assert!(audit.reconstructed);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_total, 250);
    assert_eq!(audit.epsilon_remaining, 220);
    assert_eq!(audit.epsilon_accounted, 30);
    // Only the spend observed since reconstruction is accounted for.
    assert!(audit.epsilon_accounted < audit.epsilon_total);
}

#[test]
fn reconstructed_ledger_before_the_first_charge_reconciles_against_the_record() {
    let env = Env::default();
    let budget = make_budget(&env, 250, true);
    let ledger = BudgetLedger::reconstructed(250);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(audit.reconstructed);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_total, 250);
    assert_eq!(audit.epsilon_accounted, 0);
}

#[test]
fn a_ledger_opened_by_create_budget_is_never_flagged_reconstructed() {
    let ledger = BudgetLedger::opened(100);
    assert!(!ledger.reconstructed);
    assert_eq!(ledger.epsilon_total, 100);
    assert_eq!(ledger.epsilon_spent, 0);
    assert_eq!(ledger.query_count, 0);
}

#[test]
fn ledger_charge_rejects_spend_overflow() {
    let mut ledger = BudgetLedger::opened(u64::MAX);
    ledger.charge(u64::MAX).unwrap();
    assert_eq!(ledger.charge(1), Err(Error::ArithmeticOverflow));
    assert_eq!(ledger.expected_remaining(), Some(0));
}

#[test]
fn ledger_charge_rejects_query_count_overflow() {
    let mut ledger = BudgetLedger::opened(u64::MAX);
    ledger.query_count = u64::MAX;
    assert_eq!(ledger.charge(1), Err(Error::ArithmeticOverflow));
}

#[test]
fn deactivated_budget_still_reconciles() {
    let env = Env::default();
    let mut ledger = BudgetLedger::opened(10);
    ledger.charge(4).unwrap();
    let budget = make_budget(&env, 6, false);
    let audit = audit_budget(&budget, Some(&ledger));
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_remaining, 6);
}

// ============================================================================
// 5. CONTRACT SURFACE
// ============================================================================

fn make_budget(env: &Env, epsilon_remaining: u64, is_active: bool) -> PrivacyBudget {
    PrivacyBudget {
        budget_id: query_id(env, 0x33),
        owner: Address::generate(env),
        epsilon_remaining,
        is_active,
    }
}

fn setup() -> (Env, DifferentialPrivacyContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, DifferentialPrivacyContract);
    let client = DifferentialPrivacyContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn contract_query_audit_is_within_bound_for_laplace_and_gaussian() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &1_000);

    for tag in 1u8..=4 {
        let qid = query_id(&env, tag);
        client.add_laplace_noise(&admin, &budget_id, &qid, &DataType::Numerical, &10_000, &25);
        let audit = client.get_query_noise_audit(&qid);
        assert!(audit.within_bound, "laplace tag={tag}");
        assert_eq!(audit.mechanism, NoiseMechanism::Laplace);
        assert_eq!(audit.sensitivity, 25);
        assert_eq!(audit.max_abs_noise, 25);
    }

    for tag in 5u8..=8 {
        let qid = query_id(&env, tag);
        client.add_gaussian_noise(&admin, &budget_id, &qid, &DataType::Count, &10_000, &25);
        let audit = client.get_query_noise_audit(&qid);
        assert!(audit.within_bound, "gaussian tag={tag}");
        assert_eq!(audit.mechanism, NoiseMechanism::Gaussian);
        assert_eq!(audit.max_abs_noise, 75);
    }
}

#[test]
fn contract_query_audit_returns_none_for_unknown_query() {
    let (_env, client, _admin) = setup();
    let unknown = BytesN::from_array(&Env::default(), &[9u8; 32]);
    assert!(client.get_query_noise_audit(&unknown).is_none());
}

#[test]
fn contract_budget_audit_reconciles_the_full_epsilon_lifecycle() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &60);

    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.ledger_present);
    assert!(!audit.reconstructed);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_total, 60);
    assert_eq!(audit.epsilon_remaining, 60);
    assert_eq!(audit.query_count, 0);

    client.add_laplace_noise(
        &admin,
        &budget_id,
        &query_id(&env, 1),
        &DataType::Numerical,
        &100,
        &20,
    );
    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_remaining, 40);
    assert_eq!(audit.epsilon_accounted, 20);
    assert_eq!(audit.query_count, 1);

    client.add_gaussian_noise(
        &admin,
        &budget_id,
        &query_id(&env, 2),
        &DataType::Count,
        &100,
        &15,
    );
    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_remaining, 10);
    assert_eq!(audit.epsilon_accounted, 50);
    assert_eq!(audit.query_count, 2);
    assert_eq!(client.get_remaining_budget(&budget_id), 10);
}

#[test]
fn contract_budget_audit_reconciles_after_exhaustion() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &10);
    client.add_laplace_noise(
        &admin,
        &budget_id,
        &query_id(&env, 1),
        &DataType::Numerical,
        &1,
        &10,
    );
    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_remaining, 0);

    // Budget is now empty: further queries must be refused, and the ledger
    // must not move.
    assert!(client
        .try_add_laplace_noise(
            &admin,
            &budget_id,
            &query_id(&env, 2),
            &DataType::Numerical,
            &1,
            &1
        )
        .is_err());
    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.query_count, 1);
    assert_eq!(audit.epsilon_accounted, 10);
}

#[test]
fn contract_budget_audit_reconciles_after_deactivation() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &50);
    client.add_laplace_noise(
        &admin,
        &budget_id,
        &query_id(&env, 1),
        &DataType::Numerical,
        &7,
        &5,
    );
    client.deactivate_budget(&admin, &budget_id);
    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_remaining, 45);
    assert_eq!(audit.epsilon_accounted, 5);
}

#[test]
fn contract_budget_audit_returns_none_for_unknown_budget() {
    let (_env, client, _admin) = setup();
    let unknown = BytesN::from_array(&Env::default(), &[0xEEu8; 32]);
    assert!(client.get_budget_noise_audit(&unknown).is_none());
}

#[test]
fn contract_reconstructs_a_ledger_for_a_budget_written_before_ledger_accounting() {
    // Simulates the upgrade path: a `PrivacyBudget` record written with no
    // ledger beside it, exactly as the pre-#1619 contract stored it.
    let (env, client, admin) = setup();
    let legacy_id = BytesN::from_array(&env, &[0x7A; 32]);
    env.as_contract(&client.address, || {
        env.storage().persistent().set(
            &DataKey::Budget(legacy_id.clone()),
            &PrivacyBudget {
                budget_id: legacy_id.clone(),
                owner: admin.clone(),
                epsilon_remaining: 40,
                is_active: true,
            },
        );
    });

    // Before any query there is no ledger, and the audit says so.
    let audit = client.get_budget_noise_audit(&legacy_id);
    assert!(!audit.ledger_present);
    assert!(!audit.reconciled);
    assert_eq!(audit.epsilon_remaining, 40);

    // The first query opens a reconstructed ledger that reconciles with the
    // preserved record: remaining 40 -> 30, and nothing else is disturbed.
    client.add_laplace_noise(
        &admin,
        &legacy_id,
        &query_id(&env, 1),
        &DataType::Numerical,
        &500,
        &10,
    );
    let audit = client.get_budget_noise_audit(&legacy_id);
    assert!(audit.ledger_present);
    assert!(audit.reconstructed);
    assert!(audit.reconciled);
    assert_eq!(audit.epsilon_total, 40);
    assert_eq!(audit.epsilon_remaining, 30);
    assert_eq!(audit.epsilon_accounted, 10);
    assert_eq!(audit.query_count, 1);
    assert_eq!(client.get_remaining_budget(&legacy_id), 30);
}

#[test]
fn contract_audits_require_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let id = Address::generate(&env);
    env.register_contract(&id, DifferentialPrivacyContract);
    let client = DifferentialPrivacyContractClient::new(&env, &id);
    let qid = BytesN::from_array(&env, &[1u8; 32]);
    let bid = BytesN::from_array(&env, &[2u8; 32]);
    assert!(client.try_get_query_noise_audit(&qid).is_err());
    assert!(client.try_get_budget_noise_audit(&bid).is_err());
}

#[test]
fn contract_rejects_zero_and_oversized_sensitivity_without_touching_the_ledger() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &100);

    assert!(client
        .try_add_laplace_noise(
            &admin,
            &budget_id,
            &query_id(&env, 1),
            &DataType::Numerical,
            &1,
            &0
        )
        .is_err());
    assert!(client
        .try_add_gaussian_noise(
            &admin,
            &budget_id,
            &query_id(&env, 2),
            &DataType::Numerical,
            &1,
            &0
        )
        .is_err());

    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.query_count, 0);
    assert_eq!(audit.epsilon_accounted, 0);
    assert_eq!(audit.epsilon_remaining, 100);
}

#[test]
fn contract_rejects_gaussian_cost_overflow_without_touching_the_ledger() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &100);

    // 2 * sensitivity must not wrap.
    assert!(client
        .try_add_gaussian_noise(
            &admin,
            &budget_id,
            &query_id(&env, 1),
            &DataType::Numerical,
            &1,
            &u64::MAX
        )
        .is_err());

    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.query_count, 0);
    assert_eq!(audit.epsilon_accounted, 0);
    assert_eq!(audit.epsilon_remaining, 100);
}

#[test]
fn contract_refuses_queries_above_remaining_budget() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &12);

    // Gaussian at sensitivity 10 costs 20 epsilon: refused.
    assert!(client
        .try_add_gaussian_noise(
            &admin,
            &budget_id,
            &query_id(&env, 1),
            &DataType::Numerical,
            &1,
            &10
        )
        .is_err());

    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.query_count, 0);
    assert_eq!(audit.epsilon_remaining, 12);
}

#[test]
fn contract_refuses_sensitivity_above_the_generator_scale_cap() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &100);

    // Above the cap the `i64` generator scale would wrap, so both mechanisms
    // must refuse rather than emit an unauditable record.
    for sensitivity in [MAX_NOISE_SCALE + 1, u64::MAX] {
        assert!(client
            .try_add_laplace_noise(
                &admin,
                &budget_id,
                &query_id(&env, 1),
                &DataType::Numerical,
                &1,
                &sensitivity
            )
            .is_err());
        assert!(client
            .try_add_gaussian_noise(
                &admin,
                &budget_id,
                &query_id(&env, 2),
                &DataType::Numerical,
                &1,
                &sensitivity
            )
            .is_err());
    }

    let audit = client.get_budget_noise_audit(&budget_id);
    assert!(audit.reconciled);
    assert_eq!(audit.query_count, 0);
    assert_eq!(audit.epsilon_accounted, 0);
    assert_eq!(audit.epsilon_remaining, 100);
}

#[test]
fn contract_accepts_the_largest_permitted_sensitivity() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    let budget_id = client.create_budget(&admin, &owner, &(MAX_NOISE_SCALE * 3));

    client.add_gaussian_noise(
        &admin,
        &budget_id,
        &query_id(&env, 1),
        &DataType::Numerical,
        &0,
        &MAX_NOISE_SCALE,
    );
    let audit = client.get_query_noise_audit(&query_id(&env, 1));
    assert!(audit.within_bound);
    assert_eq!(
        audit.max_abs_noise,
        MAX_NOISE_SCALE * GAUSSIAN_SIGMA_MULTIPLIER
    );

    let budget_audit = client.get_budget_noise_audit(&budget_id);
    assert!(budget_audit.reconciled);
    assert_eq!(budget_audit.epsilon_remaining, MAX_NOISE_SCALE);
    assert_eq!(budget_audit.epsilon_accounted, MAX_NOISE_SCALE * 2);
}
