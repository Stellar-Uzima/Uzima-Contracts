// Differential Privacy Contract - Simplified Working Version
#![no_std]
#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
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
}

const ADMIN: Symbol = symbol_short!("ADMIN");

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
}

// =============================================================================
// Contract
// =============================================================================

#[contract]
pub struct DifferentialPrivacyContract;

#[contractimpl]
impl DifferentialPrivacyContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(Error::AlreadyInitialized);
        }
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

        // Calculate epsilon cost (simplified: cost = sensitivity)
        let epsilon_cost = sensitivity;

        if epsilon_cost > budget.epsilon_remaining {
            return Err(Error::InsufficientBudget);
        }

        // Generate Laplace noise (simplified deterministic version)
        let noise = Self::generate_laplace_noise(&env, &query_id, sensitivity as i64);
        let noisy_result = true_value
            .checked_add(noise)
            .ok_or(Error::ArithmeticOverflow)?;

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
        let epsilon_cost = sensitivity
            .checked_mul(2)
            .ok_or(Error::ArithmeticOverflow)?;

        if epsilon_cost > budget.epsilon_remaining {
            return Err(Error::InsufficientBudget);
        }

        // Generate Gaussian noise (simplified)
        let noise = Self::generate_gaussian_noise(&env, &query_id, sensitivity as i64);
        let noisy_result = true_value
            .checked_add(noise)
            .ok_or(Error::ArithmeticOverflow)?;

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

    fn require_initialized(env: &Env) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Initialized) {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }

    fn load_budget(env: &Env, budget_id: &BytesN<32>) -> Result<PrivacyBudget, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Budget(budget_id.clone()))
            .ok_or(Error::BudgetNotFound)
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

    /// Generate Laplace noise using pseudo-random generation
    fn generate_laplace_noise(env: &Env, seed: &BytesN<32>, scale: i64) -> i64 {
        let hash = env.crypto().sha256(&seed.clone().into());
        let hash_bytes: [u8; 32] = hash.into();
        let hash_int = u64::from_le_bytes(hash_bytes[0..8].try_into().unwrap_or([0; 8]));

        // Convert to signed noise centered around 0
        let noise = (hash_int % (scale.unsigned_abs() * 2)) as i64;
        noise - scale.unsigned_abs() as i64
    }

    /// Generate Gaussian noise using pseudo-random generation
    fn generate_gaussian_noise(env: &Env, seed: &BytesN<32>, scale: i64) -> i64 {
        let hash = env.crypto().sha256(&seed.clone().into());
        let hash_bytes: [u8; 32] = hash.into();
        let hash_int = u64::from_le_bytes(hash_bytes[0..8].try_into().unwrap_or([0; 8]));

        // Approximate normal distribution (3σ range)
        let noise = (hash_int % (scale.unsigned_abs() * 6)) as i64;
        noise - (scale.unsigned_abs() * 3) as i64
    }
}
