# SDK Bindings Drift Detection

## Overview

`scripts/generate-sdk-types.mjs` generates TypeScript and Python
(stellar-py style) SDK bindings from the shared contract schema. Those
generated files are committed, so they can silently drift from the schema
they're supposed to reflect if someone edits the schema without
regenerating.

## Usage

```bash
npm run sdk:generate   # write bindings (node scripts/generate-sdk-types.mjs)
npm run sdk:check      # fail if the committed copy has drifted (--check)
```

## CI

`.github/workflows/sdk-bindings-check.yml` runs `npm run sdk:check` on
every push/PR touching `scripts/generate-sdk-types.mjs`, `schemas/**`, or
`package.json`. It fails the job (no `|| true`) when the checked-in
bindings don't match what the generator would produce, and uploads the
drift report (`reports/sdk_bindings_drift.txt`) as a build artifact.

If this check fails, run `npm run sdk:generate` locally and commit the
regenerated files.
