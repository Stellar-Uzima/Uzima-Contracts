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