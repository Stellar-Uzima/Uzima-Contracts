#!/bin/bash
# scripts/deployment_history.sh - Track and display deployment history
set -euo pipefail

HISTORY_DIR="deployments/history"
NETWORK="${1:-}"

mkdir -p "$HISTORY_DIR"

if [ -n "$NETWORK" ]; then
    HISTORY_FILE="$HISTORY_DIR/history-${NETWORK}.jsonl"
    if [ ! -f "$HISTORY_FILE" ]; then
        echo "No deployment history for network: $NETWORK"
        exit 0
    fi
    echo "=== Deployment History: $NETWORK ==="
    echo ""
    cat "$HISTORY_FILE" | jq -r '"\(.at) | \(.action) | \(.status // "n/a") | \(.actor // "system") | \(.commit // .version // "n/a")"' | column -t -s'|'
else
    echo "=== All Deployment History ==="
    echo ""
    for f in "$HISTORY_DIR"/history-*.jsonl; do
        [ -f "$f" ] || continue
        net=$(basename "$f" | sed 's/history-//;s/.jsonl//')
        count=$(wc -l < "$f")
        last=$(tail -1 "$f" | jq -r '.at // "unknown"')
        echo "  $net: $count deployments (last: $last)"
    done
fi