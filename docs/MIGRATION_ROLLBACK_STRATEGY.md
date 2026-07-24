# Migration and Rollback Strategy for Upgradeable Contracts

This document defines the **formal migration and rollback strategy** for all
upgradeable contracts in the Uzima workspace. It is the authoritative reference
for developers, reviewers, and operators performing contract upgrades.

---

## Principles

1. **All upgrades are reversible** until the new contract version has been
   verified and the rollback window expires.
2. **State migrations are append-only** where possible; destructive schema
   changes require a migration proof test.
3. **Every upgrade has a documented rollback plan** before it is executed.
4. **No upgrade proceeds without a dry run** on a lower environment.

---

## Upgrade Lifecycle

```
Developer → PR Review → Testnet Dry Run → Testnet Upgrade → Verification
                                                                   │
                                                          rollback window (72h)
                                                                   │
                                                          Mainnet Dry Run
                                                                   │
                                                          Mainnet Upgrade
                                                                   │
                                                          Verification + Monitor
```

---

## Step-by-Step Upgrade Process

### 1. Pre-upgrade Checklist

Before initiating an upgrade, verify:

- [ ] Storage schema changes are documented in `docs/migrations/`
- [ ] Migration function is implemented and tested
- [ ] Rollback WASM (previous version) is archived in `deployments/<network>/rollback/`
- [ ] Dry run completed on testnet with `--dry-run` flag
- [ ] All integration tests pass against the new WASM
- [ ] Emergency access override is configured and tested
- [ ] Rollback plan written and reviewed

### 2. Archive the Current Version

```bash
# Before upgrading, record the current contract state
NETWORK=testnet
CONTRACT=medical_records
./scripts/migrate_contract.sh snapshot $CONTRACT $NETWORK

# Verify snapshot was created
ls deployments/$NETWORK/rollback/
```

### 3. Begin the Upgrade

```bash
# Transition contract to Upgrading lifecycle state
./scripts/upgrade_contract.sh begin $CONTRACT $NETWORK $NEW_WASM_HASH

# This:
# 1. Verifies caller is admin
# 2. Transitions lifecycle state: Active → Upgrading
# 3. Blocks new write transactions during migration
```

### 4. Run Migration

```bash
# Execute the state migration
./scripts/upgrade_contract.sh migrate $CONTRACT $NETWORK

# Migration function in Rust:
# pub fn migrate(env: Env, admin: Address) {
#     admin.require_auth();
#     ContractLifecycle::require_state(&env, ContractLifecycleState::Upgrading)?;
#     // ... migrate storage schema ...
#     ContractLifecycle::transition(&env, ContractLifecycleState::Active);
# }
```

### 5. Verify and Complete

```bash
# Run post-upgrade verification
./scripts/verify_deployment.sh $CONTRACT $NETWORK

# If verification passes, the upgrade is complete.
# The rollback window starts now (72h default).
```

---

## Rollback Process

### When to Rollback

Roll back immediately if any of the following are detected within the rollback
window (72h):

- Critical errors in contract responses
- Unexpected state corruption
- Security vulnerability discovered in new version
- Performance regression > 20% above budget

### How to Rollback

```bash
# List available rollback snapshots
./scripts/rollback_deployment.sh list $CONTRACT $NETWORK

# Execute rollback to previous version
./scripts/rollback_deployment.sh execute $CONTRACT $NETWORK <snapshot_id>

# This:
# 1. Pauses the contract (Active → Paused)
# 2. Deploys the previous WASM hash
# 3. Runs reverse migration if available
# 4. Resumes the contract (Paused → Active)
# 5. Verifies the rollback succeeded
```

### Rollback Constraints

| Scenario | Rollback possible? | Notes |
|----------|-------------------|-------|
| Storage schema extended (new keys added) | ✅ Yes | Old keys ignored |
| Storage schema changed (key renamed) | ⚠️ With migration | Forward migration must be reversible |
| Storage schema shrunk (keys deleted) | ❌ No | Data loss — cannot rollback |
| Rust logic only (no schema changes) | ✅ Yes | Drop-in WASM replacement |

---

## Storage Schema Change Policy

### Allowed (backward-compatible)

```rust
// BEFORE
#[contracttype]
pub enum DataKey { Admin, PatientRecord(Address) }

// AFTER — adding new keys is safe
#[contracttype]
pub enum DataKey { Admin, PatientRecord(Address), RecordVersion(u64) }
```

### Requires Migration

```rust
// BEFORE
#[contracttype]
pub struct Record { pub data: String, pub ledger: u64 }

// AFTER — adding fields requires migration
#[contracttype]
pub struct RecordV2 { pub data: String, pub ledger: u64, pub version: u32 }

// Migration function required:
pub fn migrate_records(env: &Env) {
    // Read all old records, add version field, write new records
}
```

### Forbidden (breaking)

- Renaming or deleting existing `DataKey` variants
- Changing field types in existing `#[contracttype]` structs
- Changing the `repr(u32)` discriminants of `#[contracterror]` enums

---

## Migration File Convention

All migrations live in `docs/migrations/` with this naming:

```
docs/migrations/<contract>/<version>-<short-description>.md
```

Example:

```
docs/migrations/medical_records/v2-add-record-version-field.md
```

Each migration doc must include:

1. **Pre-conditions**: contract must be in `Upgrading` state
2. **Storage changes**: what keys/types are added, changed, or removed
3. **Migration script**: the Rust function to run
4. **Rollback script**: the reverse Rust function (or "irreversible")
5. **Verification steps**: how to confirm the migration succeeded

---

## CI Enforcement

The following CI checks enforce this strategy:

```yaml
# .github/workflows/upgrade-safety.yml
- name: Validate migration docs
  run: scripts/validate_migrations.sh

- name: Check storage schema compatibility
  run: scripts/check_storage_compat.sh --baseline deployments/testnet/

- name: Run migration tests
  run: cargo test --package upgradeability migration
```

---

## References

- `contracts/upgradeability/src/lifecycle.rs` — lifecycle state machine
- `docs/CONTRACT_LIFECYCLE.md` — lifecycle state transition guide
- `scripts/migrate_contract.sh` — migration runner
- `scripts/rollback_deployment.sh` — rollback executor
- `scripts/upgrade_contract.sh` — upgrade orchestrator
