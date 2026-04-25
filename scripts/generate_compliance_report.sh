#!/usr/bin/env bash
# Simple helper script to generate a compliance report by querying the healthcare_compliance contract
# Requires: soroban CLI configured and network set (local/testnet)
# Usage: ./scripts/generate_compliance_report.sh <NETWORK> <COMPLIANCE_CONTRACT_ID> <OUTPUT_DIR>

set -euo pipefail

NETWORK=${1:-local}
CONTRACT_ID=${2:-}
OUT_DIR=${3:-deployments}

if [ -z "$CONTRACT_ID" ]; then
  echo "Usage: $0 <network> <compliance_contract_id> [output_dir]"
  exit 1
fi

mkdir -p "$OUT_DIR"
TIMESTAMP=$(date --utc +%Y%m%dT%H%M%SZ)
OUT_FILE="$OUT_DIR/compliance_report_${TIMESTAMP}.json"

# Fetch compliance metrics (this uses soroban CLI - replace with your configured command if needed)
# The following commands are placeholders showing how to call the contract. Replace CONTRACT_ID and method names as needed.

echo "Generating compliance report for contract $CONTRACT_ID on network $NETWORK"

# Get metrics
soroban contract invoke --network "$NETWORK" --id "$CONTRACT_ID" --fn get_compliance_metrics > /tmp/metrics_raw.json || true

# Get a sample audit log batch (limit 50). If your CLI supports passing args, set the limit accordingly.
soroban contract invoke --network "$NETWORK" --id "$CONTRACT_ID" --fn get_audit_logs --args '[]' > /tmp/audits_raw.json || true

# Build combined report
jq -n --slurpfile metrics /tmp/metrics_raw.json --slurpfile audits /tmp/audits_raw.json '{generated_at: "'"$TIMESTAMP"'", metrics: $metrics[0], audits: $audits[0]}' > "$OUT_FILE" || echo "Could not create JSON with jq; copying raw outputs instead" && cat /tmp/metrics_raw.json > "$OUT_FILE"

echo "Report written to $OUT_FILE"
