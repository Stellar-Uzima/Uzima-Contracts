#!/usr/bin/env bash
# Helper script to generate a compliance report by querying the healthcare_compliance contract
# Requires: soroban CLI configured and network set (local/testnet/futurenet/mainnet)
# Usage: ./scripts/generate_compliance_report.sh <NETWORK> <COMPLIANCE_CONTRACT_ID> [OUTPUT_DIR]
#
# Contract IDs may also be supplied via environment variables so the script
# runs against configured contracts without manual edits:
#   COMPLIANCE_CONTRACT_ID  - healthcare_compliance contract ID (used if arg 2 is omitted)
#   NETWORK                 - network name (used if arg 1 is omitted)
#   COMPLIANCE_METRICS_FN   - contract method returning metrics (default: get_compliance_metrics)
#   COMPLIANCE_AUDIT_FN     - contract method returning audit logs (default: get_audit_logs)
#   COMPLIANCE_AUDIT_ARGS   - JSON args passed to the audit method (default: [])

set -euo pipefail

NETWORK=${1:-${NETWORK:-local}}
CONTRACT_ID=${2:-${COMPLIANCE_CONTRACT_ID:-}}
OUT_DIR=${3:-deployments}

METRICS_FN=${COMPLIANCE_METRICS_FN:-get_compliance_metrics}
AUDIT_FN=${COMPLIANCE_AUDIT_FN:-get_audit_logs}
AUDIT_ARGS=${COMPLIANCE_AUDIT_ARGS:-[]}

if [ -z "$CONTRACT_ID" ]; then
  echo "Usage: $0 <network> <compliance_contract_id> [output_dir]"
  echo "Or set COMPLIANCE_CONTRACT_ID (and optionally NETWORK) in the environment."
  exit 1
fi

if ! command -v soroban >/dev/null 2>&1; then
  echo "Error: soroban CLI not found in PATH" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
TIMESTAMP=$(date --utc +%Y%m%dT%H%M%SZ)
OUT_FILE="$OUT_DIR/compliance_report_${TIMESTAMP}.json"
METRICS_RAW=$(mktemp)
AUDITS_RAW=$(mktemp)
trap 'rm -f "$METRICS_RAW" "$AUDITS_RAW"' EXIT

echo "Generating compliance report for contract $CONTRACT_ID on network $NETWORK"

# Fetch compliance metrics from the contract
soroban contract invoke --network "$NETWORK" --id "$CONTRACT_ID" --fn "$METRICS_FN" > "$METRICS_RAW"

# Fetch audit log batch from the contract
soroban contract invoke --network "$NETWORK" --id "$CONTRACT_ID" --fn "$AUDIT_FN" --args "$AUDIT_ARGS" > "$AUDITS_RAW"

# Build combined report; fail loudly rather than writing partial data
jq -n \
  --arg generated_at "$TIMESTAMP" \
  --slurpfile metrics "$METRICS_RAW" \
  --slurpfile audits "$AUDITS_RAW" \
  '{generated_at: $generated_at, metrics: $metrics[0], audits: $audits[0]}' > "$OUT_FILE"

echo "Report written to $OUT_FILE"
