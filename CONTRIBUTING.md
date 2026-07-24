# Contributing to Stellar Uzima Contracts

Thank you for your interest in contributing to Stellar Uzima. This guide explains how to author contracts, submit changes, and meet review expectations so that every pull request can be merged with confidence.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Repository Layout](#repository-layout)
- [Development Workflow](#development-workflow)
- [Contract Authoring Patterns](#contract-authoring-patterns)
- [Naming Conventions](#naming-conventions)
- [Testing Requirements](#testing-requirements)
- [Documentation Standards](#documentation-standards)
- [Review Expectations](#review-expectations)
- [Pull Request Process](#pull-request-process)
- [Event Topic Naming Convention](#event-topic-naming-convention)
- [Security Guidelines](#security-guidelines)
- [Getting Help](#getting-help)

---

## Getting Started

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.92.0 | `rustup default 1.92.0` |
| Soroban CLI | 21.7.7 | `cargo install --locked --version 21.7.7 soroban-cli` |
| Make | any | Usually pre-installed on macOS/Linux |
| Git | 2.x | [git-scm.com](https://git-scm.com/) |

### Setup

```bash
git clone https://github.com/Stellar-Uzima/Uzima-Contracts.git
cd Uzima-Contracts
chmod +x setup.sh
./setup.sh
```

Or step-by-step:

```bash
rustup target add wasm32-unknown-unknown
rustup component add rustfmt clippy rust-src
make build
make test
```

### Verify your environment

```bash
make check        # runs fmt + clippy + test in one command
```

---

## Repository Layout

```
Uzima-Contracts/
├── contracts/           # 100+ Soroban smart contracts
│   ├── medical_records/
│   ├── patient_consent_management/
│   ├── access_control/
│   └── ...
├── libs/                # Shared libraries (governance_commons, replay_protection, validation_utils)
├── tests/               # Integration, e2e, fuzz, and unit test suites
├── docs/                # Architecture, security, API reference, ADRs
├── scripts/             # Deployment, monitoring, and utility scripts
├── config/              # Network and environment configuration
├── schemas/             # Event and interface schemas
├── mobile-sdk/          # Multi-platform mobile SDKs
├── resource-budgets/    # Contract resource budget definitions
├── makefile             # Build automation
├── Cargo.toml           # Workspace configuration
├── clippy.toml          # Clippy lint configuration
└── rustfmt.toml         # Formatting configuration
```

See [README.md](README.md) for the full project overview.

---

## Development Workflow

### 1. Fork and clone

```bash
git clone https://github.com/<your-username>/Uzima-Contracts.git
cd Uzima-Contracts
git remote add upstream https://github.com/Stellar-Uzima/Uzima-Contracts.git
```

### 2. Create a branch

Always branch from the latest `main`:

```bash
git fetch upstream
git checkout -b feature/your-feature upstream/main
```

Branch naming:

| Purpose | Format | Example |
|---------|--------|---------|
| Bug fix | `fix/issue-<number>-<short-desc>` | `fix/issue-42-fix-auth-validation` |
| Feature | `feature/<short-desc>` | `feature/add-batch-export` |
| Docs | `docs/<short-desc>` | `docs/update-deployment-guide` |

### 3. Make changes

- Follow the [Contract Authoring Patterns](#contract-authoring-patterns) below.
- Run quality checks frequently: `make check`.
- Keep changes focused and minimal.

### 4. Test locally

```bash
make test            # all tests
make test-unit       # unit tests only
make test-integration # integration tests only
```

### 5. Commit

Use [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "fix(auth): resolve JWT validation issue"
git commit -m "feat(records): add batch export support"
git commit -m "docs: update contributor guide"
```

### 6. Push and open a PR

```bash
git push origin feature/your-feature
```

Open a pull request against `upstream/main`. See [Pull Request Process](#pull-request-process).

---

## Contract Authoring Patterns

Every contract in this repository follows a consistent set of patterns. When authoring or modifying a contract, match the existing conventions.

### File structure

```
contracts/
└── your_contract/
    ├── src/
    │   ├── lib.rs       # Main contract implementation
    │   ├── errors.rs    # Error enum definition
    │   ├── types.rs     # Type definitions (when non-trivial)
    │   ├── events.rs    # Event helpers (when non-trivial)
    │   ├── storage.rs   # Storage key helpers (when non-trivial)
    │   └── test.rs      # Unit tests
    └── Cargo.toml
```

**Rules:**
- `lib.rs` must contain the `#[contract]` struct and `#[contractimpl]` block.
- Separate `errors.rs` when the error enum has 5+ variants.
- Separate `types.rs` when there are 3+ struct/enum definitions.
- Tests go in `src/test.rs` for unit tests and `tests/` for integration tests.

### Contract boilerplate

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

mod errors;
mod types;

pub use errors::Error;

#[contract]
pub struct YourContract;

#[contractimpl]
impl YourContract {
    /// Initialize the contract. Must be called once after deployment.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Single-initialization guard
        if env.storage().instance().has(&errors::ADMIN_KEY) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&errors::ADMIN_KEY, &admin);
        Ok(())
    }
}
```

### Authorization

Every state-mutating function must call `require_auth()` on the caller **before** any role check:

```rust
pub fn do_something(env: Env, caller: Address) -> Result<(), Error> {
    caller.require_auth();                    // 1. Authenticate
    Self::require_admin(&env, &caller)?;     // 2. Authorize
    // ... perform action
    Ok(())
}
```

Never skip `require_auth()`. See [Security Best Practices](docs/SECURITY_BEST_PRACTICES.md) for the full rationale.

### Initialization guard

Use a fallible guard (not a panic) on re-initialization:

```rust
if env.storage().instance().has(&DataKey::Admin) {
    return Err(Error::AlreadyInitialized);
}
```

Import the `init_guard` from `libs/validation_utils` when starting a new contract to avoid duplicating this pattern.

### Error handling

- Always return `Result<T, Error>` for fallible functions.
- Error variants use `PascalCase` with `#[contracterror]`:
  ```rust
  #[contracterror]
  #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
  #[repr(u32)]
  pub enum Error {
      NotAuthorized = 1,
      InvalidInput = 2,
      AlreadyInitialized = 3,
  }
  ```
- Assign error codes in ranges (see [Contract Naming Conventions](docs/CONTRACT_NAMING_CONVENTIONS.md)).
- Never expose internal implementation details in error messages.

### Storage patterns

- Use `DataKey` enum for storage keys:
  ```rust
  #[derive(Clone)]
  #[contracttype]
  pub enum DataKey {
      Admin,
      User(Address),
      Record(u64),
  }
  ```
- Use `instance().set()` / `instance().get()` for contract-wide singleton data.
- Use `persistent().set()` / `persistent().get()` for per-entity data.
- Use `temporary().set()` for ephemeral data (rate limits, nonces).

### Events

- Emit events for every state-changing operation.
- Use `snake_case` for event topic symbols (see [Event Topic Naming](#event-topic-naming-convention)).
- Publish via `env.events().publish(...)`.

### Arithmetic

- Use `checked_add`, `checked_sub`, `checked_mul` for all arithmetic that can overflow.
- Return `Error::Overflow` (or a domain-specific variant) on overflow.
- Never use `+`, `-`, `*` directly on token amounts or counters.

### Documentation

- Every public function must have a `///` doc comment explaining what it does, what it expects, and what it returns.
- Include `Example` sections for complex functions.
- Module-level `//!` docs at the top of `lib.rs` should describe purpose, dependencies, initialization requirements, role/permission model, error ranges, and a usage example.

---

## Naming Conventions

See [docs/CONTRACT_NAMING_CONVENTIONS.md](docs/CONTRACT_NAMING_CONVENTIONS.md) for the full reference.

| Element | Convention | Example |
|---------|------------|---------|
| Contract directory | `snake_case` | `medical_records` |
| Function | `snake_case` | `add_record`, `get_user` |
| Variable | `snake_case` | `record_id`, `is_confidential` |
| Constant | `SCREAMING_SNAKE_CASE` | `ADMIN`, `MAX_RATE_LIMIT` |
| Struct / Enum | `PascalCase` | `MedicalRecord`, `Role` |
| Enum variant | `PascalCase` | `NotAuthorized`, `RecordCreated` |
| Storage key symbol | `SCREAMING_SNAKE_CASE` | `USER`, `RECORD` |
| Error variant | `PascalCase` | `InvalidInput`, `RecordNotFound` |

---

## Testing Requirements

### What must be tested

| Change type | Required tests |
|-------------|----------------|
| New public function | Unit tests covering normal, edge, and error paths |
| New contract | Unit tests + integration test in `tests/integration/` |
| Bug fix | Regression test proving the fix works |
| Access-control change | Tests for all affected roles (authorized and unauthorized) |
| Arithmetic | Overflow and underflow boundary tests |
| Cross-contract call | Integration test with mock or real target contract |

### Test patterns

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_add_record_success() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract = env.register_contract(None, YourContract);

        env.mock_all_auths();

        let result = YourContract::initialize(&env, &admin.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_record_not_authorized() {
        let env = Env::default();
        let user = Address::generate(&env);
        let contract = env.register_contract(None, YourContract);

        let result = YourContract::do_something(&env, &user.clone());
        assert_eq!(result, Err(Error::NotAuthorized));
    }
}
```

**Key rules:**
- Always use `env.mock_all_auths()` in tests to simulate authorization.
- Use `Address::generate(&env)` for test addresses.
- Test both success and failure paths.
- Run `cargo test --package <contract_name>` after changes.

### Fuzz and property-based tests

For security-sensitive contracts, add fuzz targets under `tests/fuzz/` and property-based tests in `tests/unit/property_based_tests.rs`. See [docs/FUZZING_PROGRAM.md](docs/FUZZING_PROGRAM.md).

---

## Documentation Standards

### When to update docs

| Change | Docs to update |
|--------|----------------|
| New public function | Add doc comment on the function |
| New contract | Create `contracts/<name>/README.md` |
| New event | Update `schemas/events/` and event docs |
| Breaking API change | Update `docs/api.md` and `CHANGELOG.md` |
| New deployment step | Update `docs/DEPLOYMENT_GUIDE.md` |
| New error code | Update `docs/ERROR_CODES.md` |

### Doc comment style

```rust
/// Creates a new medical record for the given patient.
///
/// # Arguments
/// * `caller` - The healthcare provider creating the record. Must have `CreateRecord` permission.
/// * `patient` - The patient who owns the record.
/// * `diagnosis` - A non-empty diagnosis string.
///
/// # Returns
/// The unique record ID on success, or `Error::NotAuthorized` if the caller lacks permission.
///
/// # Example
/// ```rust,ignore
/// let id = client.add_record(&doctor, &patient, &"Flu", ...);
/// ```
pub fn add_record(...) -> Result<u64, Error> { ... }
```

---

## Review Expectations

### Before requesting review

Verify the following before opening or marking a PR ready for review:

- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --all` passes locally
- [ ] No hardcoded secrets or private keys
- [ ] Storage keys use the `DataKey` enum pattern
- [ ] Error types use `#[contracterror]` where applicable
- [ ] Events are emitted for state-changing operations
- [ ] Authorization checks are present on all privileged functions
- [ ] Integer arithmetic uses `checked_*` where overflow is possible
- [ ] PR description references the relevant issue(s)

See [docs/contract-review-checklist.md](docs/contract-review-checklist.md) for the full checklist.

### What reviewers look for

| Category | What reviewers check |
|----------|---------------------|
| Correctness | Logic matches specification; edge cases handled |
| Security | No auth bypasses, no unchecked overflow, no data leaks |
| Testing | Both positive and negative paths are covered |
| Style | Passes fmt + clippy; follows naming conventions |
| Performance | No unnecessary storage reads/writes |
| Documentation | Public APIs are documented; changes reflected in docs |

### Review labels

- `[blocking]` - Must be resolved before merge
- `[nit]` - Optional improvement, not required
- `[question]` - Clarification needed

See [docs/CODE_REVIEW_PROCESS.md](docs/CODE_REVIEW_PROCESS.md) for the full review process including timelines, escalation, and approval requirements.

---

## Pull Request Process

### PR requirements

1. **Title** under 70 characters, clear and descriptive.
2. **Description** must include:
   - Summary of changes
   - Which issue is resolved (e.g., `Closes #42`)
   - Any migration or rollout concerns
3. **One issue per PR** - do not bundle unrelated changes.
4. **Branch from `main`** and keep the branch up to date.
5. **All CI checks pass** before requesting review.

### PR description template

```markdown
## Summary

<Describe what this PR does and why.>

## Changes

- <List specific changes>
- <Note any new files, removed files, or renamed items>

## Testing

- <How was this tested locally?>
- <What test coverage exists?>

## Migration / Rollback

- <Any breaking changes or migration steps?>

Closes #<issue-number>
```

### Merge strategy

| Change type | Required approvals |
|-------------|-------------------|
| Bug fix / docs | 1 maintainer |
| New feature | 2 maintainers |
| Security-sensitive | 2 maintainers + security review |
| Breaking change | 2 maintainers + issue discussion |

---

## Event Topic Naming Convention

All contract event topics **must** use `snake_case` naming. This applies to both the primary topic (first symbol) and any subtopics.

### Do

```rust
env.events().publish((symbol_short!("payment_processed"),), data);
env.events().publish((Symbol::new(&env, "record_created"),), data);
```

### Don't

```rust
env.events().publish((symbol_short!("PaymentProcessed"),), data);  // PascalCase
env.events().publish((symbol_short!("PAYMENT"),), data);           // UPPER_CASE
env.events().publish((Symbol::new(&env, "recordCreated"),), data); // camelCase
```

### Rationale

Consistent `snake_case` event naming ensures that off-chain indexers and monitoring tools can reliably pattern-match event topics without case sensitivity issues.

---

## Security Guidelines

- Follow [Security Best Practices](docs/SECURITY_BEST_PRACTICES.md) for all contract code.
- Review [Threat Models](docs/MASTER_THREAT_MODEL.md) for security-sensitive changes.
- Never commit secrets, keys, or credentials.
- Use `saturating_*` or `checked_*` arithmetic for all numeric operations.
- Validate all external inputs at the contract boundary.
- See [docs/SECURITY_CHECKLIST.md](docs/SECURITY_CHECKLIST.md) for the full security checklist.

---

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/Stellar-Uzima/Uzima-Contracts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Stellar-Uzima/Uzima-Contracts/discussions)
- **Docs**: [docs/](docs/) directory
- **Architecture**: [docs/SYSTEM_ARCHITECTURE.md](docs/SYSTEM_ARCHITECTURE.md)
- **API Reference**: [docs/api.md](docs/api.md)

---

*Last updated: 2026-07-24*
