# Deprecation And Migration Guide

## Overview

The upgradeability layer now supports tracking deprecated contract functions and emitting warnings when those functions are still used after an upgrade. This gives maintainers a safer path for phasing out legacy entrypoints without breaking clients immediately.

## Maintainer Workflow

1. Mark the legacy entrypoint with Rust's deprecation attribute.
2. Register the deprecated function metadata during the upgrade.
3. Emit a deprecation warning event whenever the legacy function is called.
4. Keep the replacement function documented until the removal version is reached.

## Example

```rust
#[deprecated(since = "v2.0.0", note = "Use new_function instead")]
pub fn old_function(env: Env) {
    upgradeability::emit_deprecation_warning(&env, Symbol::new(&env, "old_function"))
        .ok();

    // Legacy behavior remains available during the migration window.
}
```

## Registering Deprecations During Upgrade

Use `upgradeability::execute_upgrade_with_deprecations` when a new release introduces deprecated APIs:

```rust
let deprecations = soroban_sdk::Vec::from_array(
    &env,
    [upgradeability::DeprecatedFunction {
        function: Symbol::new(&env, "old_function"),
        since: soroban_sdk::String::from_str(&env, "v2.0.0"),
        replacement: Some(Symbol::new(&env, "new_function")),
        removed_in: Some(soroban_sdk::String::from_str(&env, "v3.0.0")),
        note: soroban_sdk::String::from_str(&env, "This function will be removed in v3.0.0"),
        migration_guide: Some(soroban_sdk::String::from_str(
            &env,
            "docs/deprecation_migration.md",
        )),
    }],
);
```

## What Gets Stored

Each deprecated function record tracks:

- function name
- deprecation version
- replacement function
- planned removal version
- deprecation note
- migration guide reference

## Event Behavior

Calling `upgradeability::emit_deprecation_warning` emits a `Deprecated` event with:

- topic 1: `Deprecated`
- topic 2: deprecated function name
- data: deprecation note

## Recommended Removal Process

1. Deprecate the old entrypoint in the first upgrade release.
2. Keep the old and new functions available during the migration window.
3. Monitor deprecation events to identify remaining callers.
4. Remove the old entrypoint in the announced removal version.

## AML Example

The AML contract follows this pattern for administrative blacklist updates:

- Deprecated entrypoint: `set_user_status`
- Replacement entrypoint: `update_user_status`
- Deprecation since: `v2.0.0`
- Planned removal: `v3.0.0`

Existing AML deployments that predate the upgradeability registry should call
`register_deprecated_functions` before the next contract upgrade so the warning
events and stored metadata are available immediately.
