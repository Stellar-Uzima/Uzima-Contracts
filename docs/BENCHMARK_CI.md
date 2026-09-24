# Benchmark / Load CI Job

`.github/workflows/benchmark-load.yml` runs `scripts/run_benchmarks.py`
against every contract, evaluating measured throughput against the
per-contract thresholds in `resource-budgets/contracts_budget.json`
(`min_throughput_ops_sec`). It also runs the runner's own unit tests
(`tests/benchmark_runner_test.py`) before running the real benchmarks.

## When it runs

- On every push/PR to `main` that touches `contracts/**`,
  `scripts/run_benchmarks.py`, `resource-budgets/**`, or
  `tests/benchmark_runner_test.py`.
- Nightly on a schedule (`0 3 * * *` UTC), so a regression introduced
  without touching those paths (e.g. a toolchain or dependency bump) is
  still caught.

## Failure behaviour

`scripts/run_benchmarks.py` calls `sys.exit(1)` whenever a contract's
measured `ops_per_sec` falls below its configured
`min_throughput_ops_sec` — nothing is swallowed with `|| true`, so the job
goes red on a real regression.

## Output

`tests/benchmark_results.json` (per-contract timing/throughput) is uploaded
as the `benchmark-results` build artifact on every run, pass or fail.

## Related

- [`PERFORMANCE_BUDGETS.md`](./PERFORMANCE_BUDGETS.md) — the separate
  wasm-size/storage/CPU budget gate (different system, different script).
- `scripts/run_benchmarks.py`, `tests/benchmark_runner_test.py`
