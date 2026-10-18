#![no_std]

mod errors;

use errors::Error;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

/// Minimal rental-yield distribution contract: tracks a per-property yield
/// pool and lets the admin deposit collected rent and distribute it to a
/// recipient. Intentionally small — a foundation for later Stellar Wave
/// work (access-control audit, OpenZeppelin-style refactor, validation,
/// events) to build on rather than a complete implementation. See
/// issues #1806, #1807, #1809, #1811.
#[contract]
pub struct RentalYieldContract;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Pool,
}

// require_admin! hardcodes Error::NotAuthorized, but our Error (like
// rbac::errors::Error) mirrors common_error's Unauthorized spelling instead,
// so use the custom variant to map onto our own Error type.
common_auth::require_admin_custom!(
    instance,
    DataKey::Admin,
    Error::NotInitialized,
    Error::Unauthorized
);

#[contractimpl]
impl RentalYieldContract {
    /// One-time setup: records the admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Pool, &0i128);
        Ok(())
    }

    /// Adds `amount` of collected rent to the yield pool (issue #1807: input
    /// validation rejects non-positive amounts).
    pub fn deposit(env: Env, caller: Address, amount: i128) -> Result<i128, Error> {
        caller.require_auth();
        require_admin(&env, &caller)?;
        if amount <= 0 {
            return Err(Error::InvalidInput);
        }

        let pool: i128 = env.storage().instance().get(&DataKey::Pool).unwrap_or(0);
        let pool = pool
            .checked_add(amount)
            .ok_or(Error::InvalidState)?;
        env.storage().instance().set(&DataKey::Pool, &pool);

        // issue #1806: emit an event for the state change.
        env.events()
            .publish((symbol_short!("deposit"), caller), (amount, pool));

        Ok(pool)
    }

    /// Pays `amount` out of the yield pool to `recipient`. Admin-gated
    /// (issue #1811: access control) using the shared `common_auth`
    /// `require_admin_custom!` primitive rather than a hand-rolled check
    /// (issue #1809: reuse shared primitives instead of duplicating auth
    /// logic).
    pub fn distribute(
        env: Env,
        caller: Address,
        recipient: Address,
        amount: i128,
    ) -> Result<i128, Error> {
        caller.require_auth();
        require_admin(&env, &caller)?;
        if amount <= 0 {
            return Err(Error::InvalidInput);
        }

        let pool: i128 = env.storage().instance().get(&DataKey::Pool).unwrap_or(0);
        if amount > pool {
            return Err(Error::InsufficientFunds);
        }
        let pool = pool - amount;
        env.storage().instance().set(&DataKey::Pool, &pool);

        // issue #1806: emit an event for the state change.
        env.events()
            .publish((symbol_short!("yield"), recipient), (amount, pool));

        Ok(pool)
    }

    /// Current yield pool balance.
    pub fn pool_balance(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Pool).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup(env: &Env) -> (Address, Address) {
        let contract_id = env.register(RentalYieldContract, ());
        let admin = Address::generate(env);
        env.as_contract(&contract_id, || {
            RentalYieldContract::initialize(env.clone(), admin.clone()).unwrap();
        });
        (contract_id, admin)
    }

    #[test]
    fn deposit_and_distribute_update_the_pool() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, admin) = setup(&env);
        let recipient = Address::generate(&env);

        env.as_contract(&contract_id, || {
            assert_eq!(
                RentalYieldContract::deposit(env.clone(), admin.clone(), 100).unwrap(),
                100
            );
            assert_eq!(
                RentalYieldContract::distribute(env.clone(), admin.clone(), recipient, 40)
                    .unwrap(),
                60
            );
            assert_eq!(RentalYieldContract::pool_balance(env.clone()), 60);
        });
    }

    #[test]
    fn rejects_non_positive_deposit() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, admin) = setup(&env);

        env.as_contract(&contract_id, || {
            assert_eq!(
                RentalYieldContract::deposit(env.clone(), admin, 0),
                Err(Error::InvalidInput)
            );
        });
    }

    #[test]
    fn rejects_distribution_beyond_pool_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, admin) = setup(&env);
        let recipient = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RentalYieldContract::deposit(env.clone(), admin.clone(), 10).unwrap();
            assert_eq!(
                RentalYieldContract::distribute(env.clone(), admin, recipient, 11),
                Err(Error::InsufficientFunds)
            );
        });
    }

    #[test]
    fn rejects_non_admin_caller() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, _admin) = setup(&env);
        let stranger = Address::generate(&env);

        env.as_contract(&contract_id, || {
            assert_eq!(
                RentalYieldContract::deposit(env.clone(), stranger, 10),
                Err(Error::Unauthorized)
            );
        });
    }
}
