# no_std Compliance Guide for Soroban Smart Contracts

## Overview

All Uzima smart contracts must be `#![no_std]` compliant because Soroban contracts run in a WebAssembly environment that does not support the Rust standard library. This document outlines common pitfalls, verification steps, and best practices.

## Why no_std?

- Soroban smart contracts compile to WebAssembly (WASM) via the `wasm32-unknown-unknown` target
- The Rust standard library (`std`) is not available in this environment
- Only the `core` and `alloc` crates are available
- Using `std` imports will cause compilation failures

## Required Attributes

Every contract's `src/lib.rs` MUST include:

```rust
#![no_std]
```

Contracts that use custom entry points should also include:

```rust
#![no_main]
```

## Common Pitfalls

### 1. `format!` Macro

The `format!` macro requires `std`. Use Soroban's `String::from_str` instead:

```rust
// WRONG - uses std::fmt
let msg = format!("Patient {} has record {}", patient_id, record_id);

// CORRECT
let msg = soroban_sdk::String::from_str(&env, "Patient record created");
```

### 2. `println!` / `eprintln!` Macros

These macros require `std::io` and are not available:

```rust
// WRONG
println!("Record created: {}", record_id);

// CORRECT - use events for logging
env.events().publish((symbol_short!("LOG"),), record_id);
```

### 3. `std::collections`

Use Soroban SDK collections instead:

```rust
// WRONG
use std::collections::HashMap;

// CORRECT
use soroban_sdk::{Map, Vec};
```

### 4. `std::vec!` / `std::string`

Use Soroban SDK equivalents:

```rust
// WRONG
let v: Vec<u64> = vec![1, 2, 3];

// CORRECT
use soroban_sdk::vec;
let v: soroban_sdk::Vec<u64> = vec![&env, 1u64, 2u64, 3u64];
```

### 5. `std::error::Error` Trait

The standard Error trait is not available. Use `#[contracterror]` instead:

```rust
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 100,
    NotFound = 404,
}
```

### 6. `std::time` / `std::thread`

These modules are not available. Use `env.ledger().timestamp()` for time:

```rust
// CORRECT
let current_time = env.ledger().timestamp();
```

## Verification Steps

### Local Verification

```bash
# Build for WASM target
cargo build --target wasm32-unknown-unknown --release

# Check for std dependencies
cargo tree --target wasm32-unknown-unknown | grep -E "std |alloc"
```

### CI Verification

The CI pipeline automatically:

1. Builds every contract targeting `wasm32-unknown-unknown`
2. Verifies that `#![no_std]` is present in all contract `src/lib.rs` files
3. Runs `cargo clippy` for code quality

## Dependencies to Avoid

| Crate | Alternative |
|-------|-------------|
| `std` | `core`, `alloc`, `soroban-sdk` |
| `serde` | `soroban-sdk` built-in serialization |
| `chrono` | `env.ledger().timestamp()` |
| `rand` | `env.prng()` |
| `anyhow` | Custom error types with `#[contracterror]` |
| `thiserror` | Custom error types with `#[contracterror]` |
| `log` | `env.events().publish()` |

## Excluded Contracts

The following contracts are excluded from the workspace because they require `std`:

- `credential_notifications`
- `medical_imaging`
- `healthcare_compliance`
- `clinical_nlp`
- `clinical_decision_support`
- `remote_patient_monitoring`
- `healthcare_analytics_dashboard`
- `healthcare_data_marketplace`
- `telemedicine`
- `patient_portal`
- `mental_health_support`
- `patient_gamification`
- `medical_imaging_ai`
- `dicomweb_services`
- `health_data_access_logging`
- `mfa`
- `multi_region_orchestrator`
- `regional_node_manager`
- `digital_twin`
- `aml`
- `forensics`
- `audit`
- `rbac`
- `federated_learning`
- `sync_manager`
- `failover_detector`
- `healthcare_compliance_automation`
- `drug_discovery`
- `health_check`

These contracts should be migrated to `no_std` before workspace inclusion.
