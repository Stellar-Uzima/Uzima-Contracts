# Contract Resource Budgets

This document describes the deterministic resource budget pipeline for Uzima contracts.

## Overview

The resource budget pipeline tracks WASM size, storage entry, and CPU instruction budgets per contract. It compares current measurements against declared budgets and baselines, failing CI when a contract exceeds its limits.

## Budget Configuration

Budgets are defined in `resource-budgets/budgets.json`. Each contract entry specifies:

| Field | Description | Default |
|---|---|---|
| `max_wasm_bytes` | Maximum WASM binary size in bytes | 65536 (64 KB) |
| `max_storage_entries` | Maximum storage entries per contract | 500 |
| `max_cpu_instructions` | Maximum CPU instructions per operation | 10,000,000 |
| `regression_tolerance_pct` | Maximum allowed regression over baseline (%) | 10 |
| `notes` | Human-readable description | — |

## Budget Schema

```json
{
  "version": "1.0.0",
  "defaults": {
    "max_wasm_bytes": 65536,
    "max_storage_entries": 500,
    "max_cpu_instructions": 10000000,
    "regression_tolerance_pct": 10
  },
  "contracts": {
    "<contract_name>": {
      "max_wasm_bytes": 65536,
      "max_storage_entries": 500,
      "max_cpu_instructions": 10000000,
      "regression_tolerance_pct": 10,
      "notes": "Description"
    }
  }
}
```

## Baselines

Baselines are stored in `resource-budgets/baselines.json`. To update baselines after a legitimate change:

```bash
node scripts/measure_budgets.mjs --update-baselines
```

This measures all built WASM files and writes the current sizes to the baseline file.

## Measurement Runner

The measurement runner (`scripts/measure_budgets.mjs`) performs these steps:

1. Reads the budget configuration from `resource-budgets/budgets.json`
2. Measures WASM binary sizes from `target/wasm32-unknown-unknown/release/`
3. Compares against declared budgets
4. Compares against baselines for regression detection
5. Generates a violations report and markdown PR comment

### Usage

```bash
# Measure all contracts
node scripts/measure_budgets.mjs

# Measure a single contract
node scripts/measure_budgets.mjs medical_records

# Update baselines
node scripts/measure_budgets.mjs --update-baselines
```

## Output Files

| File | Description |
|---|---|
| `reports/budget_violations.json` | Structured violation data for CI |
| `reports/budget_report.md` | Human-readable markdown report |

## CI Integration

The CI workflow runs the budget check on every PR:

1. Builds all contracts
2. Runs `node scripts/measure_budgets.mjs`
3. Uploads the budget report as an artifact
4. Comments on the PR with the violation summary
5. Fails the check if any contract exceeds its budget

## Regression Detection

The pipeline detects regressions by comparing current WASM size against the committed baseline:

- If a contract grows more than `regression_tolerance_pct` over its baseline, a warning is raised
- If a contract grows more than 2x the tolerance, a critical violation is raised
- Baselines are committed to the repo and updated manually via `--update-baselines`

## Adding a New Contract

1. Add the contract entry to `resource-budgets/budgets.json`
2. Set appropriate budget limits based on the contract's complexity
3. Build the contract: `cargo build --release --target wasm32-unknown-unknown`
4. Update baselines: `node scripts/measure_budgets.mjs --update-baselines`
5. Commit both `budgets.json` and `baselines.json`

## Updating Budgets

When a legitimate change increases a contract's size:

1. Build and verify the change is necessary
2. Update the budget in `resource-budgets/budgets.json`
3. Update baselines: `node scripts/measure_budgets.mjs --update-baselines`
4. Include a note in the PR explaining why the budget was increased
