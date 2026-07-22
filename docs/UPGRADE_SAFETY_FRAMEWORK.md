# Contract Upgrade Safety Framework

This document describes the upgrade safety framework for Uzima contracts, which formalizes upgrade policy, validates storage compatibility, runs migration rehearsal checks, and provides deterministic rollback behavior.

## Overview

The upgrade safety framework ensures that contract upgrades:
1. Declare migration requirements upfront via an **upgrade manifest**
2. Validate storage layout compatibility before execution
3. Run pre-flight and post-flight integrity checks
4. Support deterministic rollback to the previous version
5. Emit structured lifecycle events for auditability

## Upgrade Manifest

An `UpgradeManifest` declares everything about an upgrade:

```rust
pub struct UpgradeManifest {
    pub target_version: u32,
    pub new_wasm_hash: BytesN<32>,
    pub description: String,
    pub invariants: Vec<MigrationInvariant>,
    pub storage_rules: StorageCompatibilityRules,
    pub deprecated_functions: Vec<DeprecatedFunctionEntry>,
    pub rollback_supported: bool,
    pub migration_timeout_ledgers: u32,
}
```

### Migration Invariants

Each invariant declares a check that must pass after migration:

```rust
pub struct MigrationInvariant {
    pub name: Symbol,           // e.g., "all_records_have_consent"
    pub description: String,
    pub severity: InvariantSeverity,  // Critical, Warning, Info
}
```

- **Critical**: Violation blocks the upgrade
- **Warning**: Violation logs but allows the upgrade
- **Info**: Violation is informational only

### Storage Compatibility Rules

```rust
pub struct StorageCompatibilityRules {
    pub preserve_existing_keys: bool,   // Must existing keys be preserved?
    pub allow_new_keys: bool,           // Are new keys allowed?
    pub allow_type_changes: bool,       // Can key types change?
    pub max_new_entries: u32,           // Maximum new entries
    pub required_keys: Vec<Symbol>,     // Keys that must exist
    pub removed_keys: Vec<Symbol>,      // Keys that will be removed
}
```

## Upgrade Policy

The `UpgradePolicy` controls upgrade behavior:

```rust
pub struct UpgradePolicy {
    pub require_dry_run: bool,           // Must dry-run pass first?
    pub require_rollback_test: bool,     // Must rollback be possible?
    pub max_rollback_attempts: u32,      // Max rollback attempts
    pub emit_detailed_events: bool,      // Emit verbose events?
    pub pause_during_migration: bool,    // Pause contract during migration?
}
```

Default policy: `require_dry_run=true`, `require_rollback_test=true`, `max_rollback_attempts=3`.

## Upgrade Lifecycle

### 1. Submit Manifest (Dry-Run)

```rust
let manifest = create_default_manifest(env, 2, new_wasm_hash, "Upgrade v2");
let result = submit_manifest(env, manifest)?;
// result.passed == true means upgrade is safe
```

The dry-run validates:
- Storage compatibility rules
- Migration invariants (simulated)
- Rollback support
- Version compatibility

### 2. Execute Upgrade

```rust
execute_manifest_upgrade::<MyContract>(env)?;
```

This performs:
- Pre-flight validation
- Migration execution
- Post-migration integrity check
- History recording
- Structured event emission

### 3. Rollback (if needed)

```rust
safe_rollback(env)?;
```

Rollback respects the policy:
- Checks rollback count against `max_rollback_attempts`
- Emits `RolledBack` lifecycle event
- Increments rollback counter

## Lifecycle Events

Every upgrade phase emits a structured `UpgradeLifecycleEvent`:

| Phase | Symbol | When |
|---|---|---|
| `PreFlightCheck` | `PREFLIGHT` | Pre-flight validation started |
| `StorageValidated` | `STOR_VALID` | Storage compatibility checked |
| `InvariantsChecked` | `INV_CHECK` | Migration invariants checked |
| `Migrating` | `MIGRATING` | Migration executing |
| `PostMigrationCheck` | `POST_MIG` | Post-migration integrity check |
| `Completed` | `COMPLETED` | Upgrade finished |
| `RolledBack` | `ROLLED_BK` | Upgrade rolled back |
| `Failed` | `FAILED` | Upgrade failed permanently |

## Dry-Run Validation

The `DryRunResult` provides detailed feedback:

```rust
pub struct DryRunResult {
    pub passed: bool,
    pub storage_compatible: bool,
    pub invariants_satisfied: bool,
    pub estimated_gas_impact: i128,
    pub issues: Vec<DryRunIssue>,
}
```

Issues are categorized:
- `StorageIncompatible` — storage layout conflict
- `InvariantViolation` — migration invariant would fail
- `DeprecatedNotHandled` — deprecated function not handled
- `GasBudgetExceeded` — gas estimate too high
- `RollbackImpossible` — rollback not supported

## Implementing Migratable for Your Contract

```rust
impl Migratable for MyContract {
    fn migrate(env: &Env, from_version: u32) -> Result<(), UpgradeError> {
        match from_version {
            1 => {
                // Migration from v1 to v2
                // Migrate storage layout, add new fields, etc.
            }
            _ => Err(UpgradeError::IncompatibleVersion),
        }
    }

    fn verify_integrity(env: &Env) -> Result<BytesN<32>, UpgradeError> {
        // Verify all invariants hold
        // Return a hash of the current state
        Ok(BytesN::from_array(env, &[0u8; 32]))
    }
}
```

## CI Integration

The upgrade safety framework is designed to work with CI:
1. Dry-run validation can be tested in CI
2. Lifecycle events provide audit trails
3. Rollback behavior is explicitly tested

## Migration Guide

When upgrading a contract using the safety framework:

1. Create an `UpgradeManifest` with your invariants and storage rules
2. Run `submit_manifest()` to validate
3. Fix any issues reported in `DryRunResult`
4. Run `execute_manifest_upgrade()` when ready
5. Monitor lifecycle events for audit trail
6. If issues arise, use `safe_rollback()` to revert
