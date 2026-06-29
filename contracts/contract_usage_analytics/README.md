# Contract Usage Analytics

A Soroban smart contract for tracking and analyzing on-chain contract usage patterns across the Uzima ecosystem.

## Overview

The `contract_usage_analytics` contract provides a comprehensive framework for recording, aggregating, and querying contract interaction data. It enables teams to monitor function call frequencies, user behavior, gas consumption trends, error rates, and performance metrics — all stored verifiably on-chain.

## Analytics Tracked

| Category | Details |
|---|---|
| **Function Call Frequencies** | Call count per function, total and per-period |
| **User Behavior Patterns** | Per-user call history, activity timestamps |
| **Gas Usage Trends** | CPU instruction units + RAM byte-seconds per call |
| **Error Rates** | Error counts per function; rolling error rate in basis points |
| **Performance Metrics** | Per-function average latency; daily rolling snapshots |

## Contract Interface

### `initialize(admin: Address)`
Initialises the analytics contract with an admin address. Can only be called once.

### `record_call(function_name, user, cpu_usage, ram_usage, success, latency_ms)`
Records a single contract interaction. Called by integrating contracts (or off-chain relayers) after each function invocation.

- `function_name` — name of the invoked function
- `user` — address of the caller
- `cpu_usage` / `ram_usage` — Soroban resource units consumed
- `success` — whether the call completed without error
- `latency_ms` — observed wall-clock latency in milliseconds

### `take_snapshot()`
Aggregates current metrics into a `UsageSnapshot` and appends it to a rolling 30-entry history.

### Getters
| Function | Returns |
|---|---|
| `get_function_metrics(name)` | `Option<FunctionMetric>` |
| `get_user_metrics(user)` | `Option<UserMetric>` |
| `get_all_functions()` | `Vec<String>` |
| `get_snapshots()` | `Vec<UsageSnapshot>` |

## Data Structures

```rust
FunctionMetric {
    name, call_count, total_cpu_usage, total_ram_usage,
    error_count, avg_latency_ms, last_called
}

UserMetric {
    user, total_calls, last_active
}

UsageSnapshot {
    timestamp, total_calls, active_users, error_rate_bps
}
```

## Dashboard

A pre-built analytics dashboard is available at [`/dashboard/index.html`](../../dashboard/index.html). It visualises:

- Call frequency trend (last 30 days)
- Function call distribution (doughnut chart)
- Key KPI cards (total calls, active users, error rate, latency)
- Sortable function metrics table

## Automated Reports

Run the report generator to produce a dated Markdown report in `/reports/`:

```bash
node scripts/generate_usage_report.js
```

Reports include: summary KPIs, function frequency tables, daily snapshot history, gas trends, and error analysis.

## Integration Pattern

Integrating contracts emit usage data by calling this contract after their own logic executes:

```rust
// In your contract
analytics_client.record_call(
    &env,
    &String::from_str(&env, "my_function"),
    &caller,
    cpu_consumed,
    ram_consumed,
    true,   // success
    latency_ms,
);
```
