# Contract Change Impact Matrix

This document provides tools and guidance for analyzing the impact of changes to Uzima Contracts before merging or deploying.

---

## Impact Matrix

The matrix below maps each contract to the contracts it depends on (calls) and the contracts that depend on it (callers). Use this when assessing the blast radius of a change.

| Contract | Depends On | Called By |
|---|---|---|
| `medical_records` | `identity_registry`, `rbac`, `audit` | `patient_portal`, `cross_chain_bridge`, `emr_integration`, `fhir_integration` |
| `identity_registry` | `rbac` | `medical_records`, `cross_chain_identity`, `credential_registry` |
| `cross_chain_bridge` | `medical_records`, `identity_registry`, `cross_chain_access` | `cross_chain_identity`, `cross_chain_enhancements` |
| `audit` | — | `medical_records`, `aml`, `anomaly_detector`, `health_data_access_logging` |
| `aml` | `audit`, `rbac` | `healthcare_compliance`, `healthcare_compliance_automation` |
| `anomaly_detector` | `audit` | `healthcare_compliance`, `clinical_decision_support` |
| `rbac` | — | `medical_records`, `identity_registry`, `aml`, `anomaly_detector` |
| `escrow` | `sut_token` | `healthcare_payment`, `appointment_booking_escrow` |
| `governor` | `timelock` | — |
| `timelock` | — | `governor` |

> **Note**: This matrix is manually maintained. Run the dependency analysis script (see below) to regenerate it automatically.

---

## Impact Analysis Checklist

When making a change to a contract, complete this checklist:

### Affected Contracts
- [ ] Identify all contracts in the "Called By" column for the changed contract
- [ ] Review each dependent contract for API compatibility
- [ ] Check if any dependent contract caches state from the changed contract

### Dependency Analysis
- [ ] Run `scripts/analyze_dependencies.sh <contract>` to list all transitive dependencies
- [ ] Verify no circular dependencies introduced
- [ ] Check Cargo.toml workspace dependencies for version conflicts

### API Changes
- [ ] List all added/removed/modified public functions
- [ ] For removed functions: confirm no callers exist (grep across all contracts)
- [ ] For modified signatures: update all callers
- [ ] For new functions: add to API reference (`docs/api.md`)

### State Migration Needs
- [ ] Does the change add/remove/rename `DataKey` variants? → Migration required
- [ ] Does the change modify stored struct fields? → Migration required
- [ ] If migration needed: write and test migration script before deployment

### Testing Requirements
- [ ] Unit tests updated for changed functions
- [ ] Integration tests updated for cross-contract interactions
- [ ] Regression tests added for any bug fixes
- [ ] Testnet deployment tested before mainnet

---

## Automated Impact Analysis

### Dependency graph script

Add `scripts/analyze_dependencies.sh` to generate a dependency report for a given contract:

```bash
#!/usr/bin/env bash
# Usage: ./scripts/analyze_dependencies.sh <contract_name>
CONTRACT="${1:?Usage: $0 <contract_name>}"
echo "=== Direct callers of $CONTRACT ==="
grep -rl "$CONTRACT" contracts/*/src/lib.rs | grep -v "/$CONTRACT/"

echo ""
echo "=== Functions exported by $CONTRACT ==="
grep "pub fn" "contracts/$CONTRACT/src/lib.rs" | sed 's/.*pub fn //' | cut -d'(' -f1
```

### PR automation

Add the following to `.github/workflows/ci.yml` to generate an impact report on every PR:

```yaml
- name: Contract change impact analysis
  if: github.event_name == 'pull_request'
  run: |
    changed=$(git diff --name-only origin/main...HEAD | grep "contracts/" | \
              sed 's|contracts/\([^/]*\)/.*|\1|' | sort -u)
    echo "Changed contracts: $changed"
    for contract in $changed; do
      echo "=== Impact: $contract ==="
      grep -rl "$contract" contracts/*/src/lib.rs | grep -v "/$contract/" || echo "  No direct callers found"
    done | tee reports/impact_analysis.txt

- name: Upload impact analysis
  if: github.event_name == 'pull_request'
  uses: actions/upload-artifact@v4
  with:
    name: impact-analysis
    path: reports/impact_analysis.txt
```

### Review guidance comment

Post the impact analysis as a PR comment to guide reviewers:

```yaml
- name: Comment impact analysis
  if: github.event_name == 'pull_request'
  uses: actions/github-script@v7
  with:
    script: |
      const fs = require('fs');
      if (!fs.existsSync('reports/impact_analysis.txt')) return;
      const report = fs.readFileSync('reports/impact_analysis.txt', 'utf8');
      github.rest.issues.createComment({
        issue_number: context.issue.number,
        owner: context.repo.owner,
        repo: context.repo.repo,
        body: `### 🔍 Contract Change Impact Analysis\n\n\`\`\`\n${report.slice(0, 3000)}\n\`\`\`\n\nReview dependent contracts for compatibility before merging.`
      });
```

---

## Maintaining This Document

Update the impact matrix whenever:
- A new contract is added to the workspace
- A contract gains or loses a cross-contract dependency
- A contract is deprecated or removed

The matrix should be reviewed as part of every PR that adds a new contract or modifies cross-contract call sites.
