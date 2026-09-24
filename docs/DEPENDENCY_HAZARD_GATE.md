# Dependency Health / Hazard Gate

## Overview

Two scripts cover supply-chain and dependency risk for the contract
workspace:

- **`scripts/dependency_health_check.sh [output_path]`** — reports on
  workspace dependency availability and duplicate versions. Report-only;
  doesn't fail on its own.
- **`scripts/dependency_hazard_report.sh [version]`** — the actual gate.
  Checks workspace membership, Cargo `soroban-sdk` version alignment
  across contracts, circular dependencies, and WASM size budget, writing
  `schemas/dependency-hazard-report.json`. Exits non-zero when any hazard
  (not just a warning) is found.

## Usage

```bash
./scripts/dependency_health_check.sh reports/dependency_health.json
./scripts/dependency_hazard_report.sh
```

## CI

`.github/workflows/dependency-hazard-gate.yml` builds the workspace
(`make dist`, so the WASM size budget sub-check has real artifacts to
measure), runs both scripts, and fails the job on any hazard reported by
`dependency_hazard_report.sh` — nothing swallows the exit code. Both
reports are uploaded as build artifacts.
