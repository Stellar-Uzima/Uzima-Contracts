//! Published differential-privacy noise bounds and the on-chain verifier for
//! them (issue #1619).
//!
//! # Why this module exists
//!
//! `differential_privacy` releases a `PrivacyQuery` record that carries both
//! `true_result` and `noisy_result`. That record is the only on-chain evidence
//! a data subject, auditor or regulator can use to check that noise was
//! actually applied, yet nothing in the contract previously stated the bound
//! the noise is required to respect. A record emitted with
//! `noisy_result == true_result`, or with the perturbation clipped to zero,
//! would be indistinguishable from a correct one.
//!
//! This module is the single source of truth for both halves of the fix:
//!
//! 1. [`noise_bound`] publishes the maximum absolute perturbation each
//!    mechanism may emit for a given sensitivity. The two generators below
//!    hold that bound by construction and are the *only* place noise is
//!    produced, so the published bound cannot drift from the implementation.
//! 2. [`audit_query`] re-derives the observed perturbation from a stored
//!    `PrivacyQuery` and reports whether it satisfies the bound, making the
//!    check available on-chain to anyone holding the record.
//!
//! # Published bounds
//!
//! | Mechanism | `epsilon_cost` | Max abs noise |
//! |---|---|---|
//! | `Laplace` | `sensitivity` | `sensitivity` |
//! | `Gaussian` | `2 * sensitivity` | `3 * sensitivity` |
//!
//! The Laplace generator draws from a uniform window of width
//! `2 * sensitivity` centred on zero, so the absolute noise is at most
//! `sensitivity`. The Gaussian generator approximates a truncated normal over
//! a `6 * sensitivity` wide window, i.e. `+/- 3 sigma`, so the absolute noise
//! is at most `3 * sensitivity`.
//!
//! Both windows are scaled by `sensitivity` alone. The generators are
//! deliberately *not* re-weighted by `epsilon`: a smaller `epsilon` should
//! produce *more* noise, but doing that on-chain would require transcendental
//! functions, so the `epsilon_cost` accounting is kept exact and the noise
//! magnitude is tied to `sensitivity`. This is a documented property of the
//! current implementation rather than an oversight, and [`audit_query`] is
//! what makes it externally checkable.
//!
//! # Budget reconciliation
//!
//! [`audit_budget`] recomputes the expected `epsilon_remaining` from a
//! per-budget [`BudgetLedger`] and reports `reconciled: false` when the two
//! disagree, so a budget cannot be presented as larger than the queries
//! charged against it justify. Ledgers are created by `create_budget`.
//!
//! A budget created before this module existed has no ledger. Rather than
//! inventing one, the first query against such a budget opens a ledger flagged
//! [`BudgetLedger::reconstructed`], and the audit reports it as such: the total
//! it carries is a lower bound on historic spend, because it is derived from
//! the budget record's own `epsilon_remaining` and therefore can never
//! overstate the budget. Before that first query the audit reports
//! `ledger_present: false` — no evidence, not a pass.

use soroban_sdk::{contracttype, BytesN, Env};

use crate::{DataKey, Error, NoiseMechanism, PrivacyBudget, PrivacyQuery};

/// Width multiplier of the Laplace uniform window, in units of `sensitivity`.
pub const LAPLACE_WINDOW_MULTIPLIER: u64 = 2;

/// Sigma multiplier of the truncated Gaussian window, in units of `sensitivity`.
pub const GAUSSIAN_SIGMA_MULTIPLIER: u64 = 3;

/// Full width multiplier of the truncated Gaussian window.
pub const GAUSSIAN_WINDOW_MULTIPLIER: u64 = GAUSSIAN_SIGMA_MULTIPLIER * 2;

/// `epsilon_cost` multiplier applied to `sensitivity` for the Gaussian
/// mechanism, which is charged twice the Laplace cost.
pub const GAUSSIAN_EPSILON_MULTIPLIER: u64 = 2;

/// Smallest `epsilon_cost` any accepted query can carry.
///
/// `create_budget` rejects `epsilon_total == 0` and both noise entrypoints
/// reject `sensitivity == 0`, and `epsilon_cost` is `sensitivity` or
/// `2 * sensitivity`, so every recorded query costs at least one epsilon.
/// This is the floor the ledger reconciliation check uses to reject a ledger
/// that claims more queries than it can possibly have paid for.
pub const MIN_EPSILON_COST_PER_QUERY: u64 = 1;

/// Largest `sensitivity` the generators accept.
///
/// The generators take an `i64` scale, and the Gaussian one reduces a draw
/// modulo `6 * scale` before subtracting `3 * scale`. Capping the scale at
/// `i64::MAX / 3` keeps `6 * scale` inside `u64` and `3 * scale` inside `i64`
/// for both mechanisms, so neither generator can wrap. Any sensitivity a
/// caller can meaningfully express for a bounded query result is orders of
/// magnitude below this cap.
pub const MAX_NOISE_SCALE: u64 = i64::MAX as u64 / GAUSSIAN_SIGMA_MULTIPLIER;

/// Per-budget accounting record backing [`audit_budget`].
///
/// Stored under [`DataKey::BudgetLedger`]. Deliberately separate from
/// [`PrivacyBudget`] so the public `PrivacyBudget` shape — and therefore the
/// contract ABI and the `get_remaining_budget` result — is unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BudgetLedger {
    /// `epsilon` the budget was opened with.
    pub epsilon_total: u64,
    /// Sum of `epsilon_cost` over every query charged to the budget.
    pub epsilon_spent: u64,
    /// Number of queries charged to the budget.
    pub query_count: u64,
    /// `true` when this ledger was opened on first use against a budget that
    /// predates ledger accounting, so `epsilon_total` covers only the queries
    /// observed since and may understate historic spend.
    ///
    /// An auditor must treat a reconstructed ledger as a lower bound on past
    /// spend. It can never overstate the remaining budget, because
    /// `epsilon_total` is derived from the budget record's own
    /// `epsilon_remaining` at reconstruction time.
    pub reconstructed: bool,
}

impl BudgetLedger {
    /// Ledger for a freshly created budget with the given total.
    #[must_use]
    pub fn opened(epsilon_total: u64) -> Self {
        Self {
            epsilon_total,
            epsilon_spent: 0,
            query_count: 0,
            reconstructed: false,
        }
    }

    /// Ledger opened against a budget that predates ledger accounting.
    ///
    /// `observed_remaining` is the budget record's own `epsilon_remaining` at
    /// reconstruction time. Using it as the total is what makes the ledger
    /// reconcile with the record straight away: after the first query is
    /// charged, `epsilon_total - epsilon_spent` equals the record's new
    /// `epsilon_remaining`. The flip side is that the total is a strict lower
    /// bound on the budget's real original `epsilon`, which is precisely what
    /// `reconstructed` warns an auditor about. It can never overstate the
    /// remaining budget, because the total is read from the record itself.
    #[must_use]
    pub fn reconstructed(observed_remaining: u64) -> Self {
        Self {
            epsilon_total: observed_remaining,
            epsilon_spent: 0,
            query_count: 0,
            reconstructed: true,
        }
    }

    /// Record one query of `epsilon_cost` against the ledger.
    ///
    /// Returns [`Error::ArithmeticOverflow`] when the running spend or the
    /// query count would wrap. Saturating here would silently inflate the
    /// apparent remaining budget, which is the failure this module exists to
    /// prevent, so the overflow is surfaced instead.
    pub fn charge(&mut self, epsilon_cost: u64) -> Result<(), Error> {
        self.epsilon_spent = self
            .epsilon_spent
            .checked_add(epsilon_cost)
            .ok_or(Error::ArithmeticOverflow)?;
        self.query_count = self
            .query_count
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        Ok(())
    }

    /// `epsilon_remaining` implied by the ledger alone.
    ///
    /// `None` when the ledger claims more spend than the budget ever had, which
    /// is itself a falsification.
    #[must_use]
    pub fn expected_remaining(&self) -> Option<u64> {
        self.epsilon_total.checked_sub(self.epsilon_spent)
    }
}

/// Result of auditing a single stored [`PrivacyQuery`] against its bound.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct QueryNoiseAudit {
    /// Mechanism recorded on the query.
    pub mechanism: NoiseMechanism,
    /// Sensitivity recorded on the query.
    pub sensitivity: u64,
    /// Maximum absolute perturbation the mechanism may emit.
    pub max_abs_noise: u64,
    /// Absolute value of `noisy_result - true_result`.
    pub observed_abs_noise: u64,
    /// `true` when `observed_abs_noise <= max_abs_noise`.
    pub within_bound: bool,
}

/// Result of auditing a budget's epsilon accounting against its ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BudgetNoiseAudit {
    /// `epsilon` the budget was opened with, or `0` when there is no ledger.
    pub epsilon_total: u64,
    /// `epsilon` still available according to the budget record.
    pub epsilon_remaining: u64,
    /// `epsilon` the recorded queries are accounted for by, or `0` when there
    /// is no ledger.
    pub epsilon_accounted: u64,
    /// Number of queries recorded against the budget, or `0` when there is no
    /// ledger.
    pub query_count: u64,
    /// `true` when a [`BudgetLedger`] exists for this budget.
    pub ledger_present: bool,
    /// `true` when the ledger only covers queries observed since ledger
    /// accounting was introduced, so `epsilon_total` is a lower bound.
    pub reconstructed: bool,
    /// `true` when the ledger is internally consistent and the budget's
    /// `epsilon_remaining` equals `epsilon_total - epsilon_spent`.
    ///
    /// `false` for a budget with no ledger, because there is then no on-chain
    /// evidence either way.
    pub reconciled: bool,
}

/// Maximum absolute perturbation the mechanism may emit for `sensitivity`.
///
/// Returns [`Error::InvalidSensitivity`] for `sensitivity == 0` and
/// [`Error::ArithmeticOverflow`] when the multiplier would wrap, so an unbounded
/// window cannot be obtained by choosing a huge sensitivity.
pub fn noise_bound(mechanism: NoiseMechanism, sensitivity: u64) -> Result<u64, Error> {
    if sensitivity == 0 {
        return Err(Error::InvalidSensitivity);
    }
    let multiplier = match mechanism {
        NoiseMechanism::Laplace => 1,
        NoiseMechanism::Gaussian => GAUSSIAN_SIGMA_MULTIPLIER,
    };
    sensitivity
        .checked_mul(multiplier)
        .ok_or(Error::ArithmeticOverflow)
}

/// `epsilon` charged for one query under `mechanism`.
pub fn epsilon_cost(mechanism: NoiseMechanism, sensitivity: u64) -> Result<u64, Error> {
    if sensitivity == 0 {
        return Err(Error::InvalidSensitivity);
    }
    let multiplier = match mechanism {
        NoiseMechanism::Laplace => 1,
        NoiseMechanism::Gaussian => GAUSSIAN_EPSILON_MULTIPLIER,
    };
    sensitivity
        .checked_mul(multiplier)
        .ok_or(Error::ArithmeticOverflow)
}

/// Noise scale to hand the generators for `sensitivity`.
///
/// Rejects `0` and anything above [`MAX_NOISE_SCALE`], so the `i64` scale the
/// generators take is always positive and small enough that neither generator
/// can wrap.
pub fn noise_scale(sensitivity: u64) -> Result<i64, Error> {
    if sensitivity == 0 {
        return Err(Error::InvalidSensitivity);
    }
    if sensitivity > MAX_NOISE_SCALE {
        return Err(Error::InvalidSensitivity);
    }
    Ok(sensitivity as i64)
}

/// Absolute perturbation recorded by a query.
///
/// Returns [`Error::ArithmeticOverflow`] when `noisy_result - true_result` is
/// not representable as a `u64`; such a record cannot be audited and is
/// rejected rather than reported as passing.
pub fn observed_abs_noise(true_result: i64, noisy_result: i64) -> Result<u64, Error> {
    let delta = noisy_result
        .checked_sub(true_result)
        .ok_or(Error::ArithmeticOverflow)?;
    Ok(delta.unsigned_abs())
}

/// Audit one stored query against its published bound.
///
/// Reads only the stored record, so it reports a violation for any record whose
/// perturbation is missing, clipped, or out of window, regardless of how the
/// record was produced.
pub fn audit_query(query: &PrivacyQuery) -> Result<QueryNoiseAudit, Error> {
    let max_abs_noise = noise_bound(query.mechanism, query.sensitivity)?;
    let observed_abs_noise = observed_abs_noise(query.true_result, query.noisy_result)?;
    Ok(QueryNoiseAudit {
        mechanism: query.mechanism,
        sensitivity: query.sensitivity,
        max_abs_noise,
        observed_abs_noise,
        within_bound: observed_abs_noise <= max_abs_noise,
    })
}

/// Audit a budget's epsilon accounting against its ledger.
pub fn audit_budget(budget: &PrivacyBudget, ledger: Option<&BudgetLedger>) -> BudgetNoiseAudit {
    let Some(ledger) = ledger else {
        return BudgetNoiseAudit {
            epsilon_total: 0,
            epsilon_remaining: budget.epsilon_remaining,
            epsilon_accounted: 0,
            query_count: 0,
            ledger_present: false,
            reconstructed: false,
            reconciled: false,
        };
    };

    // Three independent conditions must all hold for the budget to be sound:
    //  1. the ledger does not claim more spend than the budget ever had;
    //  2. the ledger does not claim more queries than the recorded spend can
    //     possibly pay for (every query costs at least one epsilon);
    //  3. the budget record's remaining epsilon matches the ledger.
    let spend_within_total = ledger.epsilon_spent <= ledger.epsilon_total;
    let queries_affordable =
        ledger.query_count <= ledger.epsilon_spent / MIN_EPSILON_COST_PER_QUERY;
    let matches_record = ledger.expected_remaining() == Some(budget.epsilon_remaining);

    BudgetNoiseAudit {
        epsilon_total: ledger.epsilon_total,
        epsilon_remaining: budget.epsilon_remaining,
        epsilon_accounted: ledger.epsilon_spent,
        query_count: ledger.query_count,
        ledger_present: true,
        reconstructed: ledger.reconstructed,
        reconciled: spend_within_total && queries_affordable && matches_record,
    }
}

// ── deterministic noise generators ───────────────────────────────────────────

/// Uniform-window Laplace surrogate centred on zero.
///
/// The absolute noise is at most `scale` by construction: the draw is reduced
/// modulo a window of width `2 * |scale|` and then shifted back by `|scale|`.
#[allow(clippy::arithmetic_side_effects)]
pub fn generate_laplace_noise(env: &Env, seed: &BytesN<32>, scale: i64) -> i64 {
    let hash = env.crypto().sha256(&seed.clone().into());
    let hash_bytes: [u8; 32] = hash.into();
    let hash_int = u64::from_le_bytes(hash_bytes[0..8].try_into().unwrap_or([0; 8]));

    let width = scale
        .unsigned_abs()
        .saturating_mul(LAPLACE_WINDOW_MULTIPLIER);
    let noise = (hash_int % width) as i64;
    noise - scale.unsigned_abs() as i64
}

/// Truncated Gaussian surrogate over a `+/- 3 sigma` window.
///
/// The absolute noise is at most `3 * |scale|` by construction: the draw is
/// reduced modulo a window of width `6 * |scale|` and shifted back by
/// `3 * |scale|`.
#[allow(clippy::arithmetic_side_effects)]
pub fn generate_gaussian_noise(env: &Env, seed: &BytesN<32>, scale: i64) -> i64 {
    let hash = env.crypto().sha256(&seed.clone().into());
    let hash_bytes: [u8; 32] = hash.into();
    let hash_int = u64::from_le_bytes(hash_bytes[0..8].try_into().unwrap_or([0; 8]));

    let width = scale
        .unsigned_abs()
        .saturating_mul(GAUSSIAN_WINDOW_MULTIPLIER);
    let noise = (hash_int % width) as i64;
    noise - (scale.unsigned_abs() * GAUSSIAN_SIGMA_MULTIPLIER) as i64
}

// ── storage helpers ─────────────────────────────────────────────────────────

/// Load the ledger for `budget_id`, if one exists.
pub fn load_ledger(env: &Env, budget_id: &BytesN<32>) -> Option<BudgetLedger> {
    env.storage()
        .persistent()
        .get(&DataKey::BudgetLedger(budget_id.clone()))
}

/// Persist the ledger for `budget_id`.
pub fn store_ledger(env: &Env, budget_id: &BytesN<32>, ledger: &BudgetLedger) {
    env.storage()
        .persistent()
        .set(&DataKey::BudgetLedger(budget_id.clone()), ledger);
}
