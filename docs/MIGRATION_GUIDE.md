# Migration Guide

This guide documents upgrade procedures for storage layout changes and schema
migrations across Uzima contracts.

---

## Table of Contents

1. [Overview](#overview)
2. [Storage Layout Change Procedure](#storage-layout-change-procedure)
3. [Schema Migration Procedure](#schema-migration-procedure)
4. [Upgrading Contracts with Storage Changes](#upgrading-contracts-with-storage-changes)
5. [Rollback Strategy](#rollback-strategy)
6. [Checklist](#checklist)

---

## Overview

Soroban contracts store state in persistent storage keyed by typed `DataKey`
enums. When a contract upgrade changes storage layout (new keys, changed key
encoding, or different value types), **existing on-chain state may become
inaccessible or corrupted** unless a migration is performed.

This guide covers:

- How to detect a storage-breaking change.
- How to write and test a migration.
- How to deploy the upgrade safely.

---

## Storage Layout Change Procedure

### Step 1 — Audit the Diff

Before upgrading, diff the contract's `DataKey` enum and any `contracttype`
definitions between the old and new versions:

```bash
# Compare storage key enums
diff <(git show upstream/main:contracts/medical_records/src/lib.rs | grep -A 30 'enum DataKey') \
     <(cat contracts/medical_records/src/lib.rs | grep -A 30 'enum DataKey')
```

Changes that **require** migration:

- Added variants to `DataKey` (new keys — additive, usually safe).
- Changed discriminant values in `DataKey` (breaking — old keys won't resolve).
- Changed value types for existing keys (breaking — deserialization fails).
- Removed variants (breaking — orphaned state).

### Step 2 — Write a Migration Function

If the change is breaking, add a migration entry point to the contract:

```rust
pub fn migrate(env: Env) {
    let contract_id = env.current_contract_address();
    // 1. Read old-format data
    // 2. Transform to new format
    // 3. Write back to new keys
    // 4. Remove old keys if needed
    env.storage()
        .persistent()
        .set(&DataKey::MigrationVersion, &1u32);
}
```

### Step 3 — Test the Migration

Use `soroban-sdk` test utilities:

```rust
#[test]
fn test_migration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, MyContract);
    let client = MyContractClient::new(&env, &contract_id);

    // Seed old-format data using raw storage writes
    // ...

    client.migrate();

    // Assert new-format data is correct
    // ...
}
```

### Step 4 — Deploy

1. Deploy the new WASM alongside the old one.
2. Call `migrate()` on the live contract (requires admin auth).
3. Verify state with a read-only call.
4. Optionally upgrade the WASM reference to point to the new binary.

---

## Schema Migration Procedure

Schema migrations affect off-chain consumers (indexers, SDKs, APIs) but do not
break on-chain state. Follow this procedure:

1. **Document** the schema change in `CHANGELOG.md` under *Storage & Schema
   Changes* with `Migration Required: No`.
2. **Add** a row to the migration guide table below.
3. **Notify** downstream integrators via release notes.

### Schema Migration Log

| Contract | Version | Change | Date |
|----------|---------|--------|------|
| medical_records | 1.1.0 | Added `partial_updates` field support | 2026-07-25 |

---

## Upgrading Contracts with Storage Changes

### Using UpgradeManager

```text
1. Admin proposes upgrade via UpgradeManager::propose_upgrade(contract, new_wasm_hash)
2. Validators review and approve (multi-sig threshold)
3. Admin calls UpgradeManager::execute_upgrade(contract)
4. If migration needed: admin calls contract::migrate()
5. Verify with read-only test calls
```

### Direct Upgrade (single-admin contracts)

```bash
soroban contract upgrade \
  --wasm target/wasm32-unknown-unknown/release/<contract>.wasm \
  --contract <CONTRACT_ID> \
  --source <ADMIN_SECRET_KEY> \
  --network testnet

# Then trigger migration if needed
soroban contract invoke \
  --id <CONTRACT_ID> \
  --fn migrate \
  --source <ADMIN_SECRET_KEY> \
  --network testnet
```

---

## Rollback Strategy

If a migration fails or introduces a regression:

1. **Immediately** call the rollback function (if available) or stop new
   transactions against the contract.
2. Deploy the previous WASM version via `UpgradeManager`.
3. Restore state from the snapshot taken before migration (see
   `contracts/storage-snapshot`).
4. Post-mortem: document root cause in an incident report.

---

## Checklist

Before upgrading a contract with storage or schema changes:

- [ ] `CHANGELOG.md` has an entry in *Storage & Schema Changes* table.
- [ ] `docs/MIGRATION_GUIDE.md` (this file) is updated with the procedure.
- [ ] Migration function is implemented and unit-tested.
- [ ] `scripts/check_changelog.sh` passes.
- [ ] Rollback plan is documented.
- [ ] Off-chain consumers notified of schema changes.
- [ ] Upgrade tested on testnet before mainnet.
