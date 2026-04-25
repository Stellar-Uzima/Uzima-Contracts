#!/usr/bin/env bash
# health_check.sh – Contract health monitoring for disaster recovery (Issue #422)
#
# Usage: ./scripts/dr/health_check.sh <network> [--alert-on-failure]
#
# Checks all deployed contracts on the given network and reports their health.
# Exits non-zero if any contract is unhealthy and --alert-on-failure is set.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
DEPLOYMENTS_DIR="$PROJECT_ROOT/deployments"

NETWORK="${1:?Usage: $0 <network> [--alert-on-failure]}"
ALERT_ON_FAILURE=false
if [[ "${2:-}" == "--alert-on-failure" ]]; then
    ALERT_ON_FAILURE=true
fi

GREEN='\033[0;32m'; RED='\033[0;31m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'

log()     { echo -e "${BLUE}[HEALTH]${NC} $*"; }
ok()      { echo -e "${GREEN}[HEALTH] ✓${NC} $*"; }
warn()    { echo -e "${YELLOW}[HEALTH] ⚠${NC} $*"; }
fail()    { echo -e "${RED}[HEALTH] ✗${NC} $*"; }

HEALTHY=0
UNHEALTHY=0
TOTAL=0

log "=== Health Check: $NETWORK ==="
log "Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo ""

# Find all deployment files for this network
DEPLOYMENT_FILES=("$DEPLOYMENTS_DIR"/"${NETWORK}"_*.json)

if [ ${#DEPLOYMENT_FILES[@]} -eq 0 ] || [ ! -f "${DEPLOYMENT_FILES[0]}" ]; then
    warn "No deployment files found for network '$NETWORK' in $DEPLOYMENTS_DIR"
    warn "Run a deployment first or check the network name."
    exit 0
fi

for deployment_file in "${DEPLOYMENT_FILES[@]}"; do
    # Skip backup and log files
    [[ "$deployment_file" == *"_backup_"* ]] && continue
    [[ "$deployment_file" == *"incident_log"* ]] && continue
    [[ "$deployment_file" == *"rollback_log"* ]] && continue
    [[ ! -f "$deployment_file" ]] && continue

    CONTRACT_NAME=$(python3 -c "import json; d=json.load(open('$deployment_file')); print(d.get('contract_name','unknown'))" 2>/dev/null || echo "unknown")
    CONTRACT_ID=$(python3 -c "import json; d=json.load(open('$deployment_file')); print(d.get('contract_id',''))" 2>/dev/null || echo "")

    if [ -z "$CONTRACT_ID" ]; then
        warn "Skipping $deployment_file (no contract_id)"
        continue
    fi

    TOTAL=$((TOTAL + 1))

    # Attempt a lightweight read-only invocation to verify liveness
    if soroban contract invoke \
        --id "$CONTRACT_ID" \
        --network "$NETWORK" \
        -- version 2>/dev/null 1>/dev/null; then
        ok "$CONTRACT_NAME ($CONTRACT_ID) – healthy"
        HEALTHY=$((HEALTHY + 1))
    else
        fail "$CONTRACT_NAME ($CONTRACT_ID) – UNHEALTHY or unreachable"
        UNHEALTHY=$((UNHEALTHY + 1))
    fi
done

echo ""
log "=== Summary ==="
log "Total:    $TOTAL"
ok "Healthy:  $HEALTHY"
if [ "$UNHEALTHY" -gt 0 ]; then
    fail "Unhealthy: $UNHEALTHY"
else
    log "Unhealthy: $UNHEALTHY"
fi

# Write health report
REPORT_FILE="$DEPLOYMENTS_DIR/health_report_${NETWORK}_$(date +%Y%m%d_%H%M%S).json"
cat > "$REPORT_FILE" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "network": "$NETWORK",
  "total": $TOTAL,
  "healthy": $HEALTHY,
  "unhealthy": $UNHEALTHY,
  "status": "$([ "$UNHEALTHY" -eq 0 ] && echo "ok" || echo "degraded")"
}
EOF
log "Report written to $REPORT_FILE"

if [ "$UNHEALTHY" -gt 0 ] && [ "$ALERT_ON_FAILURE" = true ]; then
    fail "Alert: $UNHEALTHY contract(s) unhealthy on $NETWORK"
    exit 1
fi
