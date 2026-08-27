# Contract Behavior Fuzzing Harness

This directory contains a property-based fuzzing and invariant testing harness for Uzima smart contracts. It is a test-only component that runs natively (not as a WASM contract) to validate contract invariants, serialization correctness, and behavior under random input sequences.

## Supported Versions

| Component | Version |
|-----------|---------|
| Rust | 1.92.0 |
| Soroban SDK | 21.7.7 |
| Proptest | 1.6.0 |

## Workspace Relationship

This crate is intentionally excluded from the root workspace because:
1. It is a **test-only harness** that does not deploy to any network
2. It runs natively (not `#[no_std]` or WASM-compatible) - it depends on `std::panic` and other host-level features
3. It is not required for standard contract development workflows
4. It has different build requirements and runtime characteristics than production contracts

To run the fuzz tests, you must explicitly target this crate (see commands below).

## Maintained Fuzz Targets

All fuzz targets are located in the `tests/` directory:

| Target File | Description | Contracts Tested |
|-------------|-------------|------------------|
| `medical_records_serde_fuzz.rs` | Validates serialization/deserialization round-tripping of all medical record types | `medical_records` |
| `identity_registry_fuzz.rs` | Tests access control and identity management invariants | `identity_registry` |
| `sut_token_fuzz.rs` | Tests token economics and transfer invariants | `sut_token` |
| `token_sale_fuzz.rs` | Tests token sale mechanics and invariants | `token_sale` |
| `access_control_fuzz.rs` | Tests role-based access control and permission boundaries | multiple |
| `consent_handling_fuzz.rs` | Tests patient consent management workflows | multiple |
| `cross_contract_fuzz.rs` | Tests cross-contract interaction atomicity and state consistency | multiple |

## Bounded Verification Command

To run a bounded verification (property-based testing with controlled input complexity) that passes reliably from a clean checkout:

```bash
cd contracts/contract_behavior_fuzzing
cargo test -- --nocapture
```

This command runs all fuzz targets with proptest's default bounds, which is sufficient to catch common bugs while completing quickly. For longer fuzz campaigns, you can increase the case count:

```bash
cargo test -- --nocapture --test-threads=1 PROPTEST_CASES=10000
```

## Expected Outcomes

- **Passing run**: All tests complete successfully with exit code 0
- **Failing run**: A failing test will output a `CrashReport` with:
  - The failing operation index
  - The sequence of operations that led to the failure
  - The panic message from the assertion that failed
- **Minimization**: Proptest automatically minimizes failing test cases to the smallest reproducing sequence

## Regression Testing

The harness includes a `run_regressions` function that executes previously discovered failing cases as deterministic unit tests. These ensure that bugs fixed in the past do not regress.

## CI Integration

This harness runs on a scheduled basis in CI (not on every PR) to balance thoroughness with development velocity. It can be manually triggered by adding `[fuzz]` to a commit message. See `.github/workflows/fuzzing.yml` for the GitHub Actions configuration.