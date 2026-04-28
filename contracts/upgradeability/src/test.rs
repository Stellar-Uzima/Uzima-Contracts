#![allow(clippy::unwrap_used)]

use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String, Symbol, Vec};

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
        .filter(|event| {
            event.topics.len() >= 2 && event.topics[0] == Symbol::new(&env, "Deprecated")
        })
        .count();
    assert_eq!(deprecated_events, 1);
}
