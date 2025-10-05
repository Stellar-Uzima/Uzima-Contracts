use soroban_sdk::{Env, Address, Vec as SVec};

use crate::types::{AdapterConfig, DataKey, LastGood};

pub fn get_config(env: &Env) -> AdapterConfig {
    env.storage().instance().get(&DataKey::Config).expect("not inited")
}

pub fn set_config(env: &Env, cfg: &AdapterConfig) {
    env.storage().instance().set(&DataKey::Config, cfg);
}

pub fn get_last_good(env: &Env) -> Option<LastGood> {
    env.storage().instance().get(&DataKey::LastGood)
}

pub fn set_last_good(env: &Env, lg: &LastGood) {
    env.storage().instance().set(&DataKey::LastGood, lg);
}

pub fn is_owner(env: &Env, who: &Address) -> bool {
    let cfg = get_config(env);
    &cfg.owner == who
}

pub fn require_owner(env: &Env) {
    let invoker = env.invoker();
    let addr = invoker.require_address();
    addr.require_auth();
    assert!(is_owner(env, &addr), "not owner");
}

pub fn validate_providers(env: &Env, providers: &SVec<Address>, min_required: u32) {
    assert!(!providers.is_empty(), "no providers");
    assert!(min_required >= 1, "min_required>=1");
    assert!(min_required as u32 <= providers.len() as u32, "min_required <= providers.len");
    // Deduplicate check (gas friendly for small N)
    for i in 0..providers.len() {
        for j in (i+1)..providers.len() {
            assert!(providers.get_unchecked(i) != providers.get_unchecked(j), "dup providers");
        }
    }
}
