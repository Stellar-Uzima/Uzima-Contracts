# Monitoring & Metrics System

## Overview
The Medical Records contract includes built-in telemetry for health and performance monitoring.

## 1. Health Check Endpoint
**Function:** `health_check()`
- **Returns:** `(Status: Symbol, Version: u32, Timestamp: u64)`
- **Usage:** Call this read-only function every 5 minutes to verify network connectivity and contract liveness.
- **Expected Output:** `["OK", 1, 170603...]`

## 2. Key Metrics
The contract emits standard Soroban events for key business operations.

| Metric Name | Event Topic | Trigger |
|:---|:---|:---|
| **Record Creation** | `("METRICS", "add_rec")` | Emitted whenever `add_record` is called successfully. |
| **System Health** | `("METRICS", "health")` | Derived from the health check endpoint. |

## 3. Alerts
Recommended alert thresholds for the monitoring script:
- **Liveness:** Alert if `health_check` fails or times out > 3 times in a row.
- **Gas Usage:** Alert if `add_record` CPU cost exceeds 3,000,000 instructions.

## 4. Running the Monitor
Use the included script to check liveness:
```bash
npx ts-node scripts/monitor_health.ts