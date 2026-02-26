#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 8 ]]; then
  echo "Usage: $0 <contract_id> <network> <source_account> <admin_address> <arbiter_address> <oracle_address> <challenger_address> <base_timestamp>"
  echo "Example: $0 CABC... testnet alice GADMIN... GARB... GORACLE... GUSER... 1731800000"
  exit 1
fi

CONTRACT_ID="$1"
NETWORK="$2"
SOURCE_ACCOUNT="$3"
ADMIN_ADDRESS="$4"
ARBITER_ADDRESS="$5"
ORACLE_ADDRESS="$6"
CHALLENGER_ADDRESS="$7"
BASE_TS="$8"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INTERACT_SCRIPT="$SCRIPT_DIR/interact_healthcare_oracle_network.sh"

if [[ ! -f "$INTERACT_SCRIPT" ]]; then
  echo "Missing helper script: $INTERACT_SCRIPT"
  exit 1
fi

run() {
  "$INTERACT_SCRIPT" "$CONTRACT_ID" "$NETWORK" "$SOURCE_ACCOUNT" "$@"
}

echo "[1/7] Registering oracle source"
run register_oracle \
  operator="$ORACLE_ADDRESS" \
  endpoint="https://oracle.demo.health" \
  source_type=4 >/dev/null

echo "[2/7] Verifying oracle source"
run verify_oracle \
  admin="$ADMIN_ADDRESS" \
  operator="$ORACLE_ADDRESS" \
  verified=true \
  active=true >/dev/null

echo "[3/7] Submitting drug pricing update"
run submit_drug_price \
  operator="$ORACLE_ADDRESS" \
  feed_id="NDC:55513-1234-1:KE" \
  ndc_code="55513-1234-1" \
  currency="USD" \
  price_minor=1050 \
  availability_units=220 \
  observed_at="$BASE_TS" >/dev/null

echo "[4/7] Submitting treatment outcome update"
run submit_treatment_outcome \
  operator="$ORACLE_ADDRESS" \
  outcome_id="OUTCOME:CHF:ACEI:2026Q1" \
  condition_code="I50.9" \
  treatment_code="ACEI" \
  improvement_rate_bps=7100 \
  readmission_rate_bps=950 \
  mortality_rate_bps=180 \
  sample_size=1200 \
  reported_at="$((BASE_TS + 10))" >/dev/null

echo "[5/7] Submitting clinical trial and regulatory updates"
run submit_clinical_trial \
  operator="$ORACLE_ADDRESS" \
  trial_id="NCT-2026-001" \
  phase=3 \
  enrolled=450 \
  success_rate_bps=8200 \
  adverse_event_rate_bps=600 \
  result_hash="sha256:trial-a" \
  published_at="$((BASE_TS + 20))" >/dev/null

run submit_regulatory_update \
  operator="$ORACLE_ADDRESS" \
  regulation_id="FDA-2026-DRUG-UPDATE-11" \
  authority=1 \
  status=4 \
  title="Updated-Labeling-Requirement" \
  details_hash="sha256:reg-update-11" \
  effective_at="$((BASE_TS + 30))" >/dev/null

echo "[6/7] Raising dispute on regulatory feed"
DISPUTE_RAW=$(run raise_dispute \
  challenger="$CHALLENGER_ADDRESS" \
  kind=3 \
  feed_id="FDA-2026-DRUG-UPDATE-11" \
  reason="Source-mismatch")

DISPUTE_ID=$(echo "$DISPUTE_RAW" | grep -Eo '[0-9]+' | tail -n1)
if [[ -z "${DISPUTE_ID:-}" ]]; then
  echo "Could not parse dispute id from output: $DISPUTE_RAW"
  exit 1
fi

echo "Parsed dispute id: $DISPUTE_ID"

echo "[7/7] Resolving dispute with arbiter ruling"
run resolve_dispute \
  resolver="$ARBITER_ADDRESS" \
  dispute_id="$DISPUTE_ID" \
  valid_dispute=true \
  ruling="Confirmed-mismatch" \
  penalized_oracle="$ORACLE_ADDRESS" >/dev/null

echo "Scenario completed successfully."
echo "Contract: $CONTRACT_ID"
echo "Dispute ID: $DISPUTE_ID"
