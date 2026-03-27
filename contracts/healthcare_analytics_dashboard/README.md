# Healthcare Analytics Dashboard Contract

Soroban contract for privacy-preserving healthcare analytics, monitoring, reporting, and compliance automation.

## Features

- Privacy-preserving metric aggregation with cohort threshold and deterministic noise
- Real-time system monitoring snapshots and rolling KPI tracking
- Report template creation, report scheduling, and export record generation
- Compliance summary automation for audit and violation tracking
- Visualization series retrieval for dashboard charting
- Integration with `ai_analytics` federated rounds (`sync_ai_round`)

## Key Methods

- `initialize(admin, min_cohort_size, noise_bps)`
- `set_collector(caller, collector, enabled)`
- `record_medical_metric(caller, metric_name, period_id, metric_value_bps, cohort_size)`
- `record_system_snapshot(caller, active_users, tx_count, error_count, latency_p95_ms, uptime_bps)`
- `create_report_template(...)`
- `schedule_report(...)`
- `run_scheduled_report(...)`
- `upsert_compliance_summary(...)`
- `configure_ai_analytics(caller, ai_analytics_contract)`
- `sync_ai_round(caller, round_id)`

## Integration Notes

- `sync_ai_round` expects the target `ai_analytics` contract to implement `get_round(round_id) -> Option<FederatedRound>`.
- This module stores mapped round insights in `AiRoundInsight` for dashboard use.

## Privacy Model

- Aggregation is rejected if `cohort_size < min_cohort_size`.
- Input values are stored as basis points (`0..=10000`) and transformed with bounded deterministic noise (`noise_bps`) before aggregation.

## Testing

Run unit tests:

```bash
cargo test -p healthcare_analytics_dashboard
cargo test -p healthcare_analytics_dashboard --features testutils
```

The `testutils` run includes:

- privacy and KPI flow
- report template/schedule/compliance/export flow
- `ai_analytics` integration sync flow
