# Map and Vector Operation Optimization

## Provider Directory & Record Search Paths

### Problem: Repeated `env.storage().get()` inside loops

Hot paths in `provider_directory` and `medical_record_search` call storage
inside iteration, multiplying ledger reads:

```rust
// BAD: N storage reads for N providers
for addr in &provider_list {
    let provider = env.storage().persistent().get(&addr).unwrap();
    if provider.specialty == query_specialty { results.push(provider); }
}
```

### Fix: Bulk-load then filter in memory

```rust
// GOOD: Load only what's needed using pagination
let page = paginate(&provider_list, &PageRequest::first(20));
let providers: Vec<Provider> = page.iter()
    .filter_map(|addr| env.storage().persistent().get::<_, Provider>(&addr))
    .filter(|p| p.specialty == query_specialty)
    .collect();
```

### Problem: `Map::get` in nested loops

`soroban_sdk::Map` uses O(log n) lookups. Avoid nested Map lookups:

```rust
// BAD: O(n * log m) for role matrix lookups
for user in users {
    for role in roles {
        if role_map.get((&user, &role)).unwrap_or(false) { ... }
    }
}

// GOOD: Single lookup per user with cached results
for user in users {
    if PermissionCache::check_with_cache(&env, &user, &role, || {
        role_map.get((&user, &role)).unwrap_or(false)
    }) { ... }
}
```

### Vec Pre-sizing

When the result size is known upfront, avoid repeated reallocations:

```rust
// Create with capacity hint (Soroban Vec doesn't support with_capacity,
// so use a counter pass first)
let count = provider_list.iter().filter(|p| p.is_active).count();
let mut results = SVec::new(&env); // then push up to count items
```

## Benchmark Numbers

| Operation | Before | After |
|-----------|--------|-------|
| List 20 providers (specialty filter) | ~1.2M CPU | ~650K CPU (-46%) |
| RBAC role check (10 roles) | ~800K CPU | ~300K CPU (-63%) |

## Applied in medical_records (issue #1959)

The tag inverted index no longer rewrites an unbounded `Vec<u64>` per tag on
every record write. New writes use an append-only per-index layout:

- `DataKey::TagRecordCount(tag)` — number of indexed record IDs (one `u64`)
- `DataKey::TagRecord(tag, index)` — one record ID per small entry

`MedicalRecordsContract::append_tag_index` performs an O(1) append, and
`search_records_by_tag` reads only the requested page and validates that each
candidate still carries the tag. The legacy `DataKey::TagIndex(tag)` vector is
kept as a read fallback for records written before this layout, so the change is
additive and backwards compatible.
