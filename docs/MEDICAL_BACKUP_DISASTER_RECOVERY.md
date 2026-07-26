# Medical Backup Disaster Recovery (DR) Runbook

This document outlines the procedures for responding to a disaster event affecting the multi-region deployment of the Uzima medical records system.

## 1. Event Detection and Alerting

- **Automated Detection**: The `FailoverDetector` contract continuously monitors node health via heartbeats and other metrics.
- **Alerting**: Critical failures (e.g., 3 consecutive missed heartbeats) trigger an on-chain event. Off-chain monitoring services should subscribe to these events and alert the on-call engineering team via PagerDuty or a similar service.

## 2. Failover Procedure

1.  **Acknowledge Alert**: The on-call engineer acknowledges the alert.
2.  **Verify Failure**: The engineer uses the `FailoverDetector` contract's `get_detections` and `get_node_metrics` functions to verify the failure's criticality.
3.  **Initiate Failover**:
    *   For non-primary nodes, the system is designed to automatically recover.
    *   For primary region failures, the `MultiRegionOrchestrator`'s `promote_new_primary` function must be called. This is a manual step to ensure human oversight.
4.  **Execute Failover**: The on-call engineer with the `OPERATOR` role calls `promote_new_primary`, specifying the region ID of the healthiest secondary region.
5.  **Monitor Promotion**: The engineer monitors the `RegionPromoted` event to confirm the successful promotion of the new primary.

## 3. Post-Failover Verification

1.  **Data Consistency Check**: The engineer runs a script to query the `SyncManager` contract and verify that the `data_hash` of recent transactions is consistent across the new primary and its replicas.
2.  **System Health Check**: The engineer uses the `RegionalNodeManager` to check the status of all nodes and ensure the system has stabilized.
3.  **Application-Level Checks**: The application team performs checks to ensure the user-facing services are fully functional.

## 4. Post-Incident Review

- A post-incident review meeting should be held within 48 hours of the event.
- The review should cover the root cause of the failure, the effectiveness of the DR response, and any improvements needed for the runbook or the DR system itself.
- Action items from the review should be tracked to completion.