# Telemetry Schema & Incident Response Framework

This document defines the versioned on-chain telemetry event schema and incident-response workflow for the Uzima monitoring contract.

## Overview

The monitoring contract emits structured telemetry events that off-chain systems can parse to build dashboards, trigger alerts, and conduct incident response. Every event carries a **schema version** for forward/backward compatibility, a **correlation ID** to link related events, and a **classification** that separates routine operations from security-relevant anomalies.

## Event Schema

Each telemetry event is a `TelemetryEvent` struct emitted with topic `(TEL, <type_symbol>)`.

| Field | Type | Description |
|---|---|---|
| `schema_version` | `u32` | Packed semver `MAJOR*10000 + MINOR*100 + PATCH` |
| `correlation_id` | `BytesN<32>` | Links events across a transaction chain |
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

- **MAJOR** (breaking): removed or reordered fields — all consumers must update.
- **MINOR** (additive): new optional fields appended at the end — consumers can ignore.
- **PATCH** (fixes): bug fixes, documentation, no schema change.

Current version: **1.0.0** (`schema_version = 10000`).

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

## Correlation IDs

Every event includes a `correlation_id` derived from the caller address and ledger timestamp. Off-chain systems should group events by correlation ID to reconstruct a full execution trace for a given transaction or call chain.

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
4. Events with the same `correlation_id` are correlated to trace the root cause.
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
