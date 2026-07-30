# WASM Bloat Reduction Guide

This document identifies the highest-complexity contracts by binary size and
provides actionable techniques to reduce WASM bloat.

## Current Size Baseline

Run `./scripts/wasm_size_monitor.sh` after building to get current sizes.
The contracts identified as highest-complexity (largest WASM footprint) are:

| Contract | Approx. size | Primary bloat sources |
|----------|-------------|----------------------|
| `telemedicine` | ~65 KB | Large enum variants, many string literals |
| `fido2_authenticator` | ~65 KB | Crypto boilerplate, format strings |
| `zkp_registry` | ~73 KB | Polynomial math, large lookup tables |
| `medical_consent_nft` | ~54 KB | NFT metadata, event strings |
| `healthcare_payment` | ~53 KB | Payment state machine, many error paths |

## Optimization Techniques

### 1. Remove format! and string formatting

`format!` pulls in `core::fmt` infrastructure which adds ~2-4 KB per contract.

**Before:**
```rust
panic!("invalid state: {}", state_value);
```

**After:**
```rust
panic!("invalid state");  // or use a numeric error code
```

### 2. Inline small helper functions

Mark hot helpers `#[inline]` to avoid function call overhead in WASM:

```rust
#[inline(always)]
pub fn require_active(state: &State) -> Result<(), Error> {
    if state != &State::Active { return Err(Error::NotActive); }
    Ok(())
}
```

### 3. Avoid monomorphisation of generic types

Soroban's `Vec<T>` and `Map<K,V>` generate WASM code for every concrete type
combination. Prefer storing data as `Bytes` or `String` when the contract only
needs to pass data through without inspecting it:

```rust
// BLOAT: generates separate WASM code for Vec<Record>, Vec<u64>, Vec<String>
// BETTER: store as serialised Bytes, deserialise only when needed
```

### 4. Use `symbol_short!` instead of `Symbol::new`

`Symbol::new(&env, "long_symbol_name")` encodes at runtime.
`symbol_short!("short")` is a compile-time constant (max 9 chars):

```rust
// Before:
env.events().publish((Symbol::new(&env, "record_created"),), &data);

// After:
env.events().publish((symbol_short!("rec_crtd"),), &data);
```

### 5. Link-Time Optimization (already enabled)

The workspace `Cargo.toml` already sets:
```toml
[profile.release]
lto = true
opt-level = "z"
codegen-units = 1
```

Verify these are present in every contract's `Cargo.toml` or inherited from workspace.

### 6. Dead code elimination

Run `cargo bloat --release --crates` to identify the top code contributors.
Common culprits: unused trait implementations, dead enum variants.

```bash
cargo install cargo-bloat
cargo bloat --release -p <contract_name> --crates
```

### 7. Avoid `derive(Debug)` on production types

`#[derive(Debug)]` adds a fmt implementation that references string literals.
In `#[no_std]` contracts, remove `Debug` from `#[contracttype]` structs
that are not used in tests:

```rust
// Development only — remove Debug from production structs
#[contracttype]
#[derive(Clone)]  // not Debug
pub struct MedicalRecord { ... }
```

## CI Size Gate

The existing `scripts/wasm_size_monitor.sh` checks binary size. Thresholds
are in `docs/CONTRACT_RESOURCE_LIMITS.md`:

- Warning: > 51.2 KB (80% of 64 KB limit)
- Critical: > 62.3 KB (95% of 64 KB limit)

New contracts must pass the size gate before merging.
