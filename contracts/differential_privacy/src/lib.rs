//! differential_privacy - Healthcare smart contract on Stellar blockchain.
// Differential Privacy Contract - Simplified Working Version
#![no_std]
#![forbid(alloc)]
#![allow(clippy::too_many_arguments)]

pub mod privacy_invariant;

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_noise_invariant;

use privacy_invariant::{BudgetLedger, BudgetNoiseAudit, QueryNoiseAudit};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

// =============================================================================
// Types
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum NoiseMechanism {
    Laplace,
    Gaussian,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum DataType {
    Numerical,
    Categorical,
    Count,
}

#[derive(Clone)]
#[contracttype]
pub struct PrivacyBudget {
    pub budget_id: BytesN<32>,
    pub owner: Address,
    pub epsilon_remaining: u64,
    pub is_active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct PrivacyQuery {
    pub query_id: BytesN<32>,
    pub budget_id: BytesN<32>,
    pub data_type: DataType,
    pub mechanism: NoiseMechanism,
    pub true_result: i64,
    pub noisy_result: i64,
    pub epsilon_cost: u64,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Initialized,
    Admin,
    Budget(BytesN<32>),
    Query(BytesN<32>),
    BudgetCounter,
    QueryCounter,
    /// Per-budget epsilon accounting backing the #1619 noise audit.
    /// Additive key: budgets written before this key existed simply have no
    /// entry, which `get_budget_noise_audit` reports as `ledger_present: false`.
    BudgetLedger(BytesN<32>),
}

// =============================================================================
// Errors
// =============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    BudgetNotFound = 4,
    BudgetExhausted = 5,
    BudgetNotActive = 6,
    QueryNotFound = 7,
    InvalidSensitivity = 8,
    InsufficientBudget = 9,
    InvalidInput = 10,
    ArithmeticOverflow = 11,
    /// The perturbation produced by the noise generator fell outside the bound
    /// published in `privacy_invariant` (#1619). Unreachable while the
    /// generator and the bound table agree; present so a divergence fails the
    /// write instead of publishing an unauditable record.
    NoiseBoundExceeded = 12,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::AlreadyInitialized => write!(f, "already initialized"),
            Error::NotInitialized => write!(f, "not initialized"),
            Error::NotAuthorized => write!(f, "not authorized"),
            Error::BudgetNotFound => write!(f, "budget not found"),
            Error::BudgetExhausted => write!(f, "budget exhausted"),
            Error::BudgetNotActive => write!(f, "budget not active"),
            Error::QueryNotFound => write!(f, "query not found"),
            Error::InvalidSensitivity => write!(f, "invalid sensitivity"),
            Error::InsufficientBudget => write!(f, "insufficient budget"),
            Error::InvalidInput => write!(f, "invalid input"),
            Error::ArithmeticOverflow => write!(f, "arithmetic overflow"),
            Error::NoiseBoundExceeded => write!(f, "noise bound exceeded"),
        }
    }
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct DifferentialPrivacyContract;

#[contractimpl]
impl DifferentialPrivacyContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::BudgetCounter, &0u64);
        env.storage().instance().set(&DataKey::QueryCounter, &0u64);
        Ok(())
    }

    /// Create a new privacy budget
    pub fn create_budget(
        env: Env,
        admin: Address,
        owner: Address,
        epsilon_total: u64,
    ) -> Result<BytesN<32>, Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        if epsilon_total == 0 {
            return Err(Error::InvalidInput);
        }

        let budget_id = Self::generate_budget_id(&env);
        let budget = PrivacyBudget {
            budget_id: budget_id.clone(),
            owner: owner.clone(),
            epsilon_remaining: epsilon_total,
            is_active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Budget(budget_id.clone()), &budget);
        // Open the epsilon ledger so `get_budget_noise_audit` (#1619) can
        // reconcile the budget record against the queries charged to it.
        privacy_invariant::store_ledger(&env, &budget_id, &BudgetLedger::opened(epsilon_total));

        env.events().publish(
            (symbol_short!("dp"), symbol_short!("budget")),
            (budget_id.clone(), owner, epsilon_total),
        );
        Ok(budget_id)
    }

    /// Add Laplace noise for ε-differential privacy
    pub fn add_laplace_noise(
        env: Env,
        caller: Address,
        budget_id: BytesN<32>,
        query_id: BytesN<32>,
        data_type: DataType,
        true_value: i64,
        sensitivity: u64,
    ) -> Result<PrivacyQuery, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let mut budget = Self::load_budget(&env, &budget_id)?;
        if !budget.is_active {
            return Err(Error::BudgetNotActive);
        }

        if sensitivity == 0 {
            return Err(Error::InvalidSensitivity);
        }

        // Calculate epsilon cost from the published cost table (#1619).
        let epsilon_cost = privacy_invariant::epsilon_cost(NoiseMechanism::Laplace, sensitivity)?;

        if epsilon_cost > budget.epsilon_remaining {
            return Err(Error::InsufficientBudget);
        }

        // Generate Laplace noise (simplified deterministic version)
        let scale = privacy_invariant::noise_scale(sensitivity)?;
        let noise = privacy_invariant::generate_laplace_noise(&env, &query_id, scale);
        let noisy_result = true_value
            .checked_add(noise)
            .ok_or(Error::ArithmeticOverflow)?;

        // Enforce the published Laplace bound at the write path (#1619): the
        // generator holds it by construction, so a failure here means the
        // generator and the bound table have diverged.
        Self::require_within_noise_bound(
            NoiseMechanism::Laplace,
            sensitivity,
            true_value,
            noisy_result,
        )?;

        // A budget created before ledger accounting existed has no entry: open
        // one flagged as reconstructed, seeded from the budget record's own
        // remaining epsilon so it can only understate historic spend (#1619).
        let mut ledger = privacy_invariant::load_ledger(&env, &budget_id)
            .unwrap_or_else(|| BudgetLedger::reconstructed(budget.epsilon_remaining));
        ledger.charge(epsilon_cost)?;

        // Deduct from budget
        budget.epsilon_remaining = budget
            .epsilon_remaining
            .checked_sub(epsilon_cost)
            .ok_or(Error::ArithmeticOverflow)?;

        let query = PrivacyQuery {
            query_id: query_id.clone(),
            budget_id: budget_id.clone(),
            data_type,
            mechanism: NoiseMechanism::Laplace,
            true_result: true_value,
            noisy_result,
            epsilon_cost,
            timestamp: env.ledger().timestamp(),
        };

        // Save state
        env.storage()
            .persistent()
            .set(&DataKey::Budget(budget_id.clone()), &budget);
        env.storage()
            .persistent()
            .set(&DataKey::Query(query_id.clone()), &query);
        privacy_invariant::store_ledger(&env, &budget_id, &ledger);

        env.events().publish(
            (symbol_short!("dp"), symbol_short!("laplace")),
            (query_id, budget_id, epsilon_cost),
        );
        Ok(query)
    }

    /// Add Gaussian noise for differential privacy
    pub fn add_gaussian_noise(
        env: Env,
        caller: Address,
        budget_id: BytesN<32>,
        query_id: BytesN<32>,
        data_type: DataType,
        true_value: i64,
        sensitivity: u64,
    ) -> Result<PrivacyQuery, Error> {
        caller.require_auth();
        Self::require_initialized(&env)?;

        let mut budget = Self::load_budget(&env, &budget_id)?;
        if !budget.is_active {
            return Err(Error::BudgetNotActive);
        }

        if sensitivity == 0 {
            return Err(Error::InvalidSensitivity);
        }

        // Gaussian mechanism has higher cost (2x sensitivity)
        let epsilon_cost = privacy_invariant::epsilon_cost(NoiseMechanism::Gaussian, sensitivity)?;

        if epsilon_cost > budget.epsilon_remaining {
            return Err(Error::InsufficientBudget);
        }

        // Generate Gaussian noise (simplified)
        let scale = privacy_invariant::noise_scale(sensitivity)?;
        let noise = privacy_invariant::generate_gaussian_noise(&env, &query_id, scale);
        let noisy_result = true_value
            .checked_add(noise)
            .ok_or(Error::ArithmeticOverflow)?;

        // Enforce the published Gaussian bound at the write path (#1619).
        Self::require_within_noise_bound(
            NoiseMechanism::Gaussian,
            sensitivity,
            true_value,
            noisy_result,
        )?;

        // A budget created before ledger accounting existed has no entry: open
        // one flagged as reconstructed, seeded from the budget record's own
        // remaining epsilon so it can only understate historic spend (#1619).
        let mut ledger = privacy_invariant::load_ledger(&env, &budget_id)
            .unwrap_or_else(|| BudgetLedger::reconstructed(budget.epsilon_remaining));
        ledger.charge(epsilon_cost)?;

        // Deduct from budget
        budget.epsilon_remaining = budget
            .epsilon_remaining
            .checked_sub(epsilon_cost)
            .ok_or(Error::ArithmeticOverflow)?;

        let query = PrivacyQuery {
            query_id: query_id.clone(),
            budget_id: budget_id.clone(),
            data_type,
            mechanism: NoiseMechanism::Gaussian,
            true_result: true_value,
            noisy_result,
            epsilon_cost,
            timestamp: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Budget(budget_id.clone()), &budget);
        env.storage()
            .persistent()
            .set(&DataKey::Query(query_id.clone()), &query);
        privacy_invariant::store_ledger(&env, &budget_id, &ledger);

        env.events().publish(
            (symbol_short!("dp"), symbol_short!("gaussian")),
            (query_id, budget_id, epsilon_cost),
        );
        Ok(query)
    }

    /// Get remaining budget
    pub fn get_remaining_budget(env: Env, budget_id: BytesN<32>) -> Result<u64, Error> {
        Self::require_initialized(&env)?;
        let budget = Self::load_budget(&env, &budget_id)?;
        Ok(budget.epsilon_remaining)
    }

    /// Get query by ID
    pub fn get_query(env: Env, query_id: BytesN<32>) -> Option<PrivacyQuery> {
        env.storage().persistent().get(&DataKey::Query(query_id))
    }

    /// Audit a stored query against the published noise bound (#1619).
    ///
    /// Returns `None` when the query is unknown. Otherwise the returned audit
    /// reports `within_bound`, so a consumer can tell an unauditable record
    /// from a clean one without re-deriving anything off-chain. An
    /// `Err(ArithmeticOverflow)` means the record's `noisy_result -
    /// true_result` is not representable and the record cannot be audited.
    pub fn get_query_noise_audit(
        env: Env,
        query_id: BytesN<32>,
    ) -> Result<Option<QueryNoiseAudit>, Error> {
        Self::require_initialized(&env)?;
        let Some(query) = Self::load_query(&env, &query_id) else {
            return Ok(None);
        };
        privacy_invariant::audit_query(&query).map(Some)
    }

    /// Audit a budget's epsilon accounting against its ledger (#1619).
    ///
    /// `reconciled: false` means the budget cannot be shown to be consistent
    /// with the queries charged against it. For a budget created before ledger
    /// accounting existed this is the expected answer (`ledger_present:
    /// false`) — there is no on-chain evidence either way, and the audit says
    /// so instead of guessing.
    pub fn get_budget_noise_audit(
        env: Env,
        budget_id: BytesN<32>,
    ) -> Result<Option<BudgetNoiseAudit>, Error> {
        Self::require_initialized(&env)?;
        let Some(budget) = Self::load_budget_opt(&env, &budget_id) else {
            return Ok(None);
        };
        let ledger = privacy_invariant::load_ledger(&env, &budget_id);
        Ok(Some(privacy_invariant::audit_budget(
            &budget,
            ledger.as_ref(),
        )))
    }

    /// Deactivate a privacy budget
    pub fn deactivate_budget(env: Env, admin: Address, budget_id: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        Self::require_initialized(&env)?;

        let mut budget = Self::load_budget(&env, &budget_id)?;
        budget.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Budget(budget_id.clone()), &budget);

        env.events()
            .publish((symbol_short!("dp"), symbol_short!("deactiv")), budget_id);
        Ok(())
    }

    // Internal helper functions

    #[must_use]
    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    #[must_use]
    fn load_budget(env: &Env, budget_id: &BytesN<32>) -> Result<PrivacyBudget, Error> {
        Self::load_budget_opt(env, budget_id).ok_or(Error::BudgetNotFound)
    }

    fn load_budget_opt(env: &Env, budget_id: &BytesN<32>) -> Option<PrivacyBudget> {
        env.storage()
            .persistent()
            .get(&DataKey::Budget(budget_id.clone()))
    }

    fn load_query(env: &Env, query_id: &BytesN<32>) -> Option<PrivacyQuery> {
        env.storage()
            .persistent()
            .get(&DataKey::Query(query_id.clone()))
    }

    /// Fail unless the perturbation just produced satisfies the published
    /// bound for `mechanism` at `sensitivity` (#1619).
    fn require_within_noise_bound(
        mechanism: NoiseMechanism,
        sensitivity: u64,
        true_value: i64,
        noisy_result: i64,
    ) -> Result<(), Error> {
        let max_abs_noise = privacy_invariant::noise_bound(mechanism, sensitivity)?;
        let observed = privacy_invariant::observed_abs_noise(true_value, noisy_result)?;
        if observed > max_abs_noise {
            return Err(Error::NoiseBoundExceeded);
        }
        Ok(())
    }

    fn generate_budget_id(env: &Env) -> BytesN<32> {
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::BudgetCounter)
            .unwrap_or(0);
        let next = counter.checked_add(1).unwrap_or(0);
        env.storage().instance().set(&DataKey::BudgetCounter, &next);
        BytesN::from_array(env, &[next as u8; 32])
    }
}
