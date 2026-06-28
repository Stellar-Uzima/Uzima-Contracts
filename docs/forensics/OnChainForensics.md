# On-Chain Forensics Contract Architecture

## Overview
The On-Chain Forensics contract provides a robust framework for transaction pattern analysis, suspicious activity detection, and immutable evidence collection on the Stellar network.

## Components

### 1. Pattern Analyzer
Analyzes incoming activities against known attack patterns and identifies potential threats.
- **Pattern Tracking**: Tracks occurrences and risk scores for specialized behaviors.
- **Adaptive Scoring**: Increases risk metrics dynamically based on suspicious feedback loops.

### 2. Suspicious Detector
Evaluates actors and provides real-time threat level classification.
- **Blacklisting**: Prevents bad actors from further interactions.
- **Threat Levels**: classify events from None to Critical.

### 3. Evidence Collector
Ensures all forensic data is preserved immutably in Soroban Storage.
- **Immutable Proofs**: Stores evidence data with associated hashes and metadata.
- **Discovery Tools**: Allows investigators to query and retrieve evidence by Case ID.

### 4. Forensic Reporting
Allows administrators to generate structured investigation reports.
- **Audit Reports**: Comprehensive summaries of forensic findings.
- **Transparency**: All administrative actions and report generation are logged on-chain.

## Performance
- **Forensic analysis targets**: < 150k gas (resource equivalent).
- **Scalability**: Designed for high-volume activity monitoring with minimal state storage impact.

## Security
- All sensitive functions (blacklist, reporting, config) require **admin authorization**.
- Evidence is stored in **persistent storage** to survive ledger entry expirations.
