# Contributing to Uzima Contracts

Thank you for your interest in contributing to Uzima Contracts! This document provides guidelines and standards for contributing to the project.

## Development Workflow

1. **Fork the repository** and create a feature branch
2. **Follow coding standards** outlined in [CODING_STANDARDS.md](./docs/CODING_STANDARDS.md)
3. **Write tests** for new functionality
4. **Review the contract review checklist** in [docs/contract-review-checklist.md](./docs/contract-review-checklist.md)
5. **Run linting and tests** before submitting
6. **Submit a pull request** with a clear description

## Code Quality Standards

### Naming Conventions
All code must follow the naming conventions defined in [CODING_STANDARDS.md](./docs/CODING_STANDARDS.md) and [CONTRACT_NAMING_CONVENTIONS.md](./docs/CONTRACT_NAMING_CONVENTIONS.md):

- **Functions**: `snake_case`
- **Types**: `PascalCase`  
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`
- **Error enums**: Always use `Error`, never `Err`
- **File names**: `snake_case` (e.g., `lib.rs`, `errors.rs`, `types.rs`, `events.rs`, `test.rs`)
- **Contract directories**: `snake_case`

Run the naming check script before submitting PRs:
```bash
bash scripts/check-naming.sh
```

### Code Style
- Use Rust 2021 edition
- Follow Rustfmt formatting (run `cargo fmt`)
- Adhere to Clippy linting rules (run `cargo clippy`)

### Documentation
- Document all public APIs with `///` doc comments
- Include examples for complex functions
- Update relevant documentation when changing functionality

## Testing Requirements

### Unit Tests
- Write tests for all new functionality
- Test edge cases and error conditions
- Mock external dependencies where appropriate

### Integration Tests
- Test contract interactions
- Verify cross-contract calls work correctly
- Ensure upgrade paths are tested

## Pull Request Process

1. **Ensure code compiles** without warnings
2. **Run all tests** and verify they pass
3. **Update documentation** if needed
4. **Describe changes** in the PR description
5. **Link related issues** if applicable

### PR Review Checklist
- [ ] Code follows naming conventions
- [ ] Tests are included and pass
- [ ] Documentation is updated
- [ ] No new Clippy warnings
- [ ] Code is properly formatted
- [ ] Contract review checklist items have been considered for correctness, safety, and testing

## Contract Review Checklist
Use the shared contract review checklist for all smart contract and contract-related pull requests: [docs/contract-review-checklist.md](./docs/contract-review-checklist.md)

## Development Setup

### Prerequisites
- Rust toolchain (stable)
- Soroban CLI
- Cargo make (optional)

### Local Development
```bash
# Clone the repository
git clone <repository-url>
cd Uzima-Contracts

# Install dependencies
cargo build

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### CI/CD Pipeline
The project uses GitHub Actions for CI/CD. The pipeline includes:
- Build verification
- Linting (Clippy)
- Testing
- Formatting check

## Getting Help
- Review existing documentation in the `docs/` directory
- Check open issues for known problems
- Ask questions in pull request discussions

## Adding a New Contract

Step-by-step guide for adding a new Soroban smart contract to this workspace.

### 1. Create the contract directory

```
contracts/<your_contract_name>/
├── Cargo.toml
└── src/
    ├── lib.rs          # Contract entrypoint (must start with #![no_std])
    ├── errors.rs       # Error enum (recommended)
    └── test.rs         # Unit tests (recommended)
```

### 2. Cargo.toml

Use workspace inheritance for shared metadata:

```toml
[package]
name = "your_contract_name"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
soroban-sdk.workspace = true
# Add shared libs only if needed:
# governance_commons = { workspace = true }
# replay_protection = { workspace = true }
# common_error = { path = "../common_error" }

[dev-dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }

[features]
default = []
testutils = ["soroban-sdk/testutils"]
```

Key points:
- `crate-type = ["cdylib"]` is required for Soroban WASM contracts.
- `soroban-sdk = { workspace = true }` pulls from `[workspace.dependencies]`.
- Dev-dependencies add `testutils` for tests.
- Never pin your own soroban-sdk version; always use `workspace = true`.

### 3. lib.rs skeleton

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    // ...
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Follow the repo-wide range convention:
    //   200–299: Input Validation
    //   300–399: Lifecycle & State
    //   400–499: Entity Existence
    //   500–599: Financial & Resource
    ExampleError = 300,
}

#[contract]
pub struct YourContract;

#[contractimpl]
impl YourContract {
    // pub fn your_method(env: &Env, ...) -> Result<_, Error> { ... }
}
```

Required:
- First line must be `#![no_std]` — CI enforces this for all workspace members.
- Use `#[contracterror]` + `#[repr(u32)]` on your error enum.
- Use `#[contracttype]` on all storage/data types.

### 4. Error conventions

- Range your error codes per the repo convention (see `contracts/governor/src/errors.rs` for a reference).
- Implement `core::fmt::Display` for your error enum with human-readable variant messages.
- Use the `common_error` crate (`contracts/common_error`) for shared error types if your contract needs the same validation/lifecycle errors as other contracts.
- Never use `unwrap()` or `expect()` in contract code. CI runs `scripts/check_no_production_unwraps.sh`.

### 5. Tests

Place tests in `src/test.rs` (included via `#[cfg(test)] mod test;` in lib.rs) or inline:

```rust
use super::*;
use soroban_sdk::{Env, String};

#[test]
fn test_basic_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, YourContract);
    let client = YourContractClient::new(&env, &contract_id);

    // test your methods here
    assert_eq!(client.try_your_method(), Ok(()));
}
```

Run tests with:
```bash
cargo test --all
```

### 6. Workspace registration

Contracts under `contracts/` are **automatically** included by the workspace glob `members = ["contracts/*", ...]` in the root `Cargo.toml`. You do NOT need to add your contract to the members list.

If your contract cannot yet compile against the pinned `soroban-sdk = "=21.7.7"`, add it to the `[workspace] exclude` list in the root `Cargo.toml` with a comment referencing the tracking issue. This lets the rest of CI pass while you fix compilation.

### 7. Shared libraries

If your contract needs reusable types, errors, or utilities shared across contracts, add a library under `libs/`:

```
libs/<your_lib>/
├── Cargo.toml       # crate-type = ["rlib"]
└── src/
    └── lib.rs
```

Reference it from your contract's `Cargo.toml`:
```toml
[dependencies]
your_lib = { path = "../your_lib" }
# or if added to workspace deps:
# your_lib = { workspace = true }
```

Add the lib to the root workspace `members` list if it is not already covered by a glob.

### 8. CI checks your PR must pass

| Check | What it runs | Blocking? |
|---|---|---|
| **Tests** | `cargo test --all` + `scripts/check_error_codes.sh` + `scripts/check_no_production_unwraps.sh` | Yes |
| **Build (wasm32)** | `cargo build --workspace --target wasm32-unknown-unknown --release` + `#![no_std]` check | Yes |
| **Code Quality** | `cargo fmt --all -- --check` (non-blocking) + `cargo clippy --all-targets -- -D warnings` | Clippy yes, fmt non-blocking |
| **Documentation Coverage** | `cargo check --workspace` + `scripts/coverage_report.sh docs` | Non-blocking (Phase 1) |
| **SDK Bindings** | `node scripts/generate-sdk-types.mjs` — regenerates TypeScript/Python bindings | Yes (unless `[skip-sdk-gen]` in PR body) |

#### Running CI checks locally

```bash
# Full test suite
cargo test --all

# WASM build
cargo build --workspace --target wasm32-unknown-unknown --release

# no_std check (run from repo root)
for contract in contracts/*/; do
  name=$(basename "$contract")
  lib_file="${contract}src/lib.rs"
  if [ -f "$lib_file" ]; then
    grep -qF '#![no_std]' "$lib_file" || echo "MISSING #![no_std]: $lib_file"
  fi
done

# Error code check
bash ./scripts/check_error_codes.sh

# No-production-unwraps check
bash ./scripts/check_no_production_unwraps.sh

# Clippy
cargo clippy --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Doc coverage
./scripts/coverage_report.sh docs

# SDK bindings (if your contract has public entrypoints)
node scripts/generate-sdk-types.mjs
```

### 9. Documentation

- Add doc comments (`///`) to all public types, functions, and entrypoints.
- CI tracks missing-docs warnings via `scripts/coverage_report.sh docs`.
- If your contract introduces new domain terms, add them to `docs/GLOSSARY.md`.
- If your contract emits events, document them in `docs/EVENTS.md`.

### 10. Checklist before opening a PR

- [ ] `contracts/<name>/src/lib.rs` starts with `#![no_std]`
- [ ] `Cargo.toml` uses `crate-type = ["cdylib"]` and `soroban-sdk = { workspace = true }`
- [ ] Error enum uses `#[contracterror]`, `#[repr(u32)]`, `#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]`
- [ ] Error codes follow the repo range convention (200–299 validation, 300–399 lifecycle, etc.)
- [ ] `core::fmt::Display` implemented for error enum
- [ ] No `unwrap()` or `expect()` in contract code (check with `bash scripts/check_no_production_unwraps.sh`)
- [ ] Tests exist and pass: `cargo test --all`
- [ ] WASM build succeeds: `cargo build --workspace --target wasm32-unknown-unknown --release`
- [ ] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] Doc comments on public API items
- [ ] Contract is NOT in the `exclude` list, OR a comment explains why with a tracking issue number
- [ ] Run `node scripts/generate-sdk-types.mjs` if your contract adds new public entrypoints

## License
By contributing, you agree that your contributions will be licensed under the project's MIT license.