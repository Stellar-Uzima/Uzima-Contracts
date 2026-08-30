# Telemetry Schema & Incident Response Framework

This document defines the versioned on-chain telemetry event schema and incident-response workflow for the Uzima monitoring contract.

## Overview

The monitoring contract emits structured telemetry events that off-chain systems can parse to build dashboards, trigger alerts, and conduct incident response. Every event carries a **schema version** for forward/backward compatibility, a **trace ID** to link every contract in a single logical transaction, a legacy **correlation ID**, and a **classification** that separates routine operations from security-relevant anomalies.

## Event Schema

Each telemetry event is a `TelemetryEvent` struct emitted with topic `(TEL, <type_symbol>)`.

| Field | Type | Description |
|---|---|---|
| `schema_version` | `u32` | Packed semver `MAJOR*10000 + MINOR*100 + PATCH` |
| `correlation_id` | `BytesN<32>` | **Legacy** per-contract ID, derived from the direct caller + timestamp. Not a reliable trace key. |
| `trace_id` | `BytesN<32>` | Trace ID invariant across a cross-contract call chain (see [Trace IDs](#trace-ids)) |
| `contract_name` | `String` | Name of the emitting contract |
| `contract_version` | `String` | Version of the emitting contract |
| `event_type` | `TelemetryEventType` | Specific type of event |
| `event_class` | `EventClass` | `Operational` or `Security` |
| `severity` | `TelemetrySeverity` | `Info`, `Warn`, `Error`, or `Critical` |
| `operation` | `String` | Name of the function or operation |
| `detail` | `String` | Additional context (key=value) |
| `timestamp` | `u64` | Ledger timestamp |

## Schema Versioning

The version follows semver:

- **MAJOR** (breaking): removed, reordered, or newly required fields — all consumers must update.
- **MINOR** (additive): new optional fields appended at the end — consumers can ignore.
- **PATCH** (fixes): bug fixes, documentation, no schema change.

Current version: **2.0.0** (`schema_version = 20000`).

Version 2.0.0 (a MAJOR bump from 1.0.0) is a breaking change: `TelemetryEvent` gained a required `trace_id` field, changing the packed event payload. Any off-chain consumer of v1 must be updated to emit and parse v2.

To upgrade, update `SCHEMA_VERSION_MINOR` or `SCHEMA_VERSION_MAJOR` in `telemetry.rs` and emit a migration event in the changelog.

## Event Types

### Operational (class = `Operational`)

| Symbol | Enum | Description |
|---|---|---|
| `FN_INVOKE` | `FunctionInvoked` | A contract function was called |
| `FN_DONE` | `FunctionCompleted` | A contract function completed (error variant on failure) |
| `STATE` | `StateTransition` | Contract state changed |
| `METRIC` | `MetricUpdated` | A metric counter was updated |

### Security (class = `Security`)

| Symbol | Enum | Description |
|---|---|---|
| `AUTH_FAIL` | `AuthFailure` | Authentication check failed |
| `AUTHZ_FAIL` | `AuthorizationFailure` | Authorization check failed |
| `THRESHOLD` | `ThresholdBreached` | A monitoring threshold was exceeded |
| `ANOMALY` | `AnomalyDetected` | Unusual pattern detected |
| `CFG_CHG` | `ConfigChange` | Configuration change attempted |

## Severity Levels

| Level | Meaning |
|---|---|
| `Info` (0) | Normal operation — routine call/completion |
| `Warn` (1) | Unexpected but non-critical condition |
| `Error` (2) | Operation failed |
| `Critical` (3) | System-level failure, threshold breach, requires immediate attention |

## Trace IDs

Every event includes a `trace_id` (`BytesN<32>`) that is **invariant across a single transaction's cross-contract call chain**. Off-chain systems group events by `trace_id` to reconstruct a full execution trace.

The trace ID is derived by the top-level submitter from the caller that authorized the submission plus the current ledger sequence:

```text
trace_id = sha256(top_level_caller_address || ledger_sequence)
```

The derivation rule is implemented once in `telemetry::derive_trace_id`. The top-level contract computes it and **forwards the value** through every downstream contract in the chain; each downstream contract reuses the ID it was passed rather than recomputing it from its own direct caller. Because the direct caller changes at every hop (contract A's caller is the end user, contract B's caller is A), recomputing per hop is exactly what produced disjoint IDs — the fix is to forward a single value.

The legacy `correlation_id` is derived per-contract from the direct caller and ledger timestamp; it is **not** a reliable trace key and must not be used to reconstruct traces. Consumers of events at or above schema 2.0.0 must group on `trace_id`.

> **Note for off-chain consumers:** the on-chain `trace_id` derivation (top-level caller + ledger sequence) must be mirrored off-chain so events regroup identically. The M1 trace extractor implements this same rule.

## Threshold Breach Events

When a monitoring threshold is breached (error rate, gas, storage), the contract emits a `ThresholdBreached` event with severity `Critical`. The `detail` field indicates which threshold was breached (`error_rate`, `gas`, or `storage`).

## Telemetry Snapshot

The `get_telemetry_snapshot()` query returns a `TelemetrySnapshot` struct summarising all recorded events:

| Field | Description |
|---|---|
| `schema_version` | Packed version of the snapshot schema |
| `total_events` | Sum of all recorded events |
| `operational_count` | Events with class `Operational` |
| `security_count` | Events with class `Security` |
| `error_count` | Events with severity `Error` |
| `critical_count` | Events with severity `Critical` |
| `snapshot_at` | Ledger timestamp of the snapshot |

## Incident Response Workflow

When an alert threshold is breached:

1. The contract emits a `ThresholdBreached` event with severity `Critical`.
2. Off-chain monitoring (Grafana, PagerDuty, custom) detects the event.
3. The operator queries `get_telemetry_snapshot()` to assess the current state.
4. Events with the same `trace_id` are correlated to trace the root cause.
5. The operator may call `update_alert_config()` (admin only) to adjust thresholds or silence non-critical alerts.
6. For persistent issues, the operator should:
   - Review the contract's gas and error rate trends.
   - Check for unusual `AuthFailure` or `AnomalyDetected` events.
   - Escalate if `Critical` events accumulate without resolution.

## CI Validation

The `ci.yml` workflow includes a `telemetry-schema-check` job that:
1. Verifies all `TelemetryEvent` structs include the required `schema_version` field.
2. Checks that new event types have an assigned `EventClass`.
3. Ensures `SCHEMA_VERSION_MAJOR` is bumped for any breaking field changes.

## Migration Guide

When upgrading from one schema version to another:

1. Bump `SCHEMA_VERSION_MINOR` (additive) or `SCHEMA_VERSION_MAJOR` (breaking) in `telemetry.rs`.
2. Append new optional fields at the end of `TelemetryEvent` or `TelemetrySnapshot`.
3. Old consumers will ignore unknown trailing fields (forward-compatible).
4. New consumers must handle missing optional fields gracefully.
5. Document the change in `docs/TELEMETRY_SCHEMA.md`.

### Migrating 1.0.0 → 2.0.0

Version 2.0.0 is a **breaking** release: `TelemetryEvent` replaced the meaning of its trace-linkage with a new required `trace_id` field.

- Emitters must now call `record_call` / `record_error` with a `trace_id` derived once by the top-level submitter (`telemetry::derive_trace_id`) and forwarded down the call chain.
- Consumers must parse the new `trace_id` field and group by it. The legacy `correlation_id` remains present but is explicitly not a trace key.
