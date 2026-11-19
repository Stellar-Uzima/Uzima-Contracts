# Medical Records Storage Layout Optimization

## Problem
Separate ledger entries per field cause N reads per record access, inflating fees.

## Solution: Consolidated struct per record

Instead of storing each field under its own key, store the entire record as one
`contracttype` struct under a single key:

```rust
#[contracttype]
pub struct MedicalRecord {
    pub patient:    Address,
    pub provider:   Address,
    pub record_type: String,
    pub data_hash:  BytesN<32>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
pub enum DataKey {
    Record(u64),   // single persistent entry per record
    RecordCount,   // instance storage counter
}
```

## Before vs After

| Operation | Before (N fields) | After (1 struct) |
|---|---|---|
| Read record | N storage reads | 1 storage read |
| Write record | N storage writes | 1 storage write |
| Ledger fee | O(N) | O(1) |

## Migration note
Existing records stored under the old layout must be migrated via an admin
`migrate_record(id)` function that reads old keys, writes the new struct, and
deletes the old entries.
## Current Layout (issue #1961)

The consolidated single-entry layout is now the canonical storage form:

- `DataKey::Record(u64)` stores one `MedicalRecord` struct per record.
- `DataKey::RecordCount` tracks the number of records.
- `DataKey::RecordSchemaVersion(u64)` stamps `STORAGE_SCHEMA_VERSION` (= 2).
- Growth-prone tag lists are indexed via `DataKey::TagRecordCount(tag)` and
  `DataKey::TagRecord(tag, index)` so a tag search reads only the requested page
  instead of rewriting a full `Vec` (see `docs/performance/MAP_VECTOR_OPTIMIZATION.md`).

`migrate_record(id)` is the admin migration entry point for legacy
field-per-key records: it reads `DataKey::LegacyRecordField(id, field)`, writes
the consolidated struct once, stamps the schema version and removes the old
keys. It returns `Error::AlreadyInitialized` for an already-migrated record, so
it is safe to replay.

## Leaderboard verification

Storage reductions are verified by the same commands CI runs:

```bash
cargo build --workspace --target wasm32-unknown-unknown --release
./scripts/measure_storage.sh
./scripts/update_readme_storage_leaderboard.sh
```

The checked-in `STORAGE_LEADERBOARD_START/END` block in `README.md` must match a
fresh measurement. Until a release build is run in CI, the table is explicitly
marked `unmeasured` rather than showing fabricated numbers.
