#!/bin/bash

# deployment_status.sh - Show deployment status for all contracts
# Usage: ./scripts/deployment_status.sh [network]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_header() {
    echo -e "${CYAN}$1${NC}"
}

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

NETWORK="${1:-all}"
DEPLOYMENTS_DIR="deployments"

main() {
    print_header "Deployment Status Report"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    if [ "$NETWORK" = "all" ]; then
        NETWORKS=("local" "testnet" "futurenet" "mainnet")
    else
        NETWORKS=("$NETWORK")
    fi
    
    for network in "${NETWORKS[@]}"; do
        print_header "Network: $network"
        echo ""
        
        DEPLOYMENT_FILES=($(find "$DEPLOYMENTS_DIR" -name "${network}_*.json" -type f ! -name "*_backup_*" ! -name "*_alerts*" ! -name "*rollback*" 2>/dev/null))
        
        if [ ${#DEPLOYMENT_FILES[@]} -eq 0 ]; then
            print_warning "No deployments found for $network"
            echo ""
            continue
        fi
        
        printf "%-30s %-60s %-20s\n" "Contract" "Contract ID" "Deployed At"
        echo "────────────────────────────────────────────────────────────────────────────────────────────────────────"
        
        for deployment_file in "${DEPLOYMENT_FILES[@]}"; do
            contract_name=$(jq -r '.contract_name' "$deployment_file" 2>/dev/null || echo "unknown")
            contract_id=$(jq -r '.contract_id' "$deployment_file" 2>/dev/null || echo "unknown")
            deployed_at=$(jq -r '.deployed_at' "$deployment_file" 2>/dev/null || echo "unknown")
            rolled_back=$(jq -r '.rolled_back // false' "$deployment_file" 2>/dev/null)
            
            # Truncate contract ID for display
            display_id="${contract_id:0:56}..."
            if [ ${#contract_id} -le 56 ]; then
                display_id="$contract_id"
            fi
            
            # Format deployed_at
            if [ "$deployed_at" != "unknown" ] && [ "$deployed_at" != "null" ]; then
                display_date=$(echo "$deployed_at" | cut -d'T' -f1)
            else
                display_date="unknown"
            fi
            
            status_marker=""
            if [ "$rolled_back" = "true" ]; then
                status_marker=" (rolled back)"
            fi
            
            printf "%-30s %-60s %-20s%s\n" "$contract_name" "$display_id" "$display_date" "$status_marker"
        done
        
        echo ""
        
        # Show backup count
        BACKUP_COUNT=$(find "$DEPLOYMENTS_DIR" -name "${network}_*_backup_*.json" -type f 2>/dev/null | wc -l | tr -d ' ')
        if [ "$BACKUP_COUNT" -gt 0 ]; then
            print_status "$BACKUP_COUNT backup(s) available"
        fi
        
        echo ""
    done
    
    # Show rollback history if exists
    if [ -f "$DEPLOYMENTS_DIR/rollback_log.json" ]; then
        ROLLBACK_COUNT=$(jq '. | length' "$DEPLOYMENTS_DIR/rollback_log.json" 2>/dev/null || echo "0")
        if [ "$ROLLBACK_COUNT" -gt 0 ]; then
            print_header "Rollback History"
            echo ""
            jq -r '.[] | "\(.rolled_back_at) - \(.contract_name) on \(.network)"' "$DEPLOYMENTS_DIR/rollback_log.json" 2>/dev/null | head -5
            if [ "$ROLLBACK_COUNT" -gt 5 ]; then
                echo "... and $((ROLLBACK_COUNT - 5)) more"
            fi
            echo ""
        fi
    fi
    
    print_header "Report generated at: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
}

main "$@"

