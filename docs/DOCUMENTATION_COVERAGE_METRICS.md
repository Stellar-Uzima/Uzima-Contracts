# Contract Documentation Coverage Metrics

This document describes how documentation coverage is measured, enforced, and reported for Uzima Contracts.

---

## Metrics Tracked

| Metric | Description | Target |
|---|---|---|
| Function documentation % | Public `fn` items with doc comments | ≥ 90% |
| Type documentation % | Public `struct`, `enum`, `type` with doc comments | ≥ 80% |
| Module documentation % | Modules with a module-level `//!` comment | ≥ 70% |
| Example code coverage | Public functions with `# Examples` in doc comments | ≥ 30% |
| API documentation completeness | All parameters and return values described | ≥ 80% |

---

## Automated Measurement

### Using `cargo doc` warnings

Enable missing-docs lint to catch undocumented public items at compile time:

```rust
// Add to each contract's lib.rs
#![warn(missing_docs)]
```

Run to see all missing documentation warnings:

```bash
cargo doc --all --no-deps 2>&1 | grep "warning: missing documentation"
```

### Coverage script

The `scripts/coverage_report.sh` script generates a documentation coverage report:

```bash
# Generate doc coverage report
./scripts/coverage_report.sh --docs

# Output: reports/doc_coverage.txt
# Format: contract_name: X/Y public items documented (Z%)
```

To add doc coverage to the existing script, append:

```bash
echo "=== Documentation Coverage ===" >> "$REPORT_FILE"
for contract_dir in contracts/*/src/lib.rs; do
  contract=$(basename "$(dirname "$(dirname "$contract_dir")")")
  total=$(grep -c "pub fn\|pub struct\|pub enum\|pub type" "$contract_dir" 2>/dev/null || echo 0)
  documented=$(grep -c "^\s*///" "$contract_dir" 2>/dev/null || echo 0)
  if [ "$total" -gt 0 ]; then
    pct=$(( documented * 100 / total ))
    echo "$contract: $documented/$total items with doc comments (~$pct%)" >> "$REPORT_FILE"
  fi
done
```

---

## CI/CD Integration

Add the following step to `.github/workflows/ci.yml` to enforce documentation coverage on every PR:

```yaml
- name: Check documentation coverage
  run: |
    # Warn on missing docs for all public items
    RUSTDOCFLAGS="-D missing_docs" cargo doc --all --no-deps 2>&1 | tee reports/doc_coverage.txt
    # Fail if more than 20 missing-doc warnings (gradual enforcement)
    missing=$(grep -c "warning: missing documentation" reports/doc_coverage.txt || true)
    echo "Missing doc warnings: $missing"
    if [ "$missing" -gt 20 ]; then
      echo "❌ Too many missing documentation warnings ($missing > 20)"
      exit 1
    fi

- name: Upload doc coverage report
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: doc-coverage-report
    path: reports/doc_coverage.txt
```

---

## Dashboard Reporting

A documentation coverage summary is posted as a PR comment when coverage drops below threshold:

```yaml
- name: Comment doc coverage
  if: github.event_name == 'pull_request'
  uses: actions/github-script@v7
  with:
    script: |
      const fs = require('fs');
      if (!fs.existsSync('reports/doc_coverage.txt')) return;
      const report = fs.readFileSync('reports/doc_coverage.txt', 'utf8');
      const missing = (report.match(/warning: missing documentation/g) || []).length;
      const emoji = missing === 0 ? '✅' : missing < 10 ? '⚠️' : '❌';
      github.rest.issues.createComment({
        issue_number: context.issue.number,
        owner: context.repo.owner,
        repo: context.repo.repo,
        body: `### ${emoji} Documentation Coverage\n\nMissing doc warnings: **${missing}**\n\nSee the \`doc-coverage-report\` artifact for details.`
      });
```

---

## Writing Good Documentation

### Function documentation template

```rust
/// Brief one-line description of what the function does.
///
/// Longer description if needed. Explain the purpose, not just the mechanics.
///
/// # Arguments
///
/// * `env` - The Soroban environment.
/// * `caller` - Address of the caller; must be the contract admin.
/// * `value` - The new threshold value in basis points (1–9999).
///
/// # Errors
///
/// Returns [`Error::NotAuthorized`] if `caller` is not the admin.
/// Returns [`Error::InvalidThreshold`] if `value` is out of range.
///
/// # Examples
///
/// ```ignore
/// client.update_threshold(&admin, &model_id, &5000u32);
/// ```
pub fn update_threshold(env: Env, caller: Address, value: u32) -> Result<(), Error> {
    // ...
}
```

### Module documentation template

```rust
//! # Module Name
//!
//! Brief description of what this module provides.
//!
//! ## Overview
//!
//! Longer description of the module's responsibilities and how it fits
//! into the broader contract architecture.
```

---

## Implementation Status (issue #824)

This section tracks what is actually live in the repository as the result of
issue #824.

### What's enabled

| Capability | State | Where |
|---|---|---|
| `scripts/coverage_report.sh docs` subcommand | ✅ live | `scripts/coverage_report.sh` |
| `RUSTDOCFLAGS="-W missing_docs"` during doc | ✅ live | script step |
| Per-contract public-API heuristic report | ✅ live | `reports/doc_coverage.txt` |
| CI job that runs the script | ✅ live | `.github/workflows/ci.yml` (`doc-coverage`) |
| Artifact upload (`doc-coverage-report`) | ✅ live | workflow step |
| PR comment summarising missing-docs count | ✅ live | workflow step (PRs only) |
| Top-10 most-used contracts focus list | ✅ live | `reports/doc_coverage.txt` (script) |
| ≥80% public-API docs in Top-10 contracts | 🔄 partial | this PR adds module-level docs and key fn examples; per-fn docs are tracked in subsequent PRs |
| `# Examples` snippets on key public functions | 🔄 added where missing | examples added to `cross_chain_bridge` functions in this PR |

### Gradual enforcement rollout (matches AC "gradual enforcement")

| Phase | Status | Gate | Action |
|---|---|---|---|
| 1 | ✅ shipped (this PR) | report-only (`continue-on-error: true`, `DOC_WARN_LIMIT=500`) | run, report, comment, artifact |
| 2 | upcoming | drop `continue-on-error`, fail at >500 | once steady state under 500 |
| 3 | upcoming | tighten gate: 500 → 200 → 50 | per-quarter milestone |
| 4 | target | hard fail at >50 warnings (then >0) | final acceptance state |

The script honours `DOC_WARN_LIMIT` from the environment; the workflow
sets it explicitly for the Phase-1 value (500). Local users can override
it to validate different thresholds:

```bash
DOC_WARN_LIMIT=50 ./scripts/coverage_report.sh docs
```

### What `cargo doc` counts

The script runs:

```bash
RUSTDOCFLAGS="-W missing_docs" cargo doc --workspace --no-deps
```

`--workspace` covers every crate listed in the root `Cargo.toml`
(includes the `libs/` member crates and excludes the contracts listed in
`Cargo.toml`'s `exclude` array). `--no-deps` skips external dependencies.

`missing_docs` is `allow` by default; we promote it to `warn` via
`RUSTDOCFLAGS` so that we can count gaps without flipping every
`lib.rs` to `#![warn(missing_docs)]` (which would risk breaking
`cargo clippy --all-targets -- -D warnings`).

### Top-10 most-used contracts (AC: ≥80% function documentation)

The script emits a top-10 section in `reports/doc_coverage.txt`:

1. `identity_registry`
2. `access_control`
3. `escrow`
4. `governor`
5. `audit`
6. `common_error`
7. `cross_chain_bridge`
8. `credential_registry`
9. `meta_tx_forwarder`
10. `healthcare_payment`

This list is derived from cross-references in `tests/`, integration-suite
imports, and shared deployment scripts. Treat it as a focus list for
documentation efforts; it is not an exhaustive specification of
"most-used".

### Coexisting with the existing metrics

The previous per-script and per-CI shape is preserved:

* Default `./scripts/coverage_report.sh` still runs the test-coverage
  pipeline via `cargo-tarpaulin` (unchanged).
* `./scripts/coverage_report.sh docs` adds documentation coverage.
* `./scripts/coverage_report.sh <test-mode> docs` runs *both* in one go.
* The CI `test`, `build`, and `code-quality` jobs are unchanged.
* The new `doc-coverage` job is additive and does not block PRs in
  Phase 1.

---

## Current Coverage Status

Run the following to get the current state (matches what CI sees):

```bash
RUSTDOCFLAGS="-W missing_docs" cargo doc --workspace --no-deps 2>&1 \
  | grep "warning: missing documentation" | wc -l
```

For a per-contract breakdown plus the Top-10 status, use the script:

```bash
./scripts/coverage_report.sh docs
# Reports:
#   reports/cargo_doc.log
#   reports/doc_coverage.txt
```

Target: reduce missing-doc warnings to **0** across all contracts by the
end of the Phase-4 milestone (see rollout table above).
