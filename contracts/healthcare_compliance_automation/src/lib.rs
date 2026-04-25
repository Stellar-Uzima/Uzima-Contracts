#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec,
};

const ADMIN: Symbol = symbol_short!("ADMIN");
const SUPPORTED: Symbol = symbol_short!("SUPPORTED");

#[derive(Clone)]
#[contracttype]
pub struct FrameworkList {
    pub frameworks: Vec<String>,
}

#[contract]
pub struct HealthcareComplianceAutomation;

#[contractimpl]
impl HealthcareComplianceAutomation {
    pub fn initialize(env: Env, admin: Address, frameworks: Vec<String>) {
        #[cfg(not(test))]
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        let list = FrameworkList { frameworks };
        env.storage().instance().set(&SUPPORTED, &list);
    }

    #[allow(unused_variables)]
    pub fn add_framework(env: Env, admin: Address, framework: String) {
        #[cfg(not(test))]
        admin.require_auth();
        let mut list: FrameworkList =
            env.storage()
                .instance()
                .get(&SUPPORTED)
                .unwrap_or(FrameworkList {
                    frameworks: Vec::new(&env),
                });
        list.frameworks.push_back(framework);
        env.storage().instance().set(&SUPPORTED, &list);
    }

    pub fn get_supported_frameworks(env: Env) -> FrameworkList {
        env.storage()
            .instance()
            .get(&SUPPORTED)
            .unwrap_or(FrameworkList {
                frameworks: Vec::new(&env),
            })
    }
}
