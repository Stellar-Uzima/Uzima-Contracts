# Shared Storage Abstraction

The `libs/validation_utils/src/storage_layout.rs` module provides a
consistent, type-safe API for Soroban storage access across all Uzima
contracts, replacing the ad-hoc patterns previously scattered throughout
the codebase.

## Storage Tiers

| Store | Soroban type | TTL | Use for |
|-------|-------------|-----|---------|
| `InstanceStore` | Instance | 60 days | Admin, config, flags |
| `PersistentStore` | Persistent | 30 days (auto-bumped) | Per-entity data |
| `TempStore` | Temporary | 1 hour | Rate limits, nonces |

## Quick Start

```rust
use validation_utils::storage_layout::{InstanceStore, PersistentStore, TempStore, InitGuard};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    PatientRecord(Address),
    RateLimit(Address),
}

// Initialize once
pub fn initialize(env: Env, admin: Address) {
    admin.require_auth();
    InitGuard::assert_not_initialized(&env);
    InstanceStore::set(&env, &DataKey::Admin, &admin);
    InitGuard::mark_initialized(&env);
}

// Write persistent data
pub fn store_record(env: Env, patient: Address, data: String) {
    InitGuard::assert_initialized(&env);
    PersistentStore::set(&env, &DataKey::PatientRecord(patient), &data);
}

// Read with automatic TTL bump
pub fn get_record(env: Env, patient: Address) -> Option<String> {
    PersistentStore::get(&env, &DataKey::PatientRecord(patient))
}

// Rate limiting with temporary storage
pub fn check_rate_limit(env: Env, caller: Address) -> bool {
    TempStore::has(&env, &DataKey::RateLimit(caller.clone()))
}
```

## Migration from Ad-Hoc Patterns

### Before (ad-hoc)

```rust
// Scattered TTL constants
const LEDGER_LIFETIME: u32 = 17_280;

fn get_admin(e: &Env) -> Option<Address> {
    e.storage().instance().get(&symbol_short!("admin"))
}

fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&symbol_short!("admin"), admin);
}

fn get_record(e: &Env, id: u64) -> Option<Record> {
    let val = e.storage().persistent().get(&id);
    if val.is_some() {
        // Manual TTL bump — easy to forget
        e.storage().persistent().extend_ttl(&id, 1000, 17280);
    }
    val
}
```

### After (shared abstraction)

```rust
fn get_admin(e: &Env) -> Option<Address> {
    InstanceStore::get(e, &symbol_short!("admin"))
}

fn set_admin(e: &Env, admin: &Address) {
    InstanceStore::set(e, &symbol_short!("admin"), admin);
}

fn get_record(e: &Env, id: u64) -> Option<Record> {
    PersistentStore::get(e, &id) // TTL bump is automatic
}
```

## TTL Constants

| Constant | Value | Approximate duration |
|----------|-------|---------------------|
| `INSTANCE_TTL_LEDGERS` | 1,036,800 | 60 days |
| `PERSISTENT_TTL_LEDGERS` | 518,400 | 30 days |
| `PERSISTENT_TTL_THRESHOLD` | 414,720 | 24 days |
| `TEMP_TTL_LEDGERS` | 720 | 1 hour |

TTL values assume ~5 second ledger close times.
Adjust `PERSISTENT_TTL_LEDGERS` for contracts with longer expected lifespans.
