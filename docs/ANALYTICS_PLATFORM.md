# Uzima Analytics Platform

## Overview

This document describes the on-chain and off-chain components that make up the Uzima medical analytics platform, and how to run the provided tools to obtain privacy-preserving dashboards and reports.

The platform builds on existing smart contracts:

- `medical_records`
- `federated_learning`
- `anomaly_detection`
- `predictive_analytics`
- `explainable_ai`
- Healthcare integration contracts (FHIR / EMR / conversion)

## On-Chain Analytics Primitives

### Medical Records

- Stores medical records with hashed/off-chain references, categories, and tags.
- Emits rich events for user management, record creation and access, AI integration, recovery, and system monitoring.
- Provides AI integration endpoints for anomaly and risk scores that rely on privacy-aware configuration.

### Federated Learning

- Manages training rounds and participant updates.
- Enforces differential privacy budgets via `PrivacyBudget` and DP epsilon per round.
- Stores `ModelMetadata` for trained models, including links to metrics and fairness reports.

### Anomaly Detection

- Records anomalies as `AnomalyRecord` with severity and explanation references.
- Maintains aggregate `DetectionStats` and per-patient anomaly counts for efficient queries.

### Predictive Analytics

- Stores per-patient `HealthPrediction` records with outcome types, confidence, and risk factors.
- Maintains `PatientPredictionsSummary` and `PredictionMetrics` per model to support O(1) queries.
- Includes thresholds on confidence to avoid exposing low-quality predictions.

### Explainable AI

- Stores explanation requests and `ExplanationMetadata` for AI outputs.
- Stores `BiasAuditResult` for models and supports fairness metric evaluation.

## Off-Chain Analytics Layer

Analytics queries and dashboards are executed off-chain by calling read-only contract methods and processing events from the Soroban network. This layer is responsible for:

- Aggregating health, anomaly, predictive, and federated learning metrics.
- Combining per-contract statistics into dashboards.
- Enforcing additional organizational privacy policies (e.g., minimum cohort size, suppression of small groups).

The primary entrypoint for this layer in this repository is:

- `scripts/analytics_dashboard.ts`

## Analytics Dashboard Script

### Prerequisites

- Node.js and `npx` available.
- Dependencies installed (from project root):

```bash
npm install
```

- Access to a Soroban RPC endpoint (default: `https://soroban-testnet.stellar.org`).

### Configuration

The script reads configuration from environment variables and defaults:

- `RPC_URL` – Soroban RPC URL (default: testnet).
- `NETWORK_PASSPHRASE` – Network passphrase (default: `Networks.TESTNET`).
- `MEDICAL_RECORDS_ID` – Contract ID of the deployed `medical_records` contract.
- `ANOMALY_DETECTION_ID` – Contract ID of `anomaly_detection` (optional).
- `PREDICTIVE_ANALYTICS_ID` – Contract ID of `predictive_analytics` (optional).
- `FEDERATED_LEARNING_ID` – Contract ID of `federated_learning` (optional).
- `EXPLAINABLE_AI_ID` – Contract ID of `explainable_ai` (optional).
- `ANALYTICS_MODEL_ID` – 32-byte model identifier as 64-character hex, matching on-chain `BytesN<32>` for model metrics and bias audits (optional).
- `FEDERATED_ROUND_ID` – Round identifier (u64) for federated learning analytics (optional).

### Running the Dashboard

From the project root:

```bash
npx ts-node scripts/analytics_dashboard.ts --format table
```

This will:

- Call `health_check` on `medical_records` (if configured) to obtain liveness and basic telemetry.
- Call `get_stats` on `anomaly_detection` (if configured) to obtain anomaly counts and last detection time.
- Call `get_model_metrics` on `predictive_analytics` (if both contract and model ID are configured).
- Call `get_round` on `federated_learning` for the configured round (if available).
- Call `get_bias_audit` on `explainable_ai` for the configured model (if available).

To obtain machine-readable output suitable for piping into external dashboards:

```bash
npx ts-node scripts/analytics_dashboard.ts --format json
```

The JSON output is normalized so that 64-bit integers (usually returned as `bigint`) are converted to strings.

## Privacy-Preserving Analytics

The analytics platform is designed to preserve patient privacy while providing useful aggregate insights:

- **No raw PHI on-chain** – Contracts store hashes, identifiers, and aggregate metrics rather than raw medical content.
- **Differential privacy budgets** – Federated learning enforces privacy budgets per participant using `PrivacyBudget` and DP epsilon settings.
- **Thresholds and minimum confidence** – Anomaly detection and predictive analytics enforce thresholds to avoid exposing noisy or low-confidence outputs.
- **Aggregate queries only** – Analytics scripts consume aggregate statistics (`DetectionStats`, `PredictionMetrics`, round metadata) instead of reconstructing individual trajectories.
- **Explainability with safeguards** – Explainable AI provides explanations and bias audits without exposing raw feature vectors.

Off-chain consumers of these analytics should:

- Avoid aggregations over very small cohorts.
- Apply additional noise or suppression policies where required by local law or organizational policy.

## Cross-Institution Analytics

Cross-institution metrics can be built by combining:

- EMR and FHIR integration contracts (for provider/institution identity and interoperability).
- Aggregated anomaly detection and predictive metrics keyed by providers or networks.

The same privacy guarantees apply: use counts, rates, and differentially private aggregates, and avoid exporting detailed per-patient histories.

## Performance Considerations

The on-chain contracts are designed to support analytics efficiently:

- Aggregates such as `DetectionStats`, `PatientPredictionsSummary`, `PredictionMetrics`, and `PrivacyBudget` provide O(1) access patterns.
- No contract introduces loops over unbounded maps for analytics queries.
- Event emission is used to power higher-volume, off-chain indexing when needed.

For large deployments:

- Use an off-chain indexer to store and query events and contract state snapshots.
- Use the `--format json` mode of `analytics_dashboard.ts` as a lightweight health and metrics probe, or as a source for external dashboards (Grafana, Prometheus exporters, etc.).
