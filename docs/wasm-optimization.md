# WASM Binary Size Optimization Notes for `medical_records`

## Problem
The `medical_records` WASM binary is large due to included dependencies and
unoptimized build settings. Smaller WASM reduces deployment cost and upload fees.

## Optimization Strategies Applied

### 1. Build Profile (`Cargo.toml` workspace)
The workspace `[profile.release]` already uses:
- `opt-level = "z"` — optimize for size
- `lto = true` — link-time optimization removes dead code across crates
- `codegen-units = 1` — single codegen unit enables maximum inlining/DCE
- `strip = "symbols"` — removes debug symbols from the binary

### 2. `wasm-opt` Post-Processing
Run after `cargo build --release --target wasm32-unknown-unknown`:
```sh
wasm-opt -Oz \
  --enable-bulk-memory \
  --enable-mutable-globals \
  --enable-sign-ext \
  --enable-nontrapping-float-to-int \
  --strip-debug --strip-producers \
  target/wasm32-unknown-unknown/release/medical_records.wasm \
  -o target/wasm32-unknown-unknown/release/medical_records.wasm
```

### 3. Dependency Audit
- Avoid pulling in `std`-heavy crates; all contracts use `#![no_std]`.
- Keep `soroban-sdk` pinned to the workspace version (`=21.7.7`).
- Do not add `serde`, `thiserror`, or other std-dependent crates.

### 4. Feature Flags
- Only enable `testutils` feature in `[dev-dependencies]`, never in release.

## Tracking
The CI `wasm-size-check.yml` workflow enforces a 64 KB hard limit and warns at
50 KB. Add the contract to `.github/wasm-size-exceptions.txt` only if it is a
pre-existing overage with a tracked remediation plan.