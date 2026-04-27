# Contract Upgrade Safety

This guide documents safe practices for upgrading and migrating Uzima smart contracts on Soroban.

## Upgrade Patterns

### 1. In-place WASM Upgrade (Soroban Native)

Soroban supports replacing a contract's WASM bytecode while preserving its storage. Use `contract.upgrade(new_wasm_hash)` via the `upgrade_manager` contract.

```rust
// Only callable by admin
pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
}
```

**When to use**: Fixing bugs or adding functions without changing storage layout.

### 2. Versioned Storage Migration

When the storage schema changes, migrate data explicitly before or after the WASM upgrade.

```rust
pub fn migrate_v1_to_v2(env: Env) {
    // Read old format
    let old_value: OldType = env.storage().instance().get(&DataKey::SomeKey).unwrap();
    // Write new format
    let new_value = NewType::from(old_value);
    env.storage().instance().set(&DataKey::SomeKey, &new_value);
    env.storage().instance().set(&DataKey::SchemaVersion, &2u32);
}
```

### 3. Proxy / Router Pattern

Deploy a new contract and update a router to point to it. The old contract remains for historical reads.

## State Migration Strategies

| Strategy | Use Case | Risk |
|---|---|---|
| Lazy migration | Large datasets; migrate on first access | Inconsistent state during transition |
| Eager migration | Small datasets; migrate all at once | High gas cost in one transaction |
| Dual-write | Zero-downtime; write to both old and new | Complexity; must clean up old storage |
| Snapshot + replay | Full schema change | Requires off-chain tooling |

**Recommendation**: Use eager migration for contracts with < 1,000 storage entries. Use lazy migration with a `schema_version` guard for larger datasets.

## Testing Procedures

Before deploying any upgrade:

1. **Unit tests**: Cover all migration functions with before/after state assertions.
2. **Fork test**: Run migration against a snapshot of testnet state.
3. **Dry run**: Call migration in a simulated environment and verify storage keys.
4. **Rollback test**: Verify the rollback procedure restores the previous state.

```bash
# Run upgrade-specific tests
cargo test --test upgradeability_tests

# Simulate migration on testnet fork
./scripts/upgrade_contract.sh <contract_name> testnet --dry-run
```

## Rollback Plans

Every upgrade must have a documented rollback plan before deployment.

### Rollback Checklist

- [ ] Previous WASM hash is recorded in `deployments/`
- [ ] Storage backup taken before upgrade (via `./scripts/deploy_with_rollback.sh`)
- [ ] Rollback script tested on testnet
- [ ] On-call engineer available during upgrade window

### Executing a Rollback

```bash
# List available backups
./scripts/rollback_deployment.sh <contract_name> testnet

# Rollback to specific backup
./scripts/rollback_deployment.sh <contract_name> testnet deployments/testnet_<contract>_backup_<timestamp>.json
```

## User Communication

For upgrades that affect external callers:

1. **Announce 48 hours in advance** via GitHub Discussions and any integrated frontends.
2. **Emit a deprecation event** from the old entrypoint if it will be removed:
   ```rust
   events::emit_deprecation_warning(&env, "old_function", "Use new_function instead");
   ```
3. **Maintain backward compatibility** for at least one release cycle when possible.
4. **Update API docs** (`docs/api.md`) before the upgrade goes live.

## Monitoring During Upgrades

1. Watch CI/CD pipeline for the upgrade transaction confirmation.
2. Run health checks immediately after:
   ```bash
   ./scripts/monitor_deployments.sh testnet
   ```
3. Monitor error rates for 30 minutes post-upgrade.
4. Check the `health_check` contract's `check_alert_thresholds` for anomalies.

## Upgrade Checklist

- [ ] New WASM built with `cargo build --target wasm32-unknown-unknown --release`
- [ ] WASM hash verified: `sha256sum dist/<contract>.wasm`
- [ ] Migration function tested on testnet fork
- [ ] Rollback plan documented and tested
- [ ] Backup of current deployment created
- [ ] User communication sent (if breaking change)
- [ ] API docs updated
- [ ] Post-upgrade monitoring window scheduled (minimum 30 minutes)
- [ ] Deployment log entry created in `deployments/`

## Examples

See [`docs/upgradeability.md`](./upgradeability.md) for the deprecation tracking pattern and [`docs/migration.md`](./migration.md) for storage migration examples used in this project.
