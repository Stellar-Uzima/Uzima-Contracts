# Lint and Formatting Policy

This document defines the repository-wide linting, formatting, and code quality policy for all Rust and JavaScript assets in the Uzima Contracts project.

## Overview

Every pull request must pass all formatting, linting, and quality checks before merge. This policy defines what those checks are, how they are configured, and how to run them locally.

---

## Rust Formatting

### Configuration

- **File**: `rustfmt.toml`
- **Tool**: `cargo fmt` (stable rustfmt)
- **Indent**: 4 spaces (no tabs)
- **Max width**: 100 characters
- **Import grouping**: `StdExternalCrate` (std → external crates → local crates)
- **Import sorting**: enabled

### How to run

```bash
cargo fmt --all              # auto-format
cargo fmt --all -- --check   # check only (CI mode)
```

### Enforcement

- Pre-commit hook: `cargo fmt --all -- --check`
- CI: `cargo fmt --all -- --check` (fails on any formatting diff)

---

## Rust Linting (Clippy)

### Configuration

- **File**: `clippy.toml`
- **Tool**: `cargo clippy`
- **Lint groups enabled**: `clippy::pedantic` + `clippy::nursery`
- **Default**: `-D warnings` (all warnings are errors)

### Thresholds

| Setting | Value |
|---------|-------|
| `too-many-arguments` | 6 |
| `cognitive-complexity` | 25 |
| `too-many-lines` | 100 |
| `enum-variant-size` | 256 bytes |

### Allowed exceptions

The full list of allowlisted `pedantic`/`nursery` lints is in `clippy.toml` and duplicated in `docs/CODING_STANDARDS.md`. Each exception includes a rationale. Examples:

- `clippy::missing_errors_doc` — not all public functions need error docs
- `clippy::must_use_candidate` — overly aggressive for this codebase
- `clippy::wildcard_imports` — used intentionally in test modules

### How to run

```bash
cargo clippy --workspace --all-targets -- -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -A clippy::bool_to_int_with_if \
    ... (see CODING_STANDARDS.md for full list)
```

Or simply:

```bash
make lint
```

### Enforcement

- Pre-commit hook: `cargo clippy --all-targets -- -D warnings`
- CI: full pedantic + nursery command (see `.github/workflows/ci.yml`)

---

## JavaScript / Node.js

### Format and Lint

There is no dedicated JavaScript linter (ESLint/Prettier) configured in this repository. JavaScript assets are validated through:

1. **JSON Schema validation** — `ajv` validates event schemas and interface registry
2. **Script-level checks** — `--check` modes in `scripts/abi-compat.mjs`, `scripts/generate-sdk-types.mjs`, etc.
3. **Package.json validation** — `npm install` validates JSON syntax

### How to run

```bash
npm run abi:check         # ABI compatibility check
npm run sdk:check         # SDK bindings drift check
npm run api-docs:check    # API docs staleness check
npm run interface:check   # Interface registry validation
```

### Enforcement

- CI: each `--check` script fails if output is stale
- No pre-commit hooks for JavaScript (yet)

---

## Shell Scripts

### Configuration

- **Tool**: `shellcheck`
- **Shell dialect**: bash (`-s bash`)
- **Flags**: `-x` (follow sources)

### How to run

```bash
make shellcheck           # via Makefile
shellcheck scripts/*.sh   # directly
```

### Enforcement

- Pre-commit hook: `shellcheck -s bash -x`
- CI: `make shellcheck`

---

## Secret Scanning

### Configuration

- **Tool**: TruffleHog v3.63.0
- **File**: `.trufflehog.yaml`
- **Behavior**: scans all staged files for secrets; fails on detection

### Enforcement

- Pre-commit hook: `trufflehog --fail`
- CI: dedicated security workflow

---

## Spell Checking

### Configuration

- **Tool**: `typos` (crate-ci/typos v1.30.1)
- **Behavior**: checks for common typos in source code

### Enforcement

- Pre-commit hook: `typos`

---

## Summary of Checks

| Check | Tool | Local command | CI |
|-------|------|---------------|-----|
| Rust formatting | `cargo fmt` | `make fmt` | Yes |
| Rust linting | `cargo clippy` | `make lint` | Yes |
| Rust tests | `cargo test` | `make test` | Yes |
| Shell linting | `shellcheck` | `make shellcheck` | Yes |
| Secret scanning | `trufflehog` | pre-commit hook | Yes |
| Spell checking | `typos` | pre-commit hook | Yes |
| ABI compatibility | `abi-compat.mjs` | `npm run abi:check` | Yes |
| SDK drift | `generate-sdk-types.mjs` | `npm run sdk:check` | Yes |
| Interface validation | `validate-interfaces.mjs` | `npm run interface:check` | Yes |

### Pre-commit setup

```bash
pip install pre-commit    # if not installed
pre-commit install        # enable hooks
```

### Running all checks locally

```bash
make check                # fmt + lint + test + shellcheck
```

---

## Adding New Lint Rules

1. Add the rule to the appropriate config file (`rustfmt.toml`, `clippy.toml`)
2. Document the rationale in `clippy.toml` comments
3. Update `docs/CODING_STANDARDS.md` if it's a clippy allowlist change
4. Update this document if it's a new category of check
5. Add the check to CI and pre-commit hooks

---

## Related Documentation

- [Coding Standards](CODING_STANDARDS.md) — naming conventions and code style
- [Code Review Process](CODE_REVIEW_PROCESS.md) — what reviewers check
- [Contract Naming Conventions](CONTRACT_NAMING_CONVENTIONS.md) — naming rules
- [Security Best Practices](SECURITY_BEST_PRACTICES.md) — security lint patterns

---

*Last updated: 2026-07-24*
