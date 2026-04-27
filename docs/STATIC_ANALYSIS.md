# Static Analysis Tool Integration

This document describes the static analysis tools integrated into the Uzima Contracts CI/CD pipeline and how to use them locally.

---

## Tools

### 1. Clippy (Rust Linting)

Clippy is the primary static analysis tool for Rust code. It is enforced in the `lint` CI job on every push and pull request.

**CI command:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

All Clippy warnings are treated as errors (`-D warnings`). PRs cannot merge if Clippy reports any issues.

**Run locally:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Fix automatically where possible:**
```bash
cargo clippy --fix --all-targets --all-features
```

### 2. rustfmt (Formatting)

Code formatting is enforced in the `lint` CI job.

**CI command:**
```bash
cargo fmt --all -- --check
```

**Fix locally:**
```bash
cargo fmt --all
```

### 3. cargo-audit (Dependency Vulnerability Scanning)

Scans `Cargo.lock` for dependencies with known CVEs.

**CI:** Runs in the `security` job (PRs + main branch) and weekly via `weekly-security-report.yml`.

**Run locally:**
```bash
cargo install cargo-audit
cargo audit
```

### 4. cargo-geiger (Unsafe Code Detection)

Counts and reports `unsafe` code usage across the workspace and all dependencies.

**CI:** Runs in the `security` job via `scripts/security-scan.sh`.

**Run locally:**
```bash
cargo install cargo-geiger
cargo geiger
```

### 5. Gitleaks (Secret Detection)

Scans git history and working tree for accidentally committed secrets (API keys, private keys, tokens).

**CI:** Runs in the `security` job. On PRs, scans only the PR commit range. On main branch pushes, scans the full repository.

**Run locally:**
```bash
# Install
brew install gitleaks  # macOS
# or download from https://github.com/gitleaks/gitleaks/releases

# Scan working directory
gitleaks detect --source . --redact
```

---

## CI Integration Summary

| Tool | CI Job | Trigger | Failure Behavior |
|------|--------|---------|-----------------|
| Clippy | `lint` | Every push/PR | Blocks merge |
| rustfmt | `lint` | Every push/PR | Blocks merge |
| cargo-audit | `security` | PRs + main | Blocks merge |
| cargo-geiger | `security` | PRs + main | Reports only |
| Gitleaks | `security` | PRs + main | Blocks merge |

---

## Baseline Issues

Run the following to establish a baseline of existing issues before enforcing new rules:

```bash
# Clippy baseline
cargo clippy --all-targets --all-features 2>&1 | grep "^error\|^warning" | wc -l

# Audit baseline
cargo audit 2>&1 | tail -5

# Unsafe code baseline
cargo geiger 2>&1 | grep "Total"
```

---

## Adding New Lint Rules

To add project-wide Clippy lints, edit the workspace `Cargo.toml`:

```toml
[workspace.lints.clippy]
# Example: enforce explicit returns
explicit_returns = "warn"
```

Or add to individual contract `lib.rs` files:

```rust
#![warn(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
```

---

## Formal Verification (Future)

Formal verification tooling (e.g., Kani, Prusti) is tracked as a future enhancement. See `docs/DECISION_PROCEDURES.md` for the current decision framework.

---

## Related

- [Quality Metrics Dashboard](QUALITY_METRICS_DASHBOARD.md)
- [Security Controls Mapping](SECURITY_CONTROLS_MAPPING.md)
- [Security Incident Response](SECURITY_INCIDENT_RESPONSE.md)
- [Weekly Security Report workflow](../.github/workflows/weekly-security-report.yml)
