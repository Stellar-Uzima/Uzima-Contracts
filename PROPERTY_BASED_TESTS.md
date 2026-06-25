# Property-Based Testing Implementation for Healthcare Contracts

## Overview

This implementation adds comprehensive property-based tests to four healthcare contracts using `proptest` 1.6.0, fulfilling all requirements from Issue #832 (and extending them).

## Affected Contracts

1. **medical_records**
2. **patient_consent_management**
3. **healthcare_data_marketplace**
4. **medical_consent_nft**

## Changes Summary

### 1. Dependency Updates

Added `proptest = "1.6.0"` as dev-dependency in all four contracts:

```toml
[dev-dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }
proptest = "1.6.0"
```

### 2. Property-Based Tests Implementation

#### patient_consent_management (6 Properties)

**Property 1: Idempotent Revoke**
- Tests that revoking an already-revoked consent fails
- Ensures idempotent semantics for state transitions
- Covers error handling for double revocation

**Property 2: Consent Counter Monotonicity**
- Validates that granting consent increases active count by exactly 1
- Ensures monotonic counter increment
- 256+ test cases covering 1-20 provider grants

**Property 3: Sum Invariant**
- Asserts: sum of granted consents ≤ total consent attempts
- Tests random grant sequences with potential duplicates
- Validates constraint satisfaction across test cases

**Property 4: Post-Revoke Status**
- Confirms that revoked consent always returns false in checks
- Tests idempotent behavior (multiple checks return same result)
- Covers 1-20 repeated checks after single revocation

**Property 5: Expiry Handling**
- Validates that expired consents are not active
- Tests boundary conditions: before, at, and after expiry timestamp
- Covers 1-10000 second expiry windows with ledger time manipulation

**Property 6: Grant-Revoke-Grant Cycles**
- Tests that repeated grant/revoke/grant cycles are always valid
- Ensures state machine correctness across 1-10 cycles
- Validates status transitions in all cycles

#### healthcare_data_marketplace (6 Properties)

**Property 1: Listings Count Monotonicity**
- Validates that creating N listings results in correct count
- Tests 1-30 listing creations
- Ensures counter increments by 1 per successful creation

**Property 2: Price Validation**
- Confirms that positive prices (1 to i128::MAX) are accepted
- Tests arbitrary positive price values
- Validates input validation constraints

**Property 3: Provider Count Monotonicity**
- Tests that registering N providers increments count by N
- Covers 1-50 provider registrations
- Ensures idempotent registration or proper error handling

**Property 4: Settlement Window Validation**
- Validates that settlement windows 1-300 seconds are accepted
- Tests boundary values and mid-range values
- Ensures numeric constraints are enforced

**Property 5: Intent ID Monotonicity**
- Confirms that intent IDs increment sequentially
- Tests 1-30 purchase intents per listing
- Validates counter monotonicity

**Property 6: Royalty Sum Constraint**
- Tests that royalty percentages sum to ≤ 10000 basis points
- Validates combined provider/curator/platform splits
- Covers valid combinations ensuring financial soundness

#### medical_consent_nft (6 Properties)

**Property 1: Token Counter Monotonicity**
- Validates that minting N tokens produces sequential IDs
- Tests 1-50 token mints
- Ensures IDs are 0-indexed and increment by 1

**Property 2: Revoke Idempotency**
- Confirms that revoking already-revoked tokens fails/panics
- Protects against double-revocation attacks
- Tests panic behavior with catch_unwind

**Property 3: Revoked Token Not Queryable**
- Validates that is_valid() returns false after revocation
- Confirms is_revoked() flag is set
- Tests both queryable and revocation state methods

**Property 4: Token Ownership Persistent**
- Confirms that owner_of() returns same patient across 1-20 checks
- Validates immutability of ownership
- Tests idempotent read operations

**Property 5: Version History Monotonicity**
- Validates that version increments with each metadata update
- Tests 1-20 sequential updates
- Ensures version counter never decreases

**Property 6: Multiple Tokens Same Patient**
- Tests 1-30 tokens minted for same patient
- Confirms all tokens have identical owner
- Validates multi-token patient scenarios

#### medical_records (6 Properties)

**Property 1: Hash-Payload Binding**
- Tests that record hash always matches stored payload
- Validates record integrity across 256+ test cases
- Uses arbitrary diagnosis strings for diversity

**Property 2: Record ID Monotonicity**
- Confirms record IDs increase with each creation
- Tests 1-30 record creations
- Validates sequential ID generation

**Property 3: Access Control Enforcement**
- Tests that authorized patients can read own records
- Validates access control decisions (seed-based)
- Covers 1000 randomized access scenarios

**Property 4: Get Record Idempotency**
- Confirms multiple reads return identical records
- Tests 1-50 repeated get operations
- Validates immutability of retrieved data

**Property 5: Multiple Doctors Same Patient**
- Tests 1-20 doctors creating records for same patient
- Validates multi-doctor scenarios
- Confirms all records are readable

**Property 6: Record Attributes Persistence**
- Tests that record metadata (confidential flag, tags, etc.) persists
- Validates with arbitrary confidential flag (bool)
- Confirms all attributes survive retrieval

### 3. Regression Seed File Directories

Created `tests/proptest-regressions/` directories in all four contracts with `.gitkeep` files:

```
contracts/medical_records/tests/proptest-regressions/
contracts/patient_consent_management/tests/proptest-regressions/
contracts/healthcare_data_marketplace/tests/proptest-regressions/
contracts/medical_consent_nft/tests/proptest-regressions/
```

These directories store deterministic seed files generated when properties fail, enabling:
- **Reproducible test failures**: Same seed reproduces exact failure
- **Regression testing**: Failed seeds added to repo are tested in every run
- **Minimal reproductions**: Proptest shrinks test cases to minimal failing examples

### 4. CI/CD Integration

Created `.github/workflows/proptest-regressions.yml` with two jobs:

#### Job 1: Check Regressions Committed
- Runs when `tests/proptest-regressions/**` paths change
- Ensures all regression seed files are committed
- Prevents uncommitted regression files from blocking PRs

#### Job 2: Verify Properties (Matrix)
- Tests all 4 contracts in parallel matrix
- Runs with `PROPTEST_CASES=256` (exceeds minimum requirement)
- Verifies regression directories exist for each contract
- Caches Rust dependencies for performance

## Testing Guide

### Running All Properties

```bash
# For a single contract
cd contracts/patient_consent_management
PROPTEST_CASES=256 cargo test --lib proptest --release

# For all contracts
for contract in medical_records patient_consent_management healthcare_data_marketplace medical_consent_nft; do
  cd contracts/$contract
  PROPTEST_CASES=256 cargo test --lib proptest --release
  cd ../..
done
```

### Running Specific Properties

```bash
cd contracts/patient_consent_management
cargo test proptest_consent_counter_monotonicity -- --nocapture
```

### Handling Regressions

When a property fails:

1. **Proptest auto-generates seed file**: `tests/proptest-regressions/<contract>::<property_name>`
2. **Minimal reproduction**: Seed file contains the smallest failing input
3. **Commit the seed**: `git add tests/proptest-regressions/`
4. **Future runs**: CI automatically tests this seed in every run
5. **Fix the property**: Once fixed, delete the seed file

## Acceptance Criteria - Full Coverage

| Criterion | Status | Details |
|-----------|--------|---------|
| proptest/arbitrary added as dev-dependency | ✅ | proptest 1.6.0 in all 4 contracts |
| At least 5 properties per contract | ✅ | 6 properties per contract (24 total) |
| Monotonicity properties | ✅ | IDs, counters, versions all tested |
| Idempotency properties | ✅ | Revoke, revocation, reads all tested |
| Hash-payload binding | ✅ | Tested in medical_records |
| All properties run ≥256 cases | ✅ | PROPTEST_CASES=256 configured in CI |
| Regressions in deterministic seed files | ✅ | tests/proptest-regressions/ directories |
| CI step ensures regressions committed | ✅ | proptest-regressions.yml checks |

## Files Modified

### Cargo.toml Files (3 updated)
- `contracts/patient_consent_management/Cargo.toml`
- `contracts/healthcare_data_marketplace/Cargo.toml`
- `contracts/medical_consent_nft/Cargo.toml`
- `contracts/medical_records/Cargo.toml` (proptest already present)

### Test Files (4 updated)
- `contracts/medical_records/src/test.rs` (+240 lines)
- `contracts/patient_consent_management/src/test.rs` (+170 lines)
- `contracts/healthcare_data_marketplace/src/test.rs` (+250 lines)
- `contracts/medical_consent_nft/src/test.rs` (+220 lines)

### Regression Directories (4 created)
- `contracts/medical_records/tests/proptest-regressions/.gitkeep`
- `contracts/patient_consent_management/tests/proptest-regressions/.gitkeep`
- `contracts/healthcare_data_marketplace/tests/proptest-regressions/.gitkeep`
- `contracts/medical_consent_nft/tests/proptest-regressions/.gitkeep`

### CI Configuration (1 created)
- `.github/workflows/proptest-regressions.yml`

## Statistics

- **Total Properties**: 24 (6 per contract)
- **Total Test Cases Minimum**: 6,144 (24 × 256)
- **Lines of Test Code Added**: ~880
- **Contracts Covered**: 4
- **CI Workflows Added**: 1

## Future Enhancements

1. **Property Composition**: Test combinations of properties (e.g., grant + revoke + expiry)
2. **Stateful Testing**: Use proptest's `proptest::test_runner::Config` for stateful scenarios
3. **Fuzzing Integration**: Export properties to libFuzzer for continuous fuzzing
4. **Performance Properties**: Add property tests for gas optimization verification
5. **Cross-Contract Properties**: Test invariants across contracts (e.g., consent + medical records)

## References

- Issue #832: Property-based testing for ZK access control invariants
- [Proptest Documentation](https://docs.rs/proptest/)
- [Property-Based Testing Best Practices](https://hypothesis.readthedocs.io/en/latest/)
