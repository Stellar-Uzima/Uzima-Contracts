# WASM Size Monitoring

This repository gates contract WASM growth in CI so deployment-size regressions are caught before they reach `main`.

## CI Behavior

The `Build (wasm32)` job in `.github/workflows/ci.yml` runs:

1. `cargo build --workspace --target wasm32-unknown-unknown --release`
2. A collection step that copies `target/wasm32-unknown-unknown/release/*.wasm` into `dist/`
3. A baseline-resolution step that uses the base branch copy of `scripts/wasm_size_baselines.json` for pull requests
4. `bash ./scripts/wasm_size_monitor.sh --baseline reports/wasm_size_baselines.base.json`
5. Upload of `reports/wasm_size_report.md`
6. A pull request comment with the report table and delta against the base branch baseline

The `load_testing` contract is excluded because it is a known non-deployable test contract that exceeds the deployment-size budget.

## Gates

`scripts/wasm_size_monitor.sh` fails the job when any non-excluded contract:

- exceeds its committed baseline by more than 10%
- exceeds the 60 KiB warning threshold

The script writes `WASM_SIZE_STATUS=passed` or `WASM_SIZE_STATUS=failed` into `reports/wasm_size_report.md`. The PR comment step reads that stable token and updates a single bot-authored WASM report comment on each workflow rerun.

Contracts without a baseline are still checked against the 60 KiB warning threshold and are marked `NO_BASELINE` in the report. Run `--update` after a clean `main` build to add or refresh their baseline entries.

On pull requests, CI compares against the baseline file from the target branch when it exists. That prevents a feature branch from hiding a regression by updating its own baseline in the same PR.

## Baseline File

Baselines live in `scripts/wasm_size_baselines.json`.

```json
{
  "schemaVersion": 1,
  "limits": {
    "maxBytes": 65536,
    "warningBytes": 61440,
    "regressionPercent": 10.0
  },
  "excludedContracts": ["load_testing"],
  "contracts": {
    "medical_records": 48231
  }
}
```

The `contracts` object maps contract artifact names, without the `.wasm` suffix, to byte counts generated from `dist/*.wasm`.

## Local Usage

Build the WASM artifacts and run the gate:

```bash
cargo build --workspace --target wasm32-unknown-unknown --release
rm -rf dist reports
mkdir -p dist reports
cp target/wasm32-unknown-unknown/release/*.wasm dist/
bash ./scripts/wasm_size_monitor.sh
```

Regenerate the baseline after intentionally accepted size changes on a clean `main` build:

```bash
bash ./scripts/wasm_size_monitor.sh --update
```

Useful options:

```bash
bash ./scripts/wasm_size_monitor.sh --dist dist
bash ./scripts/wasm_size_monitor.sh --baseline scripts/wasm_size_baselines.json
bash ./scripts/wasm_size_monitor.sh --report reports/wasm_size_report.md
```

The script uses Python 3 for JSON parsing and report generation, so it does not require `jq` or `bc`.

## Reviewing Regressions

When CI fails, open the `wasm-size-report` artifact or the PR comment and check:

- which contract failed
- current size
- baseline size
- delta percentage
- whether the failure is the 10% regression gate, the 60 KiB threshold, or both

If the size growth is intentional, document the reason in the PR and regenerate the baseline from a clean build. If it is accidental, inspect dependencies, large error strings, serialization code, or other recently added logic before updating the baseline.
