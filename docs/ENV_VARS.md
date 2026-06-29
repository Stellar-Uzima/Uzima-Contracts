# Environment Variables & Secrets

This document lists all environment variables used by scripts in the `scripts/` directory.
Variables marked **Secret** contain sensitive values and must never be committed or logged.

## Table

| Variable | Script(s) | Description | Required | Secret |
|---|---|---|---|---|
| `NETWORK` | `deploy_*.sh`, `interact.sh`, `did_management.sh` | Stellar network target (`local`, `testnet`, `futurenet`, `mainnet`) | Yes | No |
| `IDENTITY` | `did_management.sh`, `deploy_environment.sh`, `deploy_did_registry.sh` | Stellar CLI identity name used as the source account | Depends | No |
| `SOROBAN_IDENTITY` | `interact.sh` | Fallback identity when no explicit arg is passed (defaults to `"default"`) | No | No |
| `RPC_URL` | `analytics_dashboard.ts`, `cross_institution_analytics.ts` | Soroban RPC endpoint URL | Yes\* | No |
| `NETWORK_PASSPHRASE` | `analytics_dashboard.ts`, `cross_institution_analytics.ts` | Network passphrase for transaction signing | Yes\* | No |
| `MEDICAL_RECORDS_ID` | `analytics_dashboard.ts` | Deployed MedicalRecords contract ID | Yes\* | No |
| `ANOMALY_DETECTION_ID` | `analytics_dashboard.ts`, `cross_institution_analytics.ts` | Deployed AnomalyDetection contract ID | No | No |
| `PREDICTIVE_ANALYTICS_ID` | `analytics_dashboard.ts`, `cross_institution_analytics.ts` | Deployed PredictiveAnalytics contract ID | No | No |
| `FEDERATED_LEARNING_ID` | `analytics_dashboard.ts` | Deployed FederatedLearning contract ID | No | No |
| `EXPLAINABLE_AI_ID` | `analytics_dashboard.ts` | Deployed ExplainableAI contract ID | No | No |
| `FHIR_INTEGRATION_ID` | `cross_institution_analytics.ts` | Deployed FhirIntegration contract ID | Yes\* | No |
| `EMR_INTEGRATION_ID` | `cross_institution_analytics.ts` | Deployed EmrIntegration contract ID | Yes\* | No |
| `ANALYTICS_MODEL_ID` | `analytics_dashboard.ts`, `cross_institution_analytics.ts` | 32-byte hex model ID (`BytesN<32>`) | No | No |
| `FEDERATED_ROUND_ID` | `analytics_dashboard.ts` | Federated learning round number | No | No |
| `PROVIDER_IDS` | `cross_institution_analytics.ts` | Comma-separated list of logical provider IDs | No | No |
| `NETWORK_NODE_IDS` | `cross_institution_analytics.ts` | Comma-separated list of network node IDs | No | No |
| `DID_CONTRACT_ID` | `did_management.sh` | Deployment-auto-detected; override when not using deployments/ directory | No | No |
| `OWNER_ADDRESS` | `deploy_identity_registry.sh` | Stellar address of the contract owner (positional arg also accepted) | Yes | No |
| `ADMIN_ADDRESS` | `deploy_healthcare_integration.sh`, `deploy_healthcare_oracle_network.sh` | Admin Stellar address for contract initialization | Yes | No |
| `ARBITER_ADDRESS` | `deploy_healthcare_oracle_network.sh` | Arbiter Stellar address for dispute resolution | Yes | No |
| `MIN_SUBMISSIONS` | `deploy_healthcare_oracle_network.sh` | Minimum oracle submissions threshold (default: 2) | No | No |

> \* — Required when the corresponding feature/module is used; otherwise optional.

## Notes

- Variables marked **Required** must be set before running the script.
- Never hard-code secrets (private keys, mnemonics) in scripts. Use environment variables or secure secret stores.
- For local development a `.env` file can be used; ensure it is listed in `.gitignore`.
- All `deploy_*.sh` scripts accept the network as the first positional argument.
- TypeScript scripts (`*.ts`) pull configuration from `process.env`; RPC URL and network passphrase must always be correct for the target network.
