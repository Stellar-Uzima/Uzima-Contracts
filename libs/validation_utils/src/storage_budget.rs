#![no_std]
//! storage_budget - Protection against storage exhaustion and rent abuse.
//!
//! Enforces per-contract and per-user storage entry caps to prevent DoS
//! through unbounded state growth and Soroban rent accumulation.

use soroban_sdk::{contracttype, symbol_short, Address, Env};

/// Default maximum persistent entries per user (e.g. medical records per patient).
pub const DEFAULT_MAX_ENTRIES_PER_USER: u32 = 1_000;

/// Default maximum total entries in a contract.
pub const DEFAULT_MAX_CONTRACT_ENTRIES: u32 = 500_000;

/// Storage budget keys.
#[derive(Clone)]
#[contracttype]
pub enum BudgetKey {
    /// Entry count for a specific user within this contract.
    UserCount(Address),
    /// Total entry count across the whole contract.
    ContractTotal,
}

/// Storage budget guard.
pub struct StorageBudget;

impl StorageBudget {
    /// Check that adding `delta` entries for `user` stays within limits.
    /// Returns `Err` if it would exceed per-user or contract-wide caps.
    pub fn check_and_increment(
        env: &Env,
        user: &Address,
        delta: u32,
        max_per_user: u32,
        max_total: u32,
    ) -> Result<(), StorageBudgetError> {
        // Per-user check
        let user_key = BudgetKey::UserCount(user.clone());
        let user_count: u32 = env.storage().instance().get(&user_key).unwrap_or(0);
        if user_count + delta > max_per_user {
            env.events().publish(
                (symbol_short!("budget"), symbol_short!("usr_limit")),
                (user, user_count),
            );
            return Err(StorageBudgetError::UserLimitExceeded);
        }

        // Contract-wide check
        let total_key = BudgetKey::ContractTotal;
        let total: u32 = env.storage().instance().get(&total_key).unwrap_or(0);
        if total + delta > max_total {
            env.events().publish(
                (symbol_short!("budget"), symbol_short!("ctr_limit")),
                total,
            );
            return Err(StorageBudgetError::ContractLimitExceeded);
        }

        // Commit
        env.storage().instance().set(&user_key, &(user_count + delta));
        env.storage().instance().set(&total_key, &(total + delta));
        Ok(())
    }

    /// Decrement budget when entries are deleted.
    pub fn decrement(env: &Env, user: &Address, delta: u32) {
        let user_key = BudgetKey::UserCount(user.clone());
        let total_key = BudgetKey::ContractTotal;
        let user_count: u32 = env.storage().instance().get(&user_key).unwrap_or(0);
        let total: u32 = env.storage().instance().get(&total_key).unwrap_or(0);
        env.storage().instance().set(&user_key, &user_count.saturating_sub(delta));
        env.storage().instance().set(&total_key, &total.saturating_sub(delta));
    }

    /// Returns the current entry count for a user.
    pub fn user_count(env: &Env, user: &Address) -> u32 {
        env.storage().instance()
            .get(&BudgetKey::UserCount(user.clone()))
            .unwrap_or(0)
    }

    /// Returns the total contract entry count.
    pub fn contract_total(env: &Env) -> u32 {
        env.storage().instance()
            .get(&BudgetKey::ContractTotal)
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[soroban_sdk::contracterror]
#[repr(u32)]
pub enum StorageBudgetError {
    /// The per-user storage entry cap has been exceeded.
    UserLimitExceeded = 560,
    /// The contract-wide storage entry cap has been exceeded.
    ContractLimitExceeded = 561,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_within_budget_passes() {
        let env = Env::default();
        let user = Address::generate(&env);
        assert!(StorageBudget::check_and_increment(&env, &user, 1, 10, 1000).is_ok());
        assert_eq!(StorageBudget::user_count(&env, &user), 1);
    }

    #[test]
    fn test_per_user_limit_enforced() {
        let env = Env::default();
        let user = Address::generate(&env);
        StorageBudget::check_and_increment(&env, &user, 5, 5, 1000).unwrap();
        assert_eq!(
            StorageBudget::check_and_increment(&env, &user, 1, 5, 1000).unwrap_err(),
            StorageBudgetError::UserLimitExceeded
        );
    }

    #[test]
    fn test_contract_limit_enforced() {
        let env = Env::default();
        let user = Address::generate(&env);
        assert_eq!(
            StorageBudget::check_and_increment(&env, &user, 11, 100, 10).unwrap_err(),
            StorageBudgetError::ContractLimitExceeded
        );
    }

    #[test]
    fn test_decrement_frees_budget() {
        let env = Env::default();
        let user = Address::generate(&env);
        StorageBudget::check_and_increment(&env, &user, 5, 5, 1000).unwrap();
        StorageBudget::decrement(&env, &user, 3);
        assert_eq!(StorageBudget::user_count(&env, &user), 2);
    }
}
