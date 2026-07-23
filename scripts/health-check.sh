#!/bin/bash
# scripts/health-check.sh - Container health checks for local dev services
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

MAX_RETRIES=30
RETRY_INTERVAL=2

check_service() {
    local name="$1"; shift
    local retries=0
    echo -n "  $name: "
    while [ $retries -lt $MAX_RETRIES ]; do
        if "$@" &>/dev/null; then
            echo -e "${GREEN}healthy${NC}"
            return 0
        fi
        ((retries++))
        sleep $RETRY_INTERVAL
    done
    echo -e "${RED}unreachable (after ${MAX_RETRIES} attempts)${NC}"
    return 1
}

echo "=== Container Health Check ==="
echo ""

FAILED=0

if command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
    echo "[Docker containers]"
    for cid in $(docker ps --format '{{.Names}}' 2>/dev/null); do
        STATUS=$(docker inspect --format '{{.State.Status}}' "$cid" 2>/dev/null || echo "unknown")
        HEALTH=$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}no-healthcheck{{end}}' "$cid" 2>/dev/null || echo "unknown")
        if [ "$STATUS" = "running" ]; then
            echo -e "  $cid: ${GREEN}$STATUS${NC} (health: $HEALTH)"
        else
            echo -e "  $cid: ${RED}$STATUS${NC}"
            ((FAILED++))
        fi
    done

    echo ""
    echo "[Port checks]"
    check_service "Stellar RPC (8000)" curl -sf http://localhost:8000/health || ((FAILED++))
    check_service "Soroban RPC (8000/soroban/rpc)" curl -sf http://localhost:8000/soroban/rpc || ((FAILED++))
else
    echo -e "${YELLOW}Docker not available - skipping container checks${NC}"
fi

echo ""
if [ "$FAILED" -gt 0 ]; then
    echo -e "${RED}Health check failed ($FAILED issue(s))${NC}"
    exit 1
fi
echo -e "${GREEN}All services healthy${NC}"