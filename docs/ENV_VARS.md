# Environment Variables and Secrets for Scripts

This document audits environment variables referenced by files under `scripts/`.
Use it as the checklist for local development, CI jobs, and deployment automation.

Safe examples below are placeholders only. Never commit real keys, mnemonics, private keys, API tokens, or production RPC credentials.

## Summary

- Script files scanned: 94
- Environment variables found: 55
- Scope: `scripts/**/*.sh`, `scripts/**/*.js`, `scripts/**/*.mjs`, `scripts/**/*.ts`, `scripts/**/*.py`, and `scripts/**/*.ps1`

## Variables

| Variable | Category | Required? | Used by | Safe example |
| --- | --- | --- | --- | --- |
| `ALERT_ON_FAILURE` | Runtime/config | Optional/defaulted | `scripts/check_release_health.sh` | `example-value` |
| `ANALYTICS_MODEL_ID` | Contract/deployment | Optional/defaulted | `scripts/analytics_dashboard.ts`<br>`scripts/cross_institution_analytics.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `ANOMALY_DETECTION_ID` | Contract/deployment | Optional/defaulted | `scripts/analytics_dashboard.ts`<br>`scripts/cross_institution_analytics.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `CARGO_TARGET_DIR` | Runtime/config | Optional/defaulted | `scripts/measure_storage.sh` | `./path/to/value` |
| `CI` | Runtime/config | Optional/defaulted | `scripts/network_manager.sh` | `true` |
| `CONTRACT_ID` | Contract/deployment | Required when running listed script(s) | `scripts/monitor_health.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `CONTRACT_NAME` | Contract/deployment | Required when running listed script(s) | `scripts/deploy_all.sh` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `DID_CONTRACT_ID` | Contract/deployment | Optional/defaulted | `scripts/did_management.sh` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `DISCORD_WEBHOOK_URL` | Secret | Optional/defaulted | `scripts/announce_release.sh` | `REDACTED_EXAMPLE_VALUE` |
| `DRY_RUN` | Runtime/config | Optional/defaulted | `scripts/announce_release.sh`<br>`scripts/publish_artifacts.sh`<br>`scripts/release.sh` | `true` |
| `EMR_INTEGRATION_ID` | Contract/deployment | Optional/defaulted | `scripts/cross_institution_analytics.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `EXPLAINABLE_AI_ID` | Contract/deployment | Optional/defaulted | `scripts/analytics_dashboard.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `FEDERATED_LEARNING_ID` | Contract/deployment | Optional/defaulted | `scripts/analytics_dashboard.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `FEDERATED_ROUND_ID` | Contract/deployment | Optional/defaulted | `scripts/analytics_dashboard.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `FHIR_INTEGRATION_ID` | Contract/deployment | Optional/defaulted | `scripts/cross_institution_analytics.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `FORMAT` | Runtime/config | Optional/defaulted | `scripts/generate_changelog.sh`<br>`scripts/generate_release_notes.sh` | `example-value` |
| `FROM_VERSION` | Runtime/config | Optional/defaulted | `scripts/generate_changelog.sh`<br>`scripts/generate_release_notes.sh` | `example-value` |
| `FUNC` | Runtime/config | Required when running listed script(s) | `scripts/profile.sh` | `example-value` |
| `GENERATOR_VERSION` | Runtime/config | Required when running listed script(s) | `scripts/generate-sdk-types.mjs` | `example-value` |
| `HEALTH_CHECK_TIMEOUT` | Runtime/config | Optional/defaulted | `scripts/check_release_health.sh` | `example-value` |
| `IDENTITY` | Contract/deployment | Optional/defaulted | `scripts/did_management.sh` | `example-value` |
| `INCLUDE_AUTHORS` | Runtime/config | Optional/defaulted | `scripts/generate_release_notes.sh` | `example-value` |
| `INCLUDE_METRICS` | Runtime/config | Optional/defaulted | `scripts/generate_release_notes.sh` | `example-value` |
| `INCLUDE_STATS` | Runtime/config | Optional/defaulted | `scripts/generate_release_notes.sh` | `example-value` |
| `INTERACTIVE` | Runtime/config | Optional/defaulted | `scripts/generate_changelog.sh` | `example-value` |
| `MEDICAL_RECORDS_ID` | Contract/deployment | Optional/defaulted | `scripts/analytics_dashboard.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `NETWORK` | Network/config | Optional/defaulted | `scripts/check_release_health.sh`<br>`scripts/deploy_identity_registry.sh`<br>`scripts/did_management.sh` | `testnet` |
| `NETWORK_PASSPHRASE` | Network/config | Optional/defaulted | `scripts/analytics_dashboard.ts`<br>`scripts/cross_institution_analytics.ts` | `testnet` |
| `OUTPUT_FILE` | Runtime/config | Optional/defaulted | `scripts/generate_release_notes.sh` | `./path/to/value` |
| `OUTPUT_HTML` | Runtime/config | Required when running listed script(s) | `scripts/docs/generate.mjs` | `example-value` |
| `OUTPUT_JSON` | Runtime/config | Required when running listed script(s) | `scripts/docs/generate.mjs` | `example-value` |
| `OUTPUT_MD` | Runtime/config | Required when running listed script(s) | `scripts/docs/generate.mjs` | `example-value` |
| `PREDICTIVE_ANALYTICS_ID` | Contract/deployment | Optional/defaulted | `scripts/analytics_dashboard.ts`<br>`scripts/cross_institution_analytics.ts` | `CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` |
| `PUBLISH_CRATES` | Runtime/config | Optional/defaulted | `scripts/publish_artifacts.sh` | `true` |
| `PUBLISH_DOCKER` | Runtime/config | Optional/defaulted | `scripts/publish_artifacts.sh` | `true` |
| `PUBLISH_GITHUB` | Runtime/config | Optional/defaulted | `scripts/publish_artifacts.sh` | `true` |
| `PUBLISH_NPM` | Runtime/config | Optional/defaulted | `scripts/publish_artifacts.sh` | `true` |
| `REPORT_PATH` | Runtime/config | Required when running listed script(s) | `scripts/generate-sdk-types.mjs` | `./path/to/value` |
| `RPC_URL` | Network/config | Optional/defaulted | `scripts/analytics_dashboard.ts`<br>`scripts/cross_institution_analytics.ts` | `https://example.invalid` |
| `SKIP_SECURITY` | Runtime/config | Optional/defaulted | `scripts/validate_release.sh` | `true` |
| `SKIP_TESTS` | Runtime/config | Optional/defaulted | `scripts/validate_release.sh` | `true` |
| `SLACK_WEBHOOK` | Secret | Optional/defaulted | `scripts/release.sh` | `REDACTED_EXAMPLE_VALUE` |
| `SLACK_WEBHOOK_URL` | Secret | Optional/defaulted | `scripts/announce_release.sh`<br>`scripts/check_release_health.sh` | `REDACTED_EXAMPLE_VALUE` |
| `SMTP_HOST` | Runtime/config | Optional/defaulted | `scripts/announce_release.sh` | `example-value` |
| `SMTP_PASS` | Runtime/config | Optional/defaulted | `scripts/announce_release.sh` | `example-value` |
| `SMTP_USER` | Runtime/config | Optional/defaulted | `scripts/announce_release.sh` | `example-value` |
| `SOROBAN_IDENTITY` | Contract/deployment | Optional/defaulted | `scripts/advanced_cli.sh`<br>`scripts/interact.sh` | `example-value` |
| `STRICT` | Runtime/config | Optional/defaulted | `scripts/validate_release.sh` | `true` |
| `TO_VERSION` | Runtime/config | Optional/defaulted | `scripts/generate_changelog.sh`<br>`scripts/generate_release_notes.sh` | `example-value` |
| `TWITTER_ACCESS_TOKEN` | Secret | Optional/defaulted | `scripts/announce_release.sh` | `REDACTED_EXAMPLE_VALUE` |
| `TWITTER_ACCESS_TOKEN_SECRET` | Secret | Optional/defaulted | `scripts/announce_release.sh` | `REDACTED_EXAMPLE_VALUE` |
| `TWITTER_API_KEY` | Secret | Optional/defaulted | `scripts/announce_release.sh` | `REDACTED_EXAMPLE_VALUE` |
| `TWITTER_API_SECRET` | Secret | Optional/defaulted | `scripts/announce_release.sh` | `REDACTED_EXAMPLE_VALUE` |
| `VERSION` | Runtime/config | Optional/defaulted | `scripts/generate_changelog.sh`<br>`scripts/generate_release_notes.sh` | `example-value` |
| `WEBHOOK_URL` | Secret | Optional/defaulted | `scripts/monitor_deployments.sh` | `REDACTED_EXAMPLE_VALUE` |

## Secret-handling guidelines

- Store secrets in your shell, CI secret store, or deployment platform secret manager; do not write them into tracked files.
- Prefer least-privilege tokens for deployment, monitoring, and release scripts.
- Use placeholder values like `REDACTED_EXAMPLE_VALUE` in examples and tests.
- Rotate any secret that was printed to logs, committed, or shared in a PR comment.
- For CI, define only the variables needed by the specific job running the script.

## Maintenance

When adding or changing scripts, update the table above in the same PR. A quick way to re-audit is to search for `process.env`, `env::var`, `$VAR`, and `${VAR}` under `scripts/`.
