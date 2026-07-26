# Performance Budgets and Benchmark Gate

Repository-wide framework that ties every code change to the three resource
dimensions that decide whether a Soroban contract can be deployed and operated
affordably:

| Dimension | Source | Why it matters |
|---|---|---|
| **WASM size** | wasm32 release build | Hard 64 KB deploy limit. Over it, the contract cannot be installed at all. |
| **Storage entries** | `scripts/measure_storage.sh` | Drives ledger rent. Unbounded growth makes a contract progressively more expensive to keep alive. |
| **CPU instructions** | `bench_storage_*` benchmarks | Drives transaction fees and the risk of exceeding the per-transaction resource budget. |

This complements the existing tooling rather than replacing it:

- `scripts/wasm_size_monitor.sh` remains the size-only baseline gate (Issue #846).
- `scripts/measure_storage.sh` remains the measurement engine for storage and CPU.
- **`scripts/performance_budget_gate.sh`** is the unified gate introduced here: it
  reads both sources, evaluates them against one versioned budget, and produces a
  single actionable report.

## Quick start

```bash
# Build the artifacts the gate measures
cargo build --workspace --target wasm32-unknown-unknown --release

# Optional but recommended: produce storage/CPU measurements
bash scripts/measure_storage.sh

# Evaluate the current build against the budgets
bash scripts/performance_budget_gate.sh --check

# See how tracked contracts have moved over retained samples
bash scripts/performance_budget_gate.sh --trend
```

Or via make:

```bash
make perf-budget        # check
make perf-budget-update # re-record budgets
make perf-budget-trend  # trend view
```

## The budget file

Budgets are versioned in `scripts/performance_budgets.json` and reviewed like any
other source change.

```jsonc
{
  "schema_version": 1,
  "limits": {                        // absolute thresholds
    "max_wasm_size_bytes": 65536,    // hard Soroban limit
    "warn_wasm_size_bytes": 61440,
    "max_storage_entries": 500,
    "warn_storage_entries": 100,
    "max_cpu_instructions": 10000000,
    "warn_cpu_instructions": 5000000
  },
  "regression": {                    // relative tolerances
    "wasm_size_pct": 10,
    "storage_entries_pct": 15,
    "cpu_instructions_pct": 15,
    "min_wasm_delta_bytes": 512,           // noise floor
    "min_cpu_delta_instructions": 50000    // noise floor
  },
  "excluded": ["load_testing"],
  "contracts": {
    "genomic_data": {
      "tier": "critical",
      "wasm_size": 61216,
      "grandfathered_over_size_limit": false
    }
  }
}
```

The absolute limits mirror the values already documented in
[`CONTRACT_RESOURCE_LIMITS.md`](./CONTRACT_RESOURCE_LIMITS.md) and enforced by
`measure_storage.sh`, so there is one source of truth for what "too big" means.

## What fails the build

The gate deliberately tracks **both** absolute thresholds and relative
regressions, so that a contract cannot drift toward a hard limit unnoticed while
still allowing necessary feature work.

A contract **fails** when any of the following is true:

1. A tracked metric grows more than its tolerance over the recorded budget
   (default: size +10 %, entries +15 %, CPU +15 %), **and** the absolute change
   exceeds the noise floor.
2. A metric **newly crosses** a hard limit that it was previously under.
3. A metric is over a hard limit and the contract is **not** grandfathered.
4. A brand-new contract is introduced already over a hard limit.

A contract **warns** (does not fail) when:

- It is over a hard limit and `grandfathered_over_size_limit` is `true`.
- It is over a warning threshold but still under the hard limit.

### Grandfathering

Several contracts already exceed 64 KB (for example `ihe_integration` at ~91 KB).
Failing on those immediately would make the gate permanently red and therefore
ignored. They are recorded with `"grandfathered_over_size_limit": true`, which
reports them as `:warning:` and fails only if they regress **further**. As those
contracts are optimised below the limit, flip the flag to `false` to lock the win
in.

### Noise floor

A tolerance alone is misleading for small values: a 400-byte change on a 1 KB
contract is +40 % but practically irrelevant. `min_wasm_delta_bytes` and
`min_cpu_delta_instructions` require the absolute change to matter as well before
a percentage regression is failed.

## Updating a budget

When an increase is intentional and reviewed:

```bash
cargo build --workspace --target wasm32-unknown-unknown --release
bash scripts/measure_storage.sh
bash scripts/performance_budget_gate.sh --update
git add scripts/performance_budgets.json
```

Commit the change **with** the code that caused it, so review sees the size,
storage and CPU cost of the feature alongside the feature itself. The `tier` of an
already-tracked contract is preserved across updates.

## CI behaviour

The `Build (wasm32)` job runs the gate after the storage measurement step:

- The markdown report is uploaded as the `performance-budget-report` artifact and
  posted as a PR comment.
- The gate fails the job on a budget violation.
- `reports/performance_budget_result.json` is emitted for downstream tooling.

If `reports/storage_summary.json` is absent (benchmarks did not run), the gate
degrades gracefully to a **size-only** evaluation and says so in the report,
rather than failing the build for a missing measurement.

To roll the gate out in observation mode first, set `WARN_ONLY=1`: violations are
reported without failing the build.

## Coverage

Per the incremental rollout, an explicit budget is recorded for the contracts
closest to a hard limit (`tier: "critical"`) plus a set of large `standard`
contracts. Built contracts without a recorded budget are counted as *untracked*
in the report and are still gated against the absolute hard limits, so a new
contract cannot land already over the limit.

Expand coverage by running `--update` after a clean build, which records every
built contract.

## Trend view

Each `--check` appends a sample to `.performance_budget_trends.json` (most recent
50 retained). `--trend` compares the oldest retained sample with the newest and
prints per-contract movement, so slow drift across many PRs is visible even when
no single PR trips a tolerance.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `Budget file not found` | `scripts/performance_budgets.json` missing | `bash scripts/performance_budget_gate.sh --update` |
| `WASM dir not found` | Contracts not built for wasm32 | `cargo build --workspace --target wasm32-unknown-unknown --release` |
| All metrics `:grey_question: not measured` | `reports/storage_summary.json` absent | `bash scripts/measure_storage.sh` |
| Gate fails right after a dependency bump | Real size regression from the new dependency | `cargo tree -d`, then optimise or re-record the budget |

## Related

- [`CONTRACT_RESOURCE_LIMITS.md`](./CONTRACT_RESOURCE_LIMITS.md) — the underlying Soroban limits
- `scripts/wasm_size_monitor.sh` — size-only baseline gate (Issue #846)
- `scripts/measure_storage.sh` — storage and CPU measurement engine
- `contract_optimizer/` — optimisation passes for oversized contracts
