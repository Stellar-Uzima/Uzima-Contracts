# Implementation Guide: Issues #397, #418, #420, #422

**Assignee:** @Xhristin3  
**PR Strategy:** All 4 issues are addressed in a single PR from a feature branch.

---

## Overview

| Issue | Title | Labels |
|-------|-------|--------|
| [#397](https://github.com/Stellar-Uzima/Uzima-Contracts/issues/397) | Implement upgrade testing framework | testing, upgradeability, infrastructure |
| [#418](https://github.com/Stellar-Uzima/Uzima-Contracts/issues/418) | Implement comprehensive API versioning strategy | api-design, enhancement, upgradeability |
| [#420](https://github.com/Stellar-Uzima/Uzima-Contracts/issues/420) | Implement proper test coverage reporting | ci-cd, testing, quality-metrics |
| [#422](https://github.com/Stellar-Uzima/Uzima-Contracts/issues/422) | Implement comprehensive disaster recovery procedures | disaster-recovery, enhancement, operations |

---

## Branch & PR Setup

```bash
# Fork already cloned — create a single feature branch
git checkout -b feat/xhristin3-upgrade-coverage-dr

# After all changes are committed, push and open one PR
git push -u origin feat/xhristin3-upgrade-coverage-dr
```

PR title: `feat: upgrade testing, API versioning, coverage reporting, and DR procedures`

PR description must reference all four issues so GitHub auto-closes them on merge:

```
Closes #397
Closes #418
Closes #420
Closes #422
```

---

## Issue #397 — Implement Upgrade Testing Framework

**Problem:** No systematic testing for contract upgrades and migrations.

### What to add

Create `tests/upgrade/upgrade_framework.rs` with a structured test suite that:

1. Deploys contract v1 with seed data
2. Upgrades to v2
3. Verifies state integrity post-upgrade
4. Tests new functionality
5. Verifies old functionality still works
6. Tests rollback capability

```rust
// tests/upgrade/upgrade_framework.rs
#[cfg(test)]
mod upgrade_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    /// Step 1 – deploy v1 and populate state
    #[test]
    fn test_state_preserved_after_upgrade() {
        let env = Env::default();
        // deploy v1 wasm, call init, write test records
        // upgrade to v2 wasm via env.deployer().upload_contract_wasm(...)
        // assert all v1 state is still readable
    }

    /// Step 2 – backward compatibility
    #[test]
    fn test_backward_compatibility() {
        let env = Env::default();
        // call v1 entry-points against the upgraded contract
        // assert they still return the same results
    }

    /// Step 3 – rollback on failure
    #[test]
    fn test_rollback_on_failed_upgrade() {
        let env = Env::default();
        // attempt upgrade with a deliberately broken wasm
        // assert the contract is still at v1 and state is intact
    }
}
```

### CI integration

Add a job to `.github/workflows/ci.yml`:

```yaml
upgrade-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown
    - run: cargo test --test upgrade_framework -- --nocapture
```

### Files to create / modify

| Action | Path |
|--------|------|
| Create | `tests/upgrade/upgrade_framework.rs` |
| Create | `tests/upgrade/mod.rs` |
| Modify | `.github/workflows/ci.yml` — add `upgrade-tests` job |

---

## Issue #418 — Implement Comprehensive API Versioning Strategy

**Problem:** No versioning strategy for contract APIs, making upgrades difficult.

### What to add

#### 1. Version constant in each contract

Add to `contracts/medical_records/src/lib.rs` (and other contracts as needed):

```rust
pub const API_VERSION: u32 = 1;
pub const MIN_COMPATIBLE_VERSION: u32 = 1;

pub fn get_api_version(env: Env) -> u32 {
    API_VERSION
}

pub fn is_compatible(env: Env, client_version: u32) -> bool {
    client_version >= MIN_COMPATIBLE_VERSION && client_version <= API_VERSION
}
```

#### 2. Version stored in contract storage

```rust
const VERSION_KEY: Symbol = symbol_short!("API_VER");

pub fn initialize(env: Env) {
    env.storage().instance().set(&VERSION_KEY, &API_VERSION);
}
```

#### 3. Deprecation documentation

Create `docs/API_VERSIONING.md`:

```markdown
# API Versioning Strategy

## Scheme
Semantic versioning: MAJOR.MINOR.PATCH stored as a single u32 (e.g., 1 = v1.0.0).

See [Contract API Stability Guarantees](./api.md) for repository-wide stability levels, deprecation timelines, and compatibility policies.

## Compatibility guarantees
- Minor/patch bumps: fully backward compatible.
- Major bumps: compatibility layer provided for one full release cycle.

## Deprecation timeline
1. Mark function `#[deprecated]` with a doc comment pointing to the replacement.
2. Keep deprecated function for at least one major version.
3. Remove in the following major version.

## Migration path
- Clients call `get_api_version()` before invoking contract functions.
- If `is_compatible(client_version)` returns false, clients must upgrade.
```

### Files to create / modify

| Action | Path |
|--------|------|
| Modify | `contracts/medical_records/src/lib.rs` — add version constants and functions |
| Create | `docs/API_VERSIONING.md` |

---

## Issue #420 — Implement Proper Test Coverage Reporting

**Problem:** No automated test coverage reporting in the CI pipeline.

### What to add

#### 1. Install `cargo-tarpaulin` in CI

Modify `.github/workflows/ci.yml` to add a coverage job:

```yaml
coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Install cargo-tarpaulin
      run: cargo install cargo-tarpaulin --locked --version 0.27.3
    - name: Run coverage
      run: |
        cargo tarpaulin \
          --out Xml Html \
          --output-dir reports/coverage \
          --exclude-files "tests/*" \
          --fail-under 80
    - name: Upload coverage report
      uses: actions/upload-artifact@v4
      with:
        name: coverage-report
        path: reports/coverage/
    - name: Post coverage to PR
      uses: codecov/codecov-action@v4
      with:
        files: reports/coverage/cobertura.xml
        fail_ci_if_error: true
```

#### 2. Coverage thresholds

Add `tarpaulin.toml` at the repo root:

```toml
[default]
exclude-files = ["tests/*", "*/target/*"]
fail-under = 80
out = ["Xml", "Html"]
output-dir = "reports/coverage"
```

#### 3. `.gitignore` update

```
reports/coverage/
```

### Thresholds

| Metric | Minimum |
|--------|---------|
| Line coverage | 80% |
| Branch coverage | 70% |
| Alert on decrease | Yes (Codecov PR comment) |

### Files to create / modify

| Action | Path |
|--------|------|
| Create | `tarpaulin.toml` |
| Modify | `.github/workflows/ci.yml` — add `coverage` job |
| Modify | `.gitignore` — exclude generated coverage artifacts |

---

## Issue #422 — Implement Comprehensive Disaster Recovery Procedures

**Problem:** No documented disaster recovery procedures for contract failures.

### What to add

#### 1. DR runbook document

Create `docs/DISASTER_RECOVERY.md` with the following sections:

```markdown
# Disaster Recovery Runbook

## 1. Emergency Contract Pause
**Trigger:** Critical bug detected in production.
**Steps:**
1. Call `pause()` on the contract using the admin key.
2. Notify on-call team via escalation matrix (see §5).
3. Open a GitHub incident issue tagged `incident`.

## 2. State Backup and Restore
**Backup:** Run `./scripts/backup_contract_state.sh <CONTRACT_ID> <NETWORK>` — stores a JSON snapshot in `deployments/backups/`.
**Restore:** Run `./scripts/restore_contract_state.sh <BACKUP_FILE> <NETWORK>`.

## 3. Fund Recovery
**Steps:**
1. Pause contract.
2. Call `emergency_withdraw(admin, recipient)` if implemented.
3. Document transaction hash in the incident issue.

## 4. Data Migration
**Steps:**
1. Deploy new contract version.
2. Run `./scripts/migrate_state.sh <OLD_CONTRACT> <NEW_CONTRACT> <NETWORK>`.
3. Verify with `./scripts/verify_deployment.sh`.
4. Update DNS / client config to point to new contract ID.

## 5. Escalation Matrix
| Severity | Contact | SLA |
|----------|---------|-----|
| P0 – funds at risk | On-call lead + security team | 15 min |
| P1 – contract paused | On-call lead | 1 hour |
| P2 – degraded | Engineering team | 4 hours |

## 6. Recovery Time Objectives
| Scenario | RTO | RPO |
|----------|-----|-----|
| Contract pause | 15 min | 0 (no data loss) |
| State restore from backup | 2 hours | Last backup (≤24h) |
| Full redeploy | 4 hours | Last backup |

## 7. DR Testing Schedule
- Quarterly: full DR drill (pause → backup → restore → verify)
- Monthly: backup verification
- Weekly: health-check script run
```

#### 2. Backup script

Create `scripts/backup_contract_state.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

CONTRACT_ID="${1:?Usage: $0 <contract_id> <network>}"
NETWORK="${2:?Usage: $0 <contract_id> <network>}"
TIMESTAMP=$(date +%Y%m%dT%H%M%S)
OUT="deployments/backups/${NETWORK}_${CONTRACT_ID}_${TIMESTAMP}.json"

mkdir -p deployments/backups

stellar contract read \
  --id "$CONTRACT_ID" \
  --network "$NETWORK" \
  --output json > "$OUT"

echo "Backup saved to $OUT"
```

#### 3. Health monitoring cron (GitHub Actions)

Add `.github/workflows/dr-health-check.yml`:

```yaml
name: DR Health Check
on:
  schedule:
    - cron: '0 6 * * 1'   # every Monday at 06:00 UTC
  workflow_dispatch:

jobs:
  health-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run health check
        run: ./scripts/monitor_deployments.sh testnet --alert-on-failure
```

### Files to create / modify

| Action | Path |
|--------|------|
| Create | `docs/DISASTER_RECOVERY.md` |
| Create | `scripts/backup_contract_state.sh` |
| Create | `.github/workflows/dr-health-check.yml` |

---

## Commit Strategy

Group commits logically so the PR history is clean:

```bash
git add tests/upgrade/ .github/workflows/ci.yml
git commit -m "feat(#397): add upgrade testing framework with state and rollback tests"

git add contracts/medical_records/src/lib.rs docs/API_VERSIONING.md
git commit -m "feat(#418): add API versioning constants, compatibility check, and docs"

git add tarpaulin.toml .gitignore
git commit -m "feat(#420): add tarpaulin coverage config and CI coverage job"

git add docs/DISASTER_RECOVERY.md scripts/backup_contract_state.sh .github/workflows/dr-health-check.yml
git commit -m "feat(#422): add disaster recovery runbook, backup script, and health-check workflow"
```

---

## PR Checklist

- [ ] All 4 issue numbers referenced in PR description (`Closes #397`, `Closes #418`, `Closes #420`, `Closes #422`)
- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --all` passes with no warnings
- [ ] `cargo test --all` passes
- [ ] Coverage job passes with ≥ 80% line coverage
- [ ] Upgrade tests pass
- [ ] DR runbook reviewed by a second team member
