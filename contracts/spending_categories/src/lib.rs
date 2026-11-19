#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

pub mod errors;
pub use errors::Error;

const WEEK_SECONDS: u64 = 604_800;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    CategoryTag(Symbol),
    CategorySpent((Address, Symbol, u64)),
}

#[contract]
pub struct SpendingCategoriesContract;

#[contractimpl]
impl SpendingCategoriesContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn set_category(
        env: Env,
        caller: Address,
        tx_id: Symbol,
        category: Symbol,
    ) -> Result<(), Error> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&DataKey::CategoryTag(tx_id), &category);
        Ok(())
    }

    pub fn get_category(env: Env, tx_id: Symbol) -> Result<Symbol, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::CategoryTag(tx_id))
            .ok_or(Error::CategoryNotFound)
    }

    pub fn get_category_spent(env: Env, user: Address, category: Symbol) -> i128 {
        let week = Self::current_week(&env);
        env.storage()
            .persistent()
            .get(&DataKey::CategorySpent((user, category, week)))
            .unwrap_or(0)
    }

    pub fn record_category_spent(env: Env, user: Address, category: Symbol, amount: i128) {
        let week = Self::current_week(&env);
        let key = DataKey::CategorySpent((user, category, week));
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
        let contract_id = env.register_contract(None, SpendingCategoriesContract);
        let client = SpendingCategoriesContractClient::new(&env, &contract_id);
        client.initialize(&admin);
        let tx_id = Symbol::new(&env, "tx_001");
        assert_eq!(client.try_get_category(&tx_id).unwrap_err(), soroban_sdk::Error::Contract(Error::CategoryNotFound as u32));
    }

    #[test]
    fn test_set_and_get_category() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, SpendingCategoriesContract);
        let client = SpendingCategoriesContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        let tx_id = Symbol::new(&env, "tx_001");
        let category = Symbol::new(&env, "Groceries");
        client.set_category(&admin, &tx_id, &category);
        assert_eq!(client.get_category(&tx_id), category);
    }

    #[test]
    fn test_category_spent_tracking() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let contract_id = env.register_contract(None, SpendingCategoriesContract);
        let client = SpendingCategoriesContractClient::new(&env, &contract_id);
        client.initialize(&admin);

        let category = Symbol::new(&env, "Groceries");
        assert_eq!(client.get_category_spent(&user, &category), 0);

        client.record_category_spent(&user, &category, &80);
        assert_eq!(client.get_category_spent(&user, &category), 80);

        client.record_category_spent(&user, &category, &50);
        assert_eq!(client.get_category_spent(&user, &category), 130);
    }

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(Error::Unauthorized as u32, 100);
        assert_eq!(Error::InvalidInput as u32, 200);
        assert_eq!(Error::NotInitialized as u32, 300);
        assert_eq!(Error::AlreadyInitialized as u32, 301);
        assert_eq!(Error::CategoryNotFound as u32, 450);
    }

    #[test]
    fn test_get_suggestion_returns_expected_hint() {
        assert_eq!(get_suggestion(Error::Unauthorized), symbol_short!("CHK_AUTH"));
        assert_eq!(get_suggestion(Error::NotInitialized), symbol_short!("INIT_CTR"));
        assert_eq!(get_suggestion(Error::AlreadyInitialized), symbol_short!("ALREADY"));
        assert_eq!(get_suggestion(Error::CategoryNotFound), symbol_short!("CHK_CAT"));
    }
}
