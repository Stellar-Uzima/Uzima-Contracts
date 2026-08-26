#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, Symbol};

pub mod engine;
pub mod errors;
pub mod types;

pub use errors::Error;
pub use types::Rule;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    SpendingLimitsAddr,
    SpendingCategoriesAddr,
    ZkVerifierAddr,
    Rule((Address, Symbol)),
}

#[contract]
pub struct SpendingRulesContract;

#[contractimpl]
impl SpendingRulesContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        spending_limits_addr: Address,
        spending_categories_addr: Address,
        zk_verifier_addr: Address,
    ) -> Result<(), Error> {
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::SpendingLimitsAddr, &spending_limits_addr);
        env.storage()
            .instance()
            .set(&DataKey::SpendingCategoriesAddr, &spending_categories_addr);
        env.storage()
            .instance()
            .set(&DataKey::ZkVerifierAddr, &zk_verifier_addr);
        Ok(())
    }

    pub fn set_rule(
        env: Env,
        caller: Address,
        user: Address,
        category: Symbol,
        weekly_limit: i128,
        zk_required_above: i128,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        if weekly_limit < 0 || zk_required_above < 0 {
            return Err(Error::InvalidInput);
        }
        let rule = Rule {
            category: category.clone(),
            weekly_limit,
            zk_required_above,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Rule((user, category)), &rule);
        Ok(())
    }

    pub fn get_rule(env: Env, user: Address, category: Symbol) -> Result<Rule, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Rule((user, category)))
            .ok_or(Error::RuleNotFound)
    }

    pub fn evaluate(
        env: Env,
        user: Address,
        category: Symbol,
        amount: i128,
        zk_proof: Option<Bytes>,
    ) -> Result<(), Error> {
        engine::evaluate_transaction(&env, &user, &category, amount, &zk_proof)
    }

    pub fn get_weekly_spent(env: Env, user: Address, category: Symbol) -> i128 {
        let spending_categories_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::SpendingCategoriesAddr)
            .unwrap();

        env.invoke_contract(
            &spending_categories_addr,
            &Symbol::new(env, "get_category_spent"),
            soroban_sdk::vec![
                env,
                user.into_val(env),
                category.into_val(env),
            ],
        )
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if caller != &admin {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;
