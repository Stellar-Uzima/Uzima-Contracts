# Security Review Checklist for New Contract Submissions

Every new contract submitted to this repository must pass this security review before merge. This checklist is tailored for new contract onboarding and complements the general [Security Checklist](SECURITY_CHECKLIST.md).

---

## Pre-Submission (Author)

Before requesting review, confirm every item below.

### 1. Contract Structure

- [ ] Contract lives in `contracts/<snake_case_name>/`
- [ ] Has `src/lib.rs` with `#[contract]` and `#[contractimpl]` attributes
- [ ] Has `src/errors.rs` with `#[contracterror]` enum
- [ ] Has `src/types.rs` for non-trivial type definitions
- [ ] Has `src/events.rs` for event emission helpers
- [ ] Has `src/test.rs` with unit tests
- [ ] `Cargo.toml` uses workspace dependencies (`soroban-sdk = { workspace = true }`)
- [ ] Contract compiles: `cargo check --package <name> --all-targets`

### 2. Initialization Safety

- [ ] `initialize()` function exists and is the first entry point
- [ ] Uses `governance_commons::try_init_guard(&env)` (or `init_guard`) as the first statement
- [ ] Re-initialization is rejected (returns `AlreadyInitialized` or panics)
- [ ] Admin address is stored during initialization
- [ ] `admin.require_auth()` is called before any state writes

### 3. Access Control

- [ ] Every state-mutating function calls `caller.require_auth()` before any logic
- [ ] Admin-only functions verify the caller is the stored admin (use `require_admin!` macro)
- [ ] Role-based functions use `require_role!` macro from `governance_commons`
- [ ] Read-only functions do not require authentication
- [ ] Ownership transfer requires auth from the **current** owner

### 4. Input Validation

- [ ] All `String` inputs have maximum byte length enforcement
- [ ] All `Vec` inputs have maximum element count enforcement
- [ ] Numeric inputs are validated against expected ranges
- [ ] Address inputs are validated (not zero address, not self)
- [ ] No user-supplied data is used as a storage key without sanitization

### 5. Arithmetic Safety

- [ ] All arithmetic uses `checked_add`, `checked_sub`, `checked_mul`, `checked_div`
- [ ] Division operations guard against zero divisors
- [ ] No unchecked casts between integer types
- [ ] Token amounts use `i128` as required by the Stellar token interface

### 6. Error Handling

- [ ] All fallible functions return `Result<T, Error>`
- [ ] Error variants use `PascalCase` and are `#[contracterror]`
- [ ] Error codes are assigned in the standard ranges (see `CONTRACT_NAMING_CONVENTIONS.md`)
- [ ] Error messages do not leak internal implementation details
- [ ] No `unwrap()` or `expect()` in production code paths

### 7. Events

- [ ] Every state-changing operation emits a corresponding event
- [ ] Event topics use `snake_case` (e.g., `symbol_short!("record_created")`)
- [ ] Events include enough context to reconstruct what changed
- [ ] Admin actions are always logged

### 8. Storage

- [ ] Uses `DataKey` enum for storage keys (not raw strings)
- [ ] Storage keys are namespaced to prevent collisions
- [ ] `instance().set()` for singleton data, `persistent().set()` for per-entity data
- [ ] `temporary().set()` for ephemeral data only
- [ ] No unbounded storage growth (entries are cleaned up or bounded)

### 9. Cross-Contract Calls

- [ ] Uses typed client interfaces, not raw invocations
- [ ] Return values are validated before use
- [ ] State is finalized before calling external contracts (reentrancy safety)
- [ ] Contract addresses passed as arguments are validated

### 10. Testing

- [ ] Unit tests cover the happy path for every public function
- [ ] Unit tests cover auth-failure cases (unauthorized callers)
- [ ] Unit tests cover boundary/edge cases (empty inputs, max lengths, zero values)
- [ ] `env.mock_all_auths()` is used in all tests
- [ ] Tests use `Address::generate(&env)` for test addresses
- [ ] At least one test verifies re-initialization is rejected

### 11. Documentation

- [ ] Module-level `//!` doc comment at the top of `lib.rs` describes purpose, dependencies, initialization, roles, error ranges, and usage example
- [ ] Every public function has `///` doc comment with parameters, return values, errors, and auth requirements
- [ ] `README.md` in the contract directory describes the contract

### 12. Build & Deployment

- [ ] Contract builds with release profile (`opt-level = "z"`, `lto = true`)
- [ ] WASM binary size is under 60 KB
- [ ] No `debug_assertions` or test-only code reachable in release builds
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --all` passes

---

## Reviewer Checklist

The reviewer must verify the following during code review:

### Security

- [ ] All authorization checks are correct and complete
- [ ] No privilege escalation paths exist
- [ ] Input validation is comprehensive and correct
- [ ] Arithmetic operations are safe
- [ ] Cross-contract calls treat external contracts as untrusted
- [ ] No sensitive data is logged or emitted in events
- [ ] Storage patterns prevent data corruption

### Code Quality

- [ ] Code follows project naming conventions (`CONTRACT_NAMING_CONVENTIONS.md`)
- [ ] Code passes `cargo fmt` and `cargo clippy` without warnings
- [ ] Error types are consistent and well-defined
- [ ] Events are emitted for all state-changing operations
- [ ] No dead code or unused imports

### Testing

- [ ] Test coverage is adequate (happy path + error paths + edge cases)
- [ ] Tests are deterministic (no flaky tests)
- [ ] Tests use proper authorization mocking (`mock_all_auths`)
- [ ] Tests verify error conditions, not just success paths

### Documentation

- [ ] Public APIs are documented
- [ ] Auth requirements are stated explicitly
- [ ] Error conditions are documented
- [ ] README.md describes the contract's purpose and usage

---

## Rollout for New Contracts

1. **Author** completes the Pre-Submission checklist above
2. **Author** adds entry to `schemas/interface-registry/registry.json`
3. **Author** runs `node scripts/abi-compat.mjs` to update the ABI baseline
4. **Author** opens a PR with the checklist completed in the PR description
5. **Reviewer** completes the Reviewer checklist above
6. **CI** validates formatting, linting, tests, and WASM size
7. **Merge** after required approvals

---

## Related Documentation

- [Security Checklist](SECURITY_CHECKLIST.md) — general contract security checklist
- [Contract Review Checklist](contract-review-checklist.md) — review process checklist
- [Security Best Practices](SECURITY_BEST_PRACTICES.md) — secure coding patterns
- [Threat Models](MASTER_THREAT_MODEL.md) — threat model reference
- [Coding Standards](CODING_STANDARDS.md) — naming and style conventions
- [Contributing Guide](../CONTRIBUTING.md) — contributor onboarding

---

*Last updated: 2026-07-24*
