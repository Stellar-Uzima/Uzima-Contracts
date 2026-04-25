# Anti-Money Laundering (AML) Contract Architecture

## Overview
The AML contract provides a comprehensive mechanism for transaction monitoring, risk assessment, and enforcement to ensure regulatory compliance across the Uzima-Contracts ecosystem.

## Performance
- **AML Monitoring Checks**: target < **120k gas** (resource equivalent).
- **Scalability**: High-throughput monitoring using persistent risk profiling.

## Core Features

### 1. Adaptive Risk Scoring
Users are assigned a dynamic risk score between **0 and 10000** (bps). 
- Scores are updated in real-time based on transaction velocity, volume, and known risk indicators.
- Factors include: Frequent small transactions (structuring), large single transfers, and proximity to high-risk actors.

### 2. Rule Enforcement
A modular rule engine allows administrators to define AML criteria.
- **Thresholds**: Limits for specific transaction sizes.
- **Velocity Rules**: Detections for too many transactions in a given window.
- **Automatic Block**: Users crossing the critical risk threshold (9000+) are automatically blacklisted.

### 3. Blacklist Management
A robust on-chain blacklist prevents sanctioned or non-compliant actors from participating in the ecosystem.
- Changes require admin authorization.
- Integrates with the Forensics contract for evidence-backed sanctions.

### 4. Regulatory Integration
Provides analytical reports and historical transaction summaries for regulatory auditing.
- `report_incident`: On-chain record of specific compliance violations.
- `is_compliant`: Interface for other contracts to check user status before sensitive operations.

## Data Structures

### `RiskProfile`
- **User**: The unique participant address.
- **Risk Score**: Calculated based on historical and current behaviors.
- **Violation Count**: Cumulative count of tripped AML rules.
- **Is Blacklisted**: Operational status reflecting compliance standing.

## Integration
Contracts handling fund movements or identity registration must integrate with `monitor_transaction` and `is_compliant` to ensure full AML coverage.
