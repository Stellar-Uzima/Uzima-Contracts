# WASM Size Baseline Staleness — Gap Analysis

Tracks: #1576 "Detect stale wasm_size_baselines.json entries"

## Finding

`scripts/wasm_size_baselines.json` tracks per-contract WASM byte sizes for
the regression gate described in `docs/PERFORMANCE_BUDGETS.md`. Comparing
its `"contracts"` keys against the actual `contracts/*/` directories:

- **114** contract directories have a `Cargo.toml`.
- **78** have a baseline entry.
- **7** of the remaining 36 are shared library crates with no `#[contract]`
  item (`common_auth`, `common_error`, `contract_behavior_fuzzing`,
  `fp_math`, `sanitization`, `shared_consent_policy`, `test-helpers`) —
  these don't produce a WASM artifact, so they were added to `"excluded"`
  as part of this change rather than left looking like an oversight.
- **31** are genuine deployable contracts (have a `#[contract]` item)
  shipped with **no baseline entry at all**, including `medical_records` —
  one of the most central contracts in the repo. Full list produced by the
  check script below.
- **0** stale entries reference a contract that no longer exists (a clean
  result — nothing to prune).

## Why the baseline wasn't regenerated here

Populating real byte sizes for the 31 missing contracts requires an actual
`cargo build --release --target wasm32-unknown-unknown` and reading the
resulting `.wasm` file sizes — not something done in this change (no
build/compile was run here, per the constraints of this pass). Guessing at
numbers would defeat the entire point of a size-regression baseline.

## What landed instead

- `scripts/check_wasm_baseline_staleness.sh` — walks `contracts/*/`,
  flags any deployable contract missing a baseline entry (or not in
  `"excluded"`). Currently **non-blocking**: it reports the 31-contract
  gap without failing the job, since flipping it blocking today would
  turn every future PR red until someone does the real build-and-measure
  work. Pass `--strict` once that backlog is cleared, so it only blocks
  *new* contracts from shipping without a baseline going forward.
- Added the 7 legitimate library crates to `"excluded"` in
  `wasm_size_baselines.json`.
- Wired into `.github/workflows/wasm-baseline-check.yml`.

## Follow-ups

1. Run a full release build and populate baseline entries for the 31
   missing contracts (`scripts/wasm_size_monitor.sh` or
   `scripts/measure_storage.sh` after building).
2. Once populated, flip `check_wasm_baseline_staleness.sh` to `--strict`
   in the workflow.
3. Cross-reference **#1556** ("Add WASM size monitoring workflow") — that
   issue is about wiring `wasm_size_monitor.sh` itself into CI as the
   size-regression gate `docs/PERFORMANCE_BUDGETS.md` describes; this
   change only adds the narrower "does every contract have an entry"
   check, and doesn't duplicate that broader work.
