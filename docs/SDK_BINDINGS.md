# SDK Bindings Drift Detection

## Overview

`scripts/generate-sdk-types.mjs` generates TypeScript, Python (stellar-py
style), Kotlin and Swift SDK bindings from the shared contract schema plus the
public interfaces declared in `schemas/interface-registry/registry.json`.
Those generated files are committed, so they can silently drift from the
schema they're supposed to reflect if someone edits the schema without
regenerating. A new public interface with no generated binding is treated as
drift and fails CI.

| Language | Generated binding |
|----------|-------------------|
| TypeScript | `mobile-sdk/core/src/generated/contract-bindings.ts` |
| Python | `mobile-sdk/python/uzima_sdk/contract_bindings.py` |
| Kotlin | `mobile-sdk/android/src/generated/ContractBindings.kt` |
| Swift | `mobile-sdk/ios/src/generated/ContractBindings.swift` |

## Usage

```bash
npm run sdk:generate   # write all four bindings
npm run sdk:check      # fail on drift, missing language bindings, or unbound new interfaces
```

`npm run sdk:check` runs `scripts/check-sdk-bindings.sh`, which performs two
checks:

1. **Drift and coverage** — `generate-sdk-types.mjs --check` regenerates the
   four bindings from the registry and compares them with the committed files.
   It fails when a file is missing/stale, or when a registered contract or
   interface has no generated client binding in any of the four languages.
2. **New public interfaces** — `scripts/check-sdk-bindings.mjs` compares the
   change with the base ref (`SDK_BINDINGS_BASE`, default `HEAD` for local
   runs; CI sets it to the PR/push base commit) and fails when a
   `#[contractimpl] pub fn` is added without a matching entry in
   `schemas/interface-registry/registry.json`. Existing functions are the
   baseline; only additions fail.

Both checks produce a machine-readable artifact:
`reports/sdk_bindings_drift.txt` contains `SDK_BINDINGS_DRIFT` and
`SDK_BINDINGS_MISSING` counters plus one `MISSING=...` line per coverage gap.

## CI

`.github/workflows/sdk-bindings-check.yml` runs `npm run sdk:check` on every
push/PR touching the generator, the check scripts, the registry, generated
bindings, contract sources, or `package.json`. It fails the job (no
`|| true`) when any of the checks above fail, runs the coverage regression
tests, and uploads the drift report as a build artifact.

If this check fails, add the new interface to the registry (for new public
contract functions), run `npm run sdk:generate`, and commit the regenerated
bindings.
