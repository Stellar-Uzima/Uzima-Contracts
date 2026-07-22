# Contract Interface Compatibility Policy

This document defines the breaking vs non-breaking change policy for the Uzima contract interface registry and SDK bindings.

## Overview

The `schemas/interface-registry/registry.json` file is the **single source of truth** for all contract ABI surfaces consumed by generated SDK clients (TypeScript, Python) and documentation. CI enforces that changes to this registry are reviewed and classified before merge.

## Change Classification

### Breaking Changes (CI fails)

Breaking changes require explicit approval via `[approve-abi-change]` in the PR body. These changes will cause downstream integration regressions if not coordinated:

| Change Type | Example |
|-------------|---------|
| Enum variant removed | `ConsentStatus.EXPIRED` deleted |
| Enum value changed | `RecordType.DIAGNOSIS` value changed from `"diagnosis"` to `"dx"` |
| Enum removed entirely | `ActionType` deleted |
| Required field added | Adding `newField: string` to `MedicalRecord` |
| Required field type changed | `amount: number` changed to `amount: string` |
| Field removed | `signature` removed from `MedicalRecord` |
| Optional field became required | `tags?: string[]` changed to `tags: string[]` |
| Interface removed | `PaymentStatus` interface deleted |
| Contract removed from registry | `medical_records` contract removed |

### Non-Breaking Changes (CI passes)

These changes are additive and do not break existing integrations:

| Change Type | Example |
|-------------|---------|
| Enum variant added | `RecordType.PROCEDURE` added |
| Enum added | New `TelehealthStatus` enum |
| Interface added | New `LabOrder` interface |
| Optional field added | `newOptional?: string` added to `MedicalRecord` |
| Required field became optional | `signature: string` changed to `signature?: string` |
| Contract added to registry | New `lab_orders` contract registered |
| Description updated | Field description text changed |

## CI Workflow

The `abi-compatibility` job in `.github/workflows/ci.yml` runs on every push and pull request:

1. Generates a fresh ABI snapshot from the canonical definitions in `scripts/abi-compat.mjs`
2. Compares against the committed baseline at `schemas/interface-registry/registry.json`
3. Classifies all changes as breaking or non-breaking
4. Posts a compatibility report as a PR comment
5. **Fails** if any breaking changes are detected (unless `[approve-abi-change]` is in the PR body)

## Workflow for Breaking Changes

When you need to make a breaking change to a contract interface:

1. Update the canonical definitions in `scripts/abi-compat.mjs` and `scripts/generate-sdk-types.mjs`
2. Run `node scripts/abi-compat.mjs` to regenerate the baseline
3. Run `node scripts/generate-sdk-types.mjs` to regenerate SDK bindings
4. Add `[approve-abi-change]` to your PR body
5. Document the migration path in your PR description
6. Coordinate with downstream consumers (mobile SDK, API integrations)

## Architecture

```
scripts/abi-compat.mjs          # Canonical interface definitions + compatibility checker
scripts/generate-sdk-types.mjs  # SDK binding generator (must stay in sync)
schemas/interface-registry/
  registry.json                 # Committed ABI baseline (versioned snapshots)
  registry.schema.json          # JSON Schema for the registry format
```

Both `abi-compat.mjs` and `generate-sdk-types.mjs` share the same canonical definitions. The CI compatibility gate ensures they stay in sync by comparing the generated output against the committed baseline.

## Running Locally

```bash
# Generate/update baseline (after intentional interface changes)
node scripts/abi-compat.mjs

# Check for uncommitted drift
node scripts/abi-compat.mjs --check

# Generate report without failing
node scripts/abi-compat.mjs --check --allow-breaking

# SDK binding sync check
node scripts/generate-sdk-types.mjs --check

# Full local validation
node scripts/abi-compat.mjs --check && node scripts/generate-sdk-types.mjs --check
```
