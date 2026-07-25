#!/usr/bin/env bash
# ============================================================================
# common_health_check.sh
# Probes deployed contracts for health status via Soroban RPC.
#
# Usage:
#   ./scripts/common_health_check.sh [network] [contract_ids_file]
#
# If contract_ids_file is omitted, uses deployments/deployment-manifest.json.
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${GREEN}[OK]${NC}     $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}   $1"; }
fail()    { echo -e "${RED}[FAIL]${NC}   $1"; }

NETWORK="${1:-testnet}"
MANIFEST="$ROOT_DIR/deployments/deployment-manifest.json"

# Network RPC URLs
case "$NETWORK" in
  testnet)   RPC_URL="https://soroban-testnet.stellar.org" ;;
  futurenet) RPC_URL="https://rpc-futurenet.stellar.org" ;;
  mainnet)   RPC_URL="https://soroban-mainnet.stellar.org" ;;
  local)     RPC_URL="http://localhost:8000/soroban/rpc" ;;
  *)         fail "Unknown network: $NETWORK"; exit 1 ;;
esac

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Common Health Check — $NETWORK${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

TOTAL=0
HEALTHY=0
DEGRADED=0
UNHEALTHY=0
NOT_DEPLOYED=0

# ── Extract contract IDs from manifest ────────────────────────────────────────
if [[ -f "$MANIFEST" ]]; then
  CONTRACT_IDS=$(python3 -c "
import json
with open('$MANIFEST') as f:
    m = json.load(f)
net = '$NETWORK'
for c in m.get('contracts', []):
    cid = c.get('networks', {}).get(net, {}).get('contract_id')
    name = c.get('name', 'unknown')
    if cid:
        print(f'{name}|{cid}')
" 2>/dev/null || true)
else
  warn "Manifest not found at $MANIFEST — checking individual contracts"
  CONTRACT_IDS=""
fi

# ── If no manifest contracts, do a basic cargo check instead ───────────────────
if [[ -z "$CONTRACT_IDS" ]]; then
  warn "No deployed contract IDs found. Running workspace compilation health check."
  echo ""

  echo -e "${BLUE}── Compilation Health ──${NC}"
  FAILING=0
  for manifest in "$ROOT_DIR"/contracts/*/Cargo.toml; do
    [[ -f "$manifest" ]] || continue
    NAME=$(basename "$(dirname "$manifest")")
    # Skip excluded
    if sed -n '/^exclude = \[/,/^]/p' "$ROOT_DIR/Cargo.toml" 2>/dev/null | grep -q "\"contracts/$NAME\""; then
      continue
    fi
    TOTAL=$((TOTAL + 1))
    if cargo check -p "$NAME" --quiet 2>/dev/null; then
      HEALTHY=$((HEALTHY + 1))
    else
      fail "$NAME — compilation failed"
      UNHEALTHY=$((UNHEALTHY + 1))
      FAILING=$((FAILING + 1))
    fi
  done

  echo ""
  echo -e "${BLUE}── Summary ──${NC}"
  info "Total contracts checked: $TOTAL"
  info "Healthy (compiles):      $HEALTHY"
  [[ $UNHEALTHY -gt 0 ]] && fail "Unhealthy (failing):    $UNHEALTHY"

  if [[ $FAILING -gt 0 ]]; then
    exit 1
  fi
  exit 0
fi

# ── Probe each deployed contract ──────────────────────────────────────────────
echo -e "${BLUE}── Probing Deployed Contracts ──${NC}"
echo ""

while IFS='|' read -r NAME CID; do
  [[ -z "$NAME" || -z "$CID" ]] && continue
  TOTAL=$((TOTAL + 1))

  # Try to invoke the health_check function (best-effort)
  RESULT=$(soroban contract invoke \
    --id "$CID" \
    --network "$NETWORK" \
    -- health_check 2>&1 || echo "ERROR")

  if echo "$RESULT" | grep -q "ERROR\|error\|panic"; then
    # Try a simpler ping: just read an instance entry
    PING=$(soroban contract invoke \
      --id "$CID" \
      --network "$NETWORK" \
      -- version 2>&1 || echo "ERROR")

    if echo "$PING" | grep -q "ERROR\|error\|panic"; then
      fail "$NAME ($CID) — unresponsive"
      UNHEALTHY=$((UNHEALTHY + 1))
    else
      warn "$NAME ($CID) — responds to version() but no health_check()"
      DEGRADED=$((DEGRADED + 1))
    fi
  else
    info "$NAME ($CID) — healthy"
    HEALTHY=$((HEALTHY + 1))
  fi
done <<< "$CONTRACT_IDS"

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo -e "${BLUE}── Summary ──${NC}"
info "Total contracts:    $TOTAL"
info "Healthy:            $HEALTHY"
[[ $DEGRADED -gt 0 ]] && warn "Degraded:           $DEGRADED"
[[ $UNHEALTHY -gt 0 ]] && fail "Unhealthy:          $UNHEALTHY"

if [[ $UNHEALTHY -gt 0 ]]; then
  exit 1
else
  exit 0
fi
