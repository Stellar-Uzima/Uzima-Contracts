#!/usr/bin/env bash
# emergency_pause.sh – One-click contract pause for disaster recovery (Issue #422)
#
# Usage: ./scripts/dr/emergency_pause.sh <contract_id> <network> [identity]
#
# Freezes a contract via the upgradeability freeze mechanism, preventing
# further upgrades and signalling downstream systems to halt operations.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
LOG_FILE="$PROJECT_ROOT/deployments/incident_log.json"

CONTRACT_ID="${1:?Usage: $0 <contract_id> <network> [identity]}"
NETWORK="${2:?Usage: $0 <contract_id> <network> [identity]}"
IDENTITY="${3:-deployer-$NETWORK}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'

log()  { echo -e "${GREEN}[DR]${NC} $*"; }
warn() { echo -e "${YELLOW}[DR]${NC} $*"; }
err()  { echo -e "${RED}[DR]${NC} $*" >&2; }

log "=== EMERGENCY PAUSE ==="
log "Contract : $CONTRACT_ID"
log "Network  : $NETWORK"
log "Identity : $IDENTITY"
echo ""

# Confirm before proceeding (skip in CI)
if [ -t 0 ] && [ "${CI:-false}" != "true" ]; then
    warn "This will FREEZE the contract. It cannot be unfrozen without a new deployment."
    read -r -p "Type 'PAUSE' to confirm: " confirm
    if [ "$confirm" != "PAUSE" ]; then
        err "Aborted."
        exit 1
    fi
fi

# Execute freeze via soroban CLI
log "Invoking freeze on contract..."
soroban contract invoke \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --source "$IDENTITY" \
    -- freeze 2>&1 | tee /tmp/dr_pause_output.txt

EXIT_CODE=${PIPESTATUS[0]}

if [ "$EXIT_CODE" -ne 0 ]; then
    err "Freeze invocation failed (exit $EXIT_CODE). Check /tmp/dr_pause_output.txt"
    exit "$EXIT_CODE"
fi

log "✅ Contract frozen successfully."

# Log the incident
mkdir -p "$(dirname "$LOG_FILE")"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
ENTRY="{\"timestamp\":\"$TIMESTAMP\",\"action\":\"emergency_pause\",\"contract\":\"$CONTRACT_ID\",\"network\":\"$NETWORK\",\"operator\":\"$IDENTITY\"}"

if [ -f "$LOG_FILE" ]; then
    # Append to existing JSON array
    python3 -c "
import json, sys
with open('$LOG_FILE') as f:
    data = json.load(f)
data.append($ENTRY)
with open('$LOG_FILE', 'w') as f:
    json.dump(data, f, indent=2)
" 2>/dev/null || echo "[$ENTRY]" > "$LOG_FILE"
else
    echo "[$ENTRY]" > "$LOG_FILE"
fi

log "Incident logged to $LOG_FILE"
log ""
log "Next steps:"
log "  1. Investigate the issue"
log "  2. Prepare a fix or rollback"
log "  3. Run: ./scripts/dr/resume_contract.sh $CONTRACT_ID $NETWORK <new_wasm_hash>"
