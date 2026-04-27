# Quality Metrics Dashboard

This document describes the automated quality metrics tracked across all Uzima contracts and how to access them.

---

## Metrics Tracked in CI

Every CI run produces a quality metrics summary in the GitHub Actions job summary. The `quality-metrics` job in `.github/workflows/ci.yml` aggregates results from all CI jobs.

| Metric | Source | Threshold |
|--------|--------|-----------|
| Lint & Format | `cargo fmt --check` + `cargo clippy` | Zero warnings/errors |
| Test pass rate | `cargo test --all` | 100% pass |
| Code coverage | `cargo-tarpaulin` | ≥ 70% (enforced) |
| Node.js API tests | `npm test` in `integrations/cloud_health_api` | 100% pass |
| WASM build | `cargo build --target wasm32-unknown-unknown` | Must succeed |
| Security scan | `cargo-audit`, `cargo-geiger`, Gitleaks | Zero critical findings |
| Testing pyramid | `npm run test:pyramid:report` | Pyramid ratios met |
| Event schema validation | `npm run events:validate` | All schemas valid |

---

## Accessing Reports

### GitHub Actions (per-run)

1. Go to the **Actions** tab on GitHub
2. Select a workflow run
3. Scroll to **Artifacts** at the bottom of the run summary
4. Download any of:
   - `coverage-report` – HTML coverage report from tarpaulin
   - `quality-metrics-dashboard` – Markdown dashboard for the run
   - `testing-pyramid-report` – JSON + Markdown pyramid report
   - `security-reports` – Security scan results

### Job Summary

Each CI run writes a quality dashboard directly to the GitHub Actions job summary, visible inline on the run page without downloading artifacts.

---

## Coverage Trends

Coverage is enforced at **≥ 70%** overall via `cargo-tarpaulin --fail-under 70` in the `coverage` CI job. The threshold is defined in `.github/workflows/ci.yml`.

To run coverage locally:

```bash
# Install tarpaulin (once)
cargo install cargo-tarpaulin

# Run with HTML output
cargo tarpaulin --out Html --output-dir coverage/ --timeout 600 \
  --exclude-files "tests/*" --ignore-panics --ignore-timeouts

# Open report
open coverage/tarpaulin-report.html
```

Or use the project script:

```bash
./scripts/coverage_report.sh
```

---

## Code Complexity

Clippy enforces complexity limits as part of the `lint` CI job:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

To check locally:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Security Scan Results

Security metrics are produced by the `security` CI job (runs on PRs and pushes to `main`) and the weekly scan in `weekly-security-report.yml`.

Tools used:
- `cargo-audit` – known CVEs in dependencies
- `cargo-geiger` – unsafe code usage
- Gitleaks – secret detection

Weekly reports are uploaded as artifacts and retained for 30 days.

---

## Documentation Completeness

All public contract functions should have doc comments. Clippy's `missing_docs` lint is available but not yet enforced globally. Track documentation coverage manually via:

```bash
cargo doc --no-deps --document-private-items 2>&1 | grep "warning: missing documentation"
```

---

## Issue and Bug Trends

Track open issues by label on GitHub:
- [`bug`](https://github.com/Stellar-Uzima/Uzima-Contracts/labels/bug)
- [`security`](https://github.com/Stellar-Uzima/Uzima-Contracts/labels/security)
- [`quality-metrics`](https://github.com/Stellar-Uzima/Uzima-Contracts/labels/quality-metrics)

---

## Related

- [Testing Best Practices](testing/TESTING_BEST_PRACTICES.md)
- [Testing Pyramid](testing/TESTING_PYRAMID.md)
- [Static Analysis Integration](STATIC_ANALYSIS.md)
- [Security Controls Mapping](SECURITY_CONTROLS_MAPPING.md)
