#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

pub mod errors;
pub use errors::Error;

const WEEK_SECONDS: u64 = 604_800;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Limit(Address),
    Spent((Address, u64)),
}

#[contract]
pub struct SpendingLimitsContract;

#[contractimpl]
impl SpendingLimitsContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn set_limit(
        env: Env,
        caller: Address,
        user: Address,
        limit: i128,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        if limit < 0 {
            return Err(Error::InvalidInput);
        }
        env.storage().persistent().set(&DataKey::Limit(user), &limit);
        Ok(())
    }

    pub fn get_limit(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Limit(user))
            .unwrap_or(0)
    }

    pub fn check_limit(env: Env, user: Address, amount: i128) -> bool {
        let limit = Self::get_limit(env.clone(), user.clone());
        if limit == 0 {
            return true;
        }
        let spent = Self::get_spent(env, user);
        spent + amount <= limit
    }

    pub fn get_spent(env: Env, user: Address) -> i128 {
        let week = Self::current_week(&env);
        env.storage()
            .persistent()
            .get(&DataKey::Spent((user, week)))
            .unwrap_or(0)
    }

    pub fn record_spent(env: Env, user: Address, amount: i128) {
        let week = Self::current_week(&env);
        let key = DataKey::Spent((user, week));
        let current: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(current + amount));
    }

    fn current_week(env: &Env) -> u64 {
        env.ledger().timestamp() / WEEK_SECONDS
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
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, SpendingLimitsContract);
        let client = SpendingLimitsContractClient::new(&env, &contract_id);
        client.initialize(&admin);
        assert_eq!(client.get_limit(&admin), 0);
    }

    #[test]
    fn test_set_and_check_limit() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let contract_id = env.register_contract(None, SpendingLimitsContract);
        let client = SpendingLimitsContractClient::new(&env, &contract_id);
        client.initialize(&admin);
        client.set_limit(&admin, &user, &200);

        assert!(client.check_limit(&user, &50));
        assert!(client.check_limit(&user, &200));
        assert!(!client.check_limit(&user, &201));
    }

    #[test]
    fn test_record_spent() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let contract_id = env.register_contract(None, SpendingLimitsContract);
        let client = SpendingLimitsContractClient::new(&env, &contract_id);
        client.initialize(&admin);
        client.set_limit(&admin, &user, &200);
        client.record_spent(&user, &80);

        assert_eq!(client.get_spent(&user), 80);
        assert!(!client.check_limit(&user, &150));
        assert!(client.check_limit(&user, &120));
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::InvalidInput as u32, 200);
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::LimitExceeded as u32, 500);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        assert_eq!(get_suggestion(Error::Unauthorized), symbol_short!("CHK_AUTH"));
        assert_eq!(get_suggestion(Error::NotInitialized), symbol_short!("INIT_CTR"));
        assert_eq!(get_suggestion(Error::AlreadyInitialized), symbol_short!("ALREADY"));
    }
}
