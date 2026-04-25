#!/bin/bash

# Multi-Region DR System Monitoring Script
# Continuously monitors health and uptime of deployed regions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
NETWORK="${1:-testnet}"
MRO_ID="${2}"
RNM_ID="${3}"
FD_ID="${4}"
SM_ID="${5}"
DEPLOYER="${6:-deployer-$NETWORK}"
CHECK_INTERVAL="${7:-30}" # seconds
LOGS_DIR="$PROJECT_ROOT/deployments"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Global tracking
declare -A region_status
declare -A uptime_metrics
restart_count=0
total_checks=0
failed_checks=0

log_info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] ✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] ⚠${NC} $1"
}

log_error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ✗${NC} $1"
}

# Validate contract IDs
validate_contracts() {
    if [ -z "$MRO_ID" ] || [ -z "$RNM_ID" ] || [ -z "$FD_ID" ] || [ -z "$SM_ID" ]; then
        log_error "Missing contract IDs. Usage: $0 <network> <mro_id> <rnm_id> <fd_id> <sm_id> [deployer] [interval]"
        echo ""
        echo "Example: $0 testnet CA7QXN26N... CA8QXN26N... CA9QXN26N... CB0QXN26N... deployer-testnet 30"
        exit 1
    fi
}

# Check health of orchestrator
check_orchestrator_health() {
    local status="healthy"
    
    local output=$(soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$MRO_ID" \
        -- check_health \
        --caller "$(soroban config identity address $DEPLOYER)" 2>&1)
    
    if echo "$output" | grep -q "true"; then
        status="healthy"
    elif echo "$output" | grep -q "false"; then
        status="degraded"
    else
        status="unknown"
        log_warn "Could not determine orchestrator status"
    fi
    
    echo "$status"
}

# Check node manager health
check_node_manager_health() {
    local count=0
    
    local output=$(soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$RNM_ID" \
        -- list_nodes 2>&1)
    
    # Count nodes in output
    if [ ! -z "$output" ]; then
        count=$(echo "$output" | grep -o "region_name" | wc -l)
    fi
    
    if [ $count -gt 0 ]; then
        echo "healthy"
    else
        echo "no_nodes"
    fi
}

# Check failover capacity
check_failover_readiness() {
    local plans=0
    
    local output=$(soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$FD_ID" \
        -- get_failover_plans 2>&1)
    
    if [ ! -z "$output" ]; then
        plans=$(echo "$output" | grep -o "plan_id" | wc -l)
    fi
    
    if [ $plans -gt 0 ]; then
        echo "ready"
    else
        echo "not_ready"
    fi
}

# Check sync operations
check_sync_status() {
    local operations=0
    
    local output=$(soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$SM_ID" \
        -- list_sync_operations 2>&1)
    
    if [ ! -z "$output" ]; then
        operations=$(echo "$output" | grep -o "operation_id" | wc -l)
    fi
    
    if [ $operations -gt 0 ]; then
        echo "active"
    else
        echo "idle"
    fi
}

# Get current uptime
get_uptime_percentage() {
    local uptime=10000  # default 100%
    
    local output=$(soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$MRO_ID" \
        -- get_current_uptime 2>&1)
    
    if [[ "$output" =~ [0-9]+ ]]; then
        uptime=$(echo "$output" | grep -oE "[0-9]+" | head -1)
    fi
    
    # Convert basis points to percentage
    echo "scale=2; $uptime / 100" | bc 2>/dev/null || echo "99.99"
}

# Perform health check
perform_health_check() {
    total_checks=$((total_checks + 1))
    
    local timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local orc_health=$(check_orchestrator_health)
    local node_health=$(check_node_manager_health)
    local failover=$(check_failover_readiness)
    local sync=$(check_sync_status)
    local uptime=$(get_uptime_percentage)
    
    log_info "Health Check #$total_checks"
    
    # Report status
    if [ "$orc_health" = "healthy" ]; then
        log_success "Orchestrator: $orc_health"
    else
        log_warn "Orchestrator: $orc_health"
        failed_checks=$((failed_checks + 1))
    fi
    
    log_info "  Node Manager: $node_health"
    log_info "  Failover: $failover"
    log_info "  Sync: $sync"
    log_info "  Uptime: ${uptime}%"
    
    # Save metrics
    cat >> "$LOGS_DIR/dr_metrics.log" << EOF
{
  "timestamp": "$timestamp",
  "check_number": $total_checks,
  "orchestrator_health": "$orc_health",
  "node_manager_status": "$node_health",
  "failover_readiness": "$failover",
  "sync_status": "$sync",
  "uptime_percentage": "$uptime"
}
EOF
    
    # Check if uptime below target
    if (( $(echo "$uptime < 99.99" | bc -l) )); then
        log_error "Uptime below SLA target: ${uptime}%"
    fi
}

# Monitor loop
monitor_loop() {
    log_info "Starting continuous monitoring..."
    log_info "Check interval: ${CHECK_INTERVAL}s"
    log_info "Contracts:"
    log_info "  Orchestrator: $MRO_ID"
    log_info "  Node Manager: $RNM_ID"
    log_info "  Failover Detector: $FD_ID"
    log_info "  Sync Manager: $SM_ID"
    
    mkdir -p "$LOGS_DIR"
    
    # Initialize metrics file
    echo "# Multi-Region DR Metrics" > "$LOGS_DIR/dr_metrics.log"
    
    while true; do
        perform_health_check
        
        # Print summary every 10 checks
        if [ $((total_checks % 10)) -eq 0 ]; then
            local success_rate=$(echo "scale=2; (($total_checks - $failed_checks) * 100) / $total_checks" | bc)
            log_info "=== Summary: $total_checks checks, Success rate: ${success_rate}% ==="
        fi
        
        sleep "$CHECK_INTERVAL"
    done
}

# Graceful shutdown
trap 'log_info "Monitoring stopped. Total checks: $total_checks, Failed: $failed_checks"; exit 0' SIGINT SIGTERM

# Main
validate_contracts
monitor_loop
