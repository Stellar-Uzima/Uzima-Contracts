# Splitting Initialization from State Migration

## Problem

Combining `initialize()` with migration logic couples two concerns and increases
upgrade deployment cost. Fresh deployments run unnecessary migration code; upgrades
re-run init code. Splitting them reduces gas cost for both operations.

## Pattern

```rust
/// Called ONCE on fresh deployment. Sets config only — no migration logic.
pub fn initialize(env: Env, admin: Address) {
    admin.require_auth();
    InitGuard::assert_not_initialized(&env);
    InstanceStore::set(&env, &DataKey::Admin, &admin);
    InstanceStore::set(&env, &DataKey::SchemaVersion, &1u32);
    ContractLifecycle::transition(&env, ContractLifecycleState::Active);
    InitGuard::mark_initialized(&env);
}

/// Called after a WASM upgrade. Idempotent — checks schema version before each step.
pub fn migrate(env: Env, admin: Address) -> Result<(), Error> {
    admin.require_auth();
    ContractLifecycle::require_active(&env)?;
    let schema: u32 = InstanceStore::get(&env, &DataKey::SchemaVersion).unwrap_or(1);
    if schema < 2 { migrate_v1_to_v2(&env); InstanceStore::set(&env, &DataKey::SchemaVersion, &2u32); }
    if schema < 3 { migrate_v2_to_v3(&env); InstanceStore::set(&env, &DataKey::SchemaVersion, &3u32); }
    Ok(())
}
```

## Cost Reduction

| Scenario | Before | After |
|----------|--------|-------|
| Fresh deployment | Migration code runs unnecessarily | Init only — lower CPU |
| WASM upgrade | Re-runs init guards | Migration only — lower CPU |
| Re-deploy | Cannot distinguish | InitGuard prevents double-init |

## Applies To

All contracts with combined init+migration: `upgradeability`, `medical_records`,
`patient_consent_management`, `identity_registry`, `rbac`.
