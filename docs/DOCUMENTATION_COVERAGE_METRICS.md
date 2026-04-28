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

## Current Coverage Status

Run the following to get the current state:

```bash
cargo doc --all --no-deps 2>&1 | grep "warning: missing documentation" | wc -l
```

Target: reduce missing-doc warnings to **0** across all contracts by end of next sprint.
