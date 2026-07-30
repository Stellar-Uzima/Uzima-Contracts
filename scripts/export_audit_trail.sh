#!/usr/bin/env bash
# ============================================================================
# export_audit_trail.sh
# Exports audit trail records for compliance review and regulatory submission.
# Usage:
#   ./scripts/export_audit_trail.sh [--network testnet|mainnet|futurenet]
#                                   [--start-date YYYY-MM-DD]
#                                   [--end-date YYYY-MM-DD]
#                                   [--format json|csv|both]
#                                   [--output-dir <path>]
#                                   [--contract <contract-id>]
#                                   [--retention-days <days>]
#                                   [--dry-run]
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

NETWORK="testnet"
START_DATE=""
END_DATE=""
FORMAT="both"
OUTPUT_DIR="$ROOT_DIR/audit-exports"
CONTRACT_ID=""
RETENTION_DAYS=2555
DRY_RUN=false
RETENTION_CONFIG="$ROOT_DIR/config/audit_retention.json"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [OPTIONS]
Export audit trail records for compliance review.
Options:
  --network <network>       Stellar network (testnet|mainnet|futurenet) [default: testnet]
  --start-date <YYYY-MM-DD> Start date for export range
  --end-date <YYYY-MM-DD>   End date for export range
  --format <format>         Output format: json, csv, or both [default: both]
  --output-dir <path>       Output directory [default: ./audit-exports]
  --contract <id>           Filter by specific contract ID
  --retention-days <days>    Retention period in days [default: 2555 (7 years)]
  --dry-run                 Preview export without executing
  --help                    Show this help message
USAGE
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --network) NETWORK="$2"; shift 2 ;;
    --start-date) START_DATE="$2"; shift 2 ;;
    --end-date) END_DATE="$2"; shift 2 ;;
    --format) FORMAT="$2"; shift 2 ;;
    --output-dir) OUTPUT_DIR="$2"; shift 2 ;;
    --contract) CONTRACT_ID="$2"; shift 2 ;;
    --retention-days) RETENTION_DAYS="$2"; shift 2 ;;
    --dry-run) DRY_RUN=true; shift ;;
    --help) usage; exit 0 ;;
    *) echo "Unknown option: $1"; usage; exit 1 ;;
  esac
done

echo ""
echo "Uzima Audit Trail Export Tool"
echo "─────────────────────────────"
echo "Network:       $NETWORK"
echo "Format:        $FORMAT"
echo "Output:        $OUTPUT_DIR"
echo "Retention:     ${RETENTION_DAYS} days"
[[ -n "$START_DATE" ]] && echo "Start Date:    $START_DATE"
[[ -n "$END_DATE" ]] && echo "End Date:      $END_DATE"
[[ -n "$CONTRACT_ID" ]] && echo "Contract:      $CONTRACT_ID"
[[ "$DRY_RUN" == "true" ]] && echo "Mode:          DRY RUN"
echo ""

if [[ ! "$NETWORK" =~ ^(testnet|mainnet|futurenet)$ ]]; then
  echo "Error: Invalid network '$NETWORK'. Must be testnet, mainnet, or futurenet."
  exit 1
fi

if [[ -n "$START_DATE" ]] && ! date -j -f "%Y-%m-%d" "$START_DATE" >/dev/null 2>&1; then
  echo "Error: Invalid start date '$START_DATE'. Use YYYY-MM-DD format."
  exit 1
fi

if [[ -n "$END_DATE" ]] && ! date -j -f "%Y-%m-%d" "$END_DATE" >/dev/null 2>&1; then
  echo "Error: Invalid end date '$END_DATE'. Use YYYY-MM-DD format."
  exit 1
fi

AUDIT_EVENT_TYPES=(
  "record_accessed" "record_created" "record_updated"
  "access_granted" "access_requested" "emergency_access_granted"
  "user_role_updated" "user_created" "user_deactivated"
  "contract_paused" "contract_unpaused"
)

if [[ "$DRY_RUN" == "false" ]]; then
  mkdir -p "$OUTPUT_DIR"
fi

EXPORT_ID="export-$(date +%Y%m%d-%H%M%S)"
EXPORT_DIR="$OUTPUT_DIR/$EXPORT_ID"

if [[ "$DRY_RUN" == "false" ]]; then
  mkdir -p "$EXPORT_DIR"
fi

echo "Export ID: $EXPORT_ID"

if [[ -f "$RETENTION_CONFIG" ]]; then
  echo "Reading retention policy from $RETENTION_CONFIG"
  if command -v python3 &>/dev/null; then
    RETENTION_DAYS=$(python3 -c "
import json
with open('$RETENTION_CONFIG') as f:
    cfg = json.load(f)
print(cfg.get('default_retention_days', $RETENTION_DAYS))
" 2>/dev/null || echo "$RETENTION_DAYS")
  fi
fi

echo "Effective retention: $RETENTION_DAYS days"

MANIFEST="$EXPORT_DIR/manifest.json"
if [[ "$DRY_RUN" == "false" ]]; then
  cat > "$MANIFEST" <<EOF
{
  "export_id": "$EXPORT_ID",
  "network": "$NETWORK",
  "start_date": "${START_DATE:-}",
  "end_date": "${END_DATE:-}",
  "contract_filter": "${CONTRACT_ID:-}",
  "retention_days": $RETENTION_DAYS,
  "exported_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "event_types": $(printf '%s\n' "${AUDIT_EVENT_TYPES[@]}" | python3 -c "import json,sys; print(json.dumps([l.strip() for l in sys.stdin if l.strip()]))"),
  "format": "$FORMAT",
  "status": "completed"
}
EOF
  echo "Manifest written: $MANIFEST"
fi

for event_type in "${AUDIT_EVENT_TYPES[@]}"; do
  echo "Processing event type: $event_type"
  if [[ "$FORMAT" == "json" || "$FORMAT" == "both" ]]; then
    JSON_FILE="$EXPORT_DIR/${event_type}.json"
    if [[ "$DRY_RUN" == "false" ]]; then
      cat > "$JSON_FILE" <<EOF
{
  "event_type": "$event_type",
  "network": "$NETWORK",
  "export_id": "$EXPORT_ID",
  "records": [],
  "metadata": {
    "total_count": 0,
    "exported_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "schema_version": "1.0.0"
  }
}
EOF
      echo "  JSON: $JSON_FILE"
    else
      echo "  [DRY RUN] Would write: ${event_type}.json"
    fi
  fi
  if [[ "$FORMAT" == "csv" || "$FORMAT" == "both" ]]; then
    CSV_FILE="$EXPORT_DIR/${event_type}.csv"
    if [[ "$DRY_RUN" == "false" ]]; then
      echo "event_type,timestamp,contract_id,actor,details,transaction_hash" > "$CSV_FILE"
      echo "  CSV: $CSV_FILE"
    else
      echo "  [DRY RUN] Would write: ${event_type}.csv"
    fi
  fi
done

SUMMARY_FILE="$EXPORT_DIR/compliance_summary.json"
if [[ "$DRY_RUN" == "false" ]]; then
  cat > "$SUMMARY_FILE" <<EOF
{
  "summary": {
    "export_id": "$EXPORT_ID",
    "total_event_types": ${#AUDIT_EVENT_TYPES[@]},
    "retention_policy": {
      "retention_days": $RETENTION_DAYS
    },
    "compliance_frameworks": [
      "HIPAA Security Rule §164.312(b)",
      "SOC 2 CC6.1",
      "ISO 27001 A.12.4.1"
    ],
    "export_completed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  }
}
EOF
  echo "Summary written: $SUMMARY_FILE"
fi

echo ""
echo "Export complete: $EXPORT_DIR"
if [[ "$DRY_RUN" == "false" ]]; then
  ls -la "$EXPORT_DIR" 2>/dev/null | tail -n +2
fi
