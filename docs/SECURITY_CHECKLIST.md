# Soroban Contract Security Checklist

Use this checklist before every PR and audit. Each item must be checked (✅) or explicitly waived with justification.

---

## 1. Access Control

- [ ] Every state-mutating function calls `address.require_auth()` before any logic (use the shared `require_admin!(env, caller)` or `require_role!(env, caller, role)` macros from `governance_commons` where applicable)
- [ ] Admin-only functions verify the caller is the stored admin address (preferably using `require_admin!(env, caller)`)
- [ ] Role checks are performed after `require_auth()`, not instead of it (preferably using `require_role!(env, caller, role)`)
- [ ] No function relies solely on caller address comparison without `require_auth()`
- [ ] Ownership transfer requires auth from the **current** owner, not the new one

## 2. Input Validation

- [ ] All `String` inputs have enforced maximum byte lengths
- [ ] All `Vec` inputs have enforced maximum element counts
- [ ] Numeric inputs are validated against expected ranges before use
- [ ] Address inputs are not assumed to be valid contracts without verification
- [ ] No user-supplied data is used as a storage key without sanitization

## 3. Arithmetic Safety

- [ ] All arithmetic uses checked operations (`checked_add`, `checked_sub`, etc.) or Soroban's overflow-checked environment
- [ ] Division operations guard against zero divisors
- [ ] No unchecked casts between integer types that could truncate values
- [ ] Token amounts and balances use `i128` as required by the Stellar token interface

## 4. State Management

- [ ] Contract initialization is guarded against re-initialization using the standardized [`init_guard`](#init-guard) helper (or explicitly waived — see below)
- [ ] All storage writes are atomic with their corresponding validation
- [ ] No partial state updates — either all writes succeed or none do
- [ ] Deleted/expired entries are cleaned up to avoid unbounded storage growth
- [ ] Storage key namespacing prevents collisions between different data types

### <a name="init-guard"></a>Re-initialization guard (required)

A contract whose `initialize`/`init` entry point can run more than once can have
its ownership stolen — the attacker simply re-initializes and installs
themselves as admin. This is a known, repeatedly-exploited attack class for
cloneable contracts. **Do not** hand-roll per-contract `has(&Admin)` /
`has(&Initialized)` checks; their semantics drifted between contracts. Use the
single standardized helper instead.

- **Source of truth:** `libs/governance_commons/src/init_guard.rs`
  (`governance_commons::init_guard`). Re-exported from `validation_utils` for
  contracts that already depend on it: `validation_utils::init_guard`.
- **Semantics:**
  - The **first** call to `init_guard(&env)` / `try_init_guard(&env)` marks the
    contract initialized and succeeds.
  - Every **subsequent** call is rejected — `init_guard` **panics**;
    `try_init_guard` returns `GovernanceError::AlreadyInitialized`.
  - The guard flips its dedicated flag **before** any admin/config write, so a
    re-init can never reach ownership-mutating code.
  - **Admin transfer is a separate, independent operation.** Rotating the admin
    is done by a dedicated `transfer_admin`-style function that authorizes the
    *current* admin and overwrites the admin key; it never touches the init
    flag. Re-initialization can therefore never be used as a backdoor admin
    transfer, and a legitimate admin transfer never re-opens initialization.
- **Usage** — call it as the first statement of `initialize`:

  ```rust
  // Result-returning initialize (most contracts):
  pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
      governance_commons::try_init_guard(&env).map_err(|_| Error::AlreadyInitialized)?;
      admin.require_auth();
      env.storage().instance().set(&DataKey::Admin, &admin);
      Ok(())
  }

  // Unit-returning initialize (panic-on-error contracts):
  pub fn initialize(env: Env, admin: Address) {
      governance_commons::init_guard(&env); // panics if called twice
      admin.require_auth();
      env.storage().instance().set(&DataKey::Admin, &admin);
  }
  ```

- **Waiver convention:** a contract that *intentionally* allows re-initialization
  (e.g. a stateless template or a contract with no admin/ownership to steal)
  must mark the `initialize` function with an explicit, greppable waiver comment
  stating why, so audits can distinguish a deliberate decision from a missing
  guard:

  ```rust
  // SECURITY: re-init allowed — <reason, e.g. "stateless; no admin/ownership stored">
  pub fn initialize(env: Env, ...) { ... }
  ```

## 5. Events & Audit Trail

- [ ] Every state-changing operation emits a corresponding event
- [ ] Events include enough context to reconstruct what changed (old/new values where relevant)
- [ ] Auth failures are not silently swallowed — they panic via `require_auth()`
- [ ] Admin actions (role grants, config changes) are always logged

## 6. Cross-Contract Calls

- [ ] All cross-contract calls use typed client interfaces, not raw invocations
- [ ] Return values from cross-contract calls are validated before use
- [ ] Reentrancy is considered: state is finalized before calling external contracts
- [ ] Contract addresses passed as arguments are validated (not zero/default)

## 7. Upgrade Safety

- [ ] Storage schema changes include a migration path
- [ ] New fields in `contracttype` structs are backward-compatible
- [ ] Deprecated functions emit a warning event and document the migration path
- [ ] The upgrade function is protected by admin auth

## 8. Build & Deployment

- [ ] Contract is built with `opt-level = "z"`, `lto = true`, `codegen-units = 1`
- [ ] WASM binary size is under the 60 KB warning threshold (see `scripts/wasm_size_monitor.sh`)
- [ ] No `debug_assertions` or test-only code paths are reachable in release builds
- [ ] Contract has been deployed and smoke-tested on testnet before mainnet

## 9. Test Coverage

- [ ] Unit tests cover the happy path for every public function
- [ ] Unit tests cover auth-failure cases (unauthorized callers are rejected)
- [ ] Unit tests cover boundary/edge cases for all validated inputs
- [ ] Integration tests verify cross-contract interactions
- [ ] Fuzz tests exist for functions that accept arbitrary byte inputs

## 10. Documentation

- [ ] All public functions have doc comments explaining parameters, return values, and errors
- [ ] Error variants are documented with the conditions that trigger them
- [ ] Auth requirements are stated explicitly in function doc comments
- [ ] `CHANGELOG.md` is updated for any breaking changes

---

## PR Requirement

All PRs that add or modify contract functions must include a completed copy of this checklist in the PR description. Automated CI checks enforce items 8 (WASM size) and 9 (test coverage).

## Audit Requirement

Contracts handling patient data, payments, or admin privileges must pass an external audit before mainnet deployment. Reference: `docs/audit/CONTRACT_AUDIT_GUIDE.md`.
