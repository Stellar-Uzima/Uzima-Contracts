#![allow(clippy::unwrap_used)]

use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{Address, Env, String, Symbol, TryFromVal, Vec};

use crate::{
    emit_deprecation_warning, get_deprecated_function, get_deprecated_functions,
    set_deprecated_functions, storage, DeprecatedFunction,
};

fn sample_deprecation(env: &Env) -> DeprecatedFunction {
    DeprecatedFunction {
        function: Symbol::new(env, "old_function"),
        since: String::from_str(env, "v2.0.0"),
        replacement: Some(Symbol::new(env, "new_function")),
        removed_in: Some(String::from_str(env, "v3.0.0")),
        note: String::from_str(env, "This function will be removed in v3.0.0"),
        migration_guide: Some(String::from_str(env, "docs/deprecation_migration.md")),
    }
}

#[test]
fn test_deprecated_functions_are_tracked() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    storage::set_admin(&env, &admin);

    let deprecation = sample_deprecation(&env);
    let deprecations = Vec::from_array(&env, [deprecation.clone()]);

    set_deprecated_functions(&env, deprecations).unwrap();

    let stored = get_deprecated_functions(&env);
    assert_eq!(stored.len(), 1);

    let tracked = get_deprecated_function(&env, Symbol::new(&env, "old_function")).unwrap();
    assert_eq!(tracked, deprecation);
}

#[test]
fn test_deprecation_warning_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    storage::set_admin(&env, &admin);

    let deprecations = Vec::from_array(&env, [sample_deprecation(&env)]);
    set_deprecated_functions(&env, deprecations).unwrap();

    let initial_event_count = env.events().all().len();
    emit_deprecation_warning(&env, Symbol::new(&env, "old_function")).unwrap();

    let events = env.events().all();
    assert!(events.len() > initial_event_count);

    let deprecated_events = events
        .iter()
        .filter(|(_, topics, _)| {
            if topics.len() < 2 {
                return false;
            }
            let Some(first) = topics.get(0) else {
                return false;
            };
            Symbol::try_from_val(&env, &first) == Ok(Symbol::new(&env, "Deprecated"))
        })
        .count();
    assert_eq!(deprecated_events, 1);
}

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env};

#[test]
fn test_pause_and_unpause_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, UpgradeableContract);
    let client = UpgradeableContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // Initial state: Not paused
    assert_eq!(client.is_contract_paused(), false);
    assert_eq!(client.execute_maintenance_action(), Ok(()));

    // Admin triggers pause
    assert_eq!(client.pause_contract(&admin), Ok(()));
    assert_eq!(client.is_contract_paused(), true);

    // Protected actions fail while contract is paused
    assert_eq!(
        client.execute_maintenance_action(),
        Err(PauseError::ContractIsPaused)
    );

    // Admin unpauses contract
    assert_eq!(client.resume_contract(&admin), Ok(()));
    assert_eq!(client.is_contract_paused(), false);

    // Operations resume normally
    assert_eq!(client.execute_maintenance_action(), Ok(()));
}

