# Contract Behavior Fuzzing & Invariant Testing Program

This document defines the repository-wide fuzzing and invariant testing strategy for high-risk contract workflows. For the harness-specific implementation details, see [contracts/contract_behavior_fuzzing/README.md](../contracts/contract_behavior_fuzzing/README.md).

## Supported Versions

The fuzzing harness is maintained for the following versions:

| Component | Version |
|-----------|---------|
| Rust | 1.92.0 |
| Soroban SDK | 21.7.7 |
| Proptest | 1.6.0 |

## Workspace Relationship

The fuzzing harness is an active **workspace member** but remains a test-only component that runs natively (not as a WASM contract). Key benefits of this integration:
- ✅ Automatically inherits the workspace's pinned `soroban-sdk = "=21.7.7"` version
- ✅ Maintains a single consistent dependency graph across the entire repository
- ✅ Still only runs when explicitly targeted - it doesn't interfere with production builds or standard test workflows

To run the fuzz tests, you must still explicitly target the harness crate (see command below).

## Maintained Fuzz Targets

The following fuzz targets are actively maintained under `contracts/contract_behavior_fuzzing/tests/`:

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

For reproducible verification that passes from a clean checkout, run the bounded fuzzing suite with proptest's default bounds:

```bash
cd contracts/contract_behavior_fuzzing
cargo test -- --nocapture
```

This command executes all fuzz targets with a manageable number of test cases, ensuring it completes quickly while still validating core invariants.

### Expected Exit Status & Artifacts

- **Successful run**: Exit code 0, all tests pass
- **Failed run**: Non-zero exit code, with a `CrashReport` output that includes:
  - The index of the failing operation
  - The full sequence of operations that led to the failure
  - The panic message from the violated invariant
- **Regression failures**: If a previously fixed bug reoccurs, the harness will panic with the name of the failing regression case

## High-Risk Workflows

The following workflows are identified as highest priority for fuzz testing:

### 1. Access Control & Authorization
- **Contracts**: `identity_registry`, `medical_records`, `healthcare_compliance`
- **Invariants**: No unauthorized access to patient data; role escalation impossible; permission boundaries enforced
- **Fuzz targets**: Random role assignments, permission checks with random addresses, concurrent access attempts

### 2. Consent Handling
- **Contracts**: `patient_consent_management`, `medical_records`
- **Invariants**: Consent required before data access; expired consent blocked; consent revocation immediate
- **Fuzz targets**: Random consent grants/revocations, expired consent access attempts, consent state transitions

### 3. Cross-Contract Interactions
- **Contracts**: `cross_chain_bridge`, `medical_record_backup`, `fhir_integration`
- **Invariants**: State consistency across contracts; no orphaned references; rollback atomicity
- **Fuzz targets**: Random cross-contract calls, interrupted migration sequences, concurrent cross-chain operations

### 4. State Transitions
- **Contracts**: `medical_records`, `clinical_trial`, `audit`
- **Invariants**: Valid state machine transitions only; no invalid state combinations; history integrity
- **Fuzz targets**: Random state transitions, invalid transition attempts, concurrent state modifications

### 5. Payment & Escrow
- **Contracts**: `healthcare_payment` (if exists), `reputation_access_control`
- **Invariants**: Balance consistency; no double-spend; escrow release only on valid conditions
- **Fuzz targets**: Random payment amounts, concurrent transactions, escrow condition violations

## Invariant Categories

### State Invariants
- Data integrity: All records have required fields
- Referential integrity: Foreign keys point to existing records
- Temporal integrity: Timestamps are monotonically increasing

### Authorization Invariants
- Role hierarchy respected
- Permission boundaries enforced
- No privilege escalation

### Financial Invariants
- Balance conservation
- No negative balances
- Escrow conditions met before release

### Cross-Contract Invariants
- State consistency across contracts
- No orphaned references
- Atomic operations where required

## Fuzzing Strategy

### Property-Based Testing (Proptest)
- Use `proptest` for generating random inputs
- Define strategies for contract types
- Run with configurable size and complexity

### State Machine Testing
- Define valid state transitions
- Generate random transition sequences
- Verify invariants after each transition

### Regression Testing
- Minimize failing cases to deterministic tests
- Add to regression suite
- Document root cause

## CI Integration

The fuzzing tests run on a schedule and can be triggered manually:

```yaml
# In ci.yml
fuzzing:
  name: Contract Behavior Fuzzing
  runs-on: ubuntu-latest
  if: github.event_name == 'schedule' || contains(github.event.head_commit.message, '[fuzz]')
  steps:
    - uses: actions/checkout@v4
    - name: Run fuzz tests
      run: cargo test -p contract_behavior_fuzzing -- --nocapture
```

## Adding New Fuzz Tests

1. Identify the high-risk workflow
2. Define invariants as assertions
3. Create fuzz operations enum
4. Implement the harness
5. Define proptest strategies
6. Add regression test for any failures
7. Document the workflow in this file