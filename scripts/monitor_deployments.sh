#!/bin/bash

# monitor_deployments.sh - Monitor deployed contracts and send alerts
# Usage: ./scripts/monitor_deployments.sh [network] [--alert-on-failure]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Configuration
NETWORK="${1:-testnet}"
ALERT_ON_FAILURE=false
DEPLOYMENTS_DIR="deployments"

# Parse arguments
if [[ "$*" == *"--alert-on-failure"* ]]; then
    ALERT_ON_FAILURE=true
fi

# Function to send alert (can be customized for different alerting systems)
send_alert() {
    local level="$1"
    local message="$2"
    local contract="$3"
    local contract_id="$4"
    
    echo "[$level] $message - Contract: $contract ($contract_id)" >> "deployments/alerts.log"
    
    # Example: Send to webhook (uncomment and configure)
    # if [ -n "${WEBHOOK_URL:-}" ]; then
    #     curl -X POST "$WEBHOOK_URL" \
    #         -H "Content-Type: application/json" \
    #         -d "{\"level\":\"$level\",\"message\":\"$message\",\"contract\":\"$contract\",\"contract_id\":\"$contract_id\"}"
    # fi
}

# Function to check contract health
check_contract_health() {
    local contract_id="$1"
    local contract_name="$2"
    
    # Try to invoke contract (simple health check)
    if soroban contract invoke \
        --id "$contract_id" \
        --network "$NETWORK" \
        -- --help &> /dev/null; then
        return 0
    else
        # Some contracts may not support --help, try a different approach
        # Check if contract exists by querying network
        if soroban contract read \
            --id "$contract_id" \
            --network "$NETWORK" \
            &> /dev/null; then
            return 0
        else
            return 1
        fi
    fi
}

# Function to get contract info
get_contract_info() {
    local contract_id="$1"
    local network="$2"
    
    # Get contract details from network
    soroban contract read \
        --id "$contract_id" \
        --network "$network" \
        2>/dev/null || echo "{}"
}

# Main monitoring function
main() {
    print_status "Monitoring deployments on $NETWORK network"
    
    mkdir -p "$DEPLOYMENTS_DIR"
    
    # Find all deployment files for this network
    DEPLOYMENT_FILES=($(find "$DEPLOYMENTS_DIR" -name "${NETWORK}_*.json" -type f ! -name "*_backup_*" ! -name "*_alerts*"))
    
    if [ ${#DEPLOYMENT_FILES[@]} -eq 0 ]; then
        print_warning "No deployment files found for network: $NETWORK"
        return
    fi
    
    print_step "Found ${#DEPLOYMENT_FILES[@]} deployment(s) to monitor"
    
    HEALTHY_COUNT=0
    UNHEALTHY_COUNT=0
    UNHEALTHY_CONTRACTS=()
    
    for deployment_file in "${DEPLOYMENT_FILES[@]}"; do
        if [ ! -f "$deployment_file" ]; then
            continue
        fi
        
        contract_name=$(jq -r '.contract_name' "$deployment_file" 2>/dev/null || echo "unknown")
        contract_id=$(jq -r '.contract_id' "$deployment_file" 2>/dev/null || echo "")
        deployed_at=$(jq -r '.deployed_at' "$deployment_file" 2>/dev/null || echo "unknown")
        
        if [ -z "$contract_id" ] || [ "$contract_id" = "null" ]; then
            print_warning "Invalid deployment file: $deployment_file"
            continue
        fi
        
        print_step "Checking $contract_name ($contract_id)..."
        
        # Check contract health
        if check_contract_health "$contract_id" "$contract_name"; then
            print_status "✓ $contract_name is healthy"
            HEALTHY_COUNT=$((HEALTHY_COUNT + 1))
        else
            print_error "✗ $contract_name is unhealthy"
            UNHEALTHY_COUNT=$((UNHEALTHY_COUNT + 1))
            UNHEALTHY_CONTRACTS+=("$contract_name:$contract_id")
            
            if [ "$ALERT_ON_FAILURE" = true ]; then
                send_alert "ERROR" "Contract health check failed" "$contract_name" "$contract_id"
            fi
        fi
        
        # Get and display contract info
        echo "  Deployed at: $deployed_at"
        echo "  Contract ID: $contract_id"
    done
    
    # Summary
    echo ""
    print_step "Monitoring Summary"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_status "Network: $NETWORK"
    print_status "Healthy contracts: $HEALTHY_COUNT"
    
    if [ $UNHEALTHY_COUNT -gt 0 ]; then
        print_error "Unhealthy contracts: $UNHEALTHY_COUNT"
        for contract_info in "${UNHEALTHY_CONTRACTS[@]}"; do
            contract_name=$(echo "$contract_info" | cut -d: -f1)
            contract_id=$(echo "$contract_info" | cut -d: -f2)
            echo "  ✗ $contract_name ($contract_id)"
        done
        
        if [ "$ALERT_ON_FAILURE" = true ]; then
            print_warning "Alerts have been sent for unhealthy contracts"
        fi
        
        exit 1
    else
        print_status "All contracts are healthy! ✓"
    fi
}

# Run monitoring
main "$@"

