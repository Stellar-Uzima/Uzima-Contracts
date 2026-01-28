use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Env, Symbol,
};

use crate::storage::ProxyStorage;

mod storage;

#[contract]
pub struct UpgradeableProxy;

const STORAGE_KEY: Symbol = Symbol::short("PROXY");

#[contractimpl]
impl UpgradeableProxy {
    // 🔹 Initialize proxy
    pub fn init(env: Env, implementation: Address, governance: Address) {
        if env.storage().instance().has(&STORAGE_KEY) {
            panic!("Proxy already initialized");
        }

        let storage = ProxyStorage {
            implementation,
            previous_implementation: None,
            governance,
            version: 1,
        };

        env.storage().instance().set(&STORAGE_KEY, &storage);
    }

    // 🔹 Upgrade implementation (governance only)
    pub fn upgrade(env: Env, new_impl: Address) {
        let mut storage: ProxyStorage = env
            .storage()
            .instance()
            .get(&STORAGE_KEY)
            .unwrap();

        storage.governance.require_auth();

        storage.previous_implementation = Some(storage.implementation.clone());
        storage.implementation = new_impl;
        storage.version += 1;

        env.storage().instance().set(&STORAGE_KEY, &storage);
    }

    // 🔹 Rollback
    pub fn rollback(env: Env) {
        let mut storage: ProxyStorage = env
            .storage()
            .instance()
            .get(&STORAGE_KEY)
            .unwrap();

        storage.governance.require_auth();

        let prev = storage
            .previous_implementation
            .clone()
            .expect("No previous implementation");

        storage.implementation = prev;
        storage.version -= 1;

        env.storage().instance().set(&STORAGE_KEY, &storage);
    }

    // 🔹 Delegate call
    pub fn call(env: Env, fn_name: Symbol, args: Vec<soroban_sdk::Val>) {
        let storage: ProxyStorage = env
            .storage()
            .instance()
            .get(&STORAGE_KEY)
            .unwrap();

        env.invoke_contract::<()>(
            &storage.implementation,
            &fn_name,
            args,
        );
    }
}
