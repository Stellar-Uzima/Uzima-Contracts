#![no_std]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

pub fn setup_admin(env: &Env) -> Address {
    let admin = Address::generate(env);
    admin
}

pub fn setup_user(env: &Env) -> Address {
    Address::generate(env)
}

pub fn setup_env_with_admin() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = setup_admin(&env);
    (env, admin)
}

pub fn advance_time(env: &Env, seconds: u64) {
    use soroban_sdk::testutils::{Ledger, LedgerInfo};
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + seconds,
        ..Default::default()
    });
}