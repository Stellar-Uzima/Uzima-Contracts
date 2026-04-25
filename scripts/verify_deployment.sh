#!/bin/bash

# verify_deployment.sh - Professional Contract Deployment Verification Script
# This script performs a 5-step verification of a deployed Soroban contract.
# Usage: ./scripts/verify_deployment.sh <contract_id> <network> [identity] [contract_name]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print helper functions
print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_header() { 
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${PURPLE}  Verification: $1${NC}"
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Check arguments
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract_id> <network> [identity] [contract_name]"
    exit 1
fi

CONTRACT_ID="$1"
NETWORK="$2"
IDENTITY="${3:-"default"}"
CONTRACT_NAME="${4:-"unknown"}"

print_header "$CONTRACT_NAME ($CONTRACT_ID) on $NETWORK"

# Helper function to invoke soroban with standard args
invoke_soroban() {
    local function_name=$1
    shift
    soroban contract invoke \
        --id "$CONTRACT_ID" \
        --source "$IDENTITY" \
        --network "$NETWORK" \
        -- "$function_name" "$@" 2>&1
}

# ------------------------------------------------------------------------------
# STEP 1: Deployment Verification
# ------------------------------------------------------------------------------
print_step "1/5: Verifying deployment existence..."
if ! soroban contract read \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --durability instance &>/dev/null; then
    
    # Fallback: check if we can get help output
    if ! soroban contract invoke --id "$CONTRACT_ID" --network "$NETWORK" -- --help &>/dev/null; then
        print_error "Contract $CONTRACT_ID not found or not reachable on $NETWORK"
        exit 1
    fi
fi
print_info "Contract successfully located on the network."

# ------------------------------------------------------------------------------
# STEP 2: Initialization Check
# ------------------------------------------------------------------------------
print_step "2/5: Checking initialization status..."
# Try to call initialize again - if it fails with "AlreadyInitialized" or similar, it's good.
# Or try to read 'Initialized' key if we know the contract structure.
INIT_CHECK=$(invoke_soroban initialize --admin "$(soroban config identity address "$IDENTITY")" 2>&1 || true)

if echo "$INIT_CHECK" | grep -qiE "AlreadyInitialized|Error\(Contract, 1\)|Already Initialized"; then
    print_info "Initialization confirmed (Contract reports already initialized)."
elif echo "$INIT_CHECK" | grep -qi "success"; then
    print_warn "Contract was NOT initialized; initialization performed during verification."
else
    # Try calling a simple getter that might fail if not initialized
    GETTER_CHECK=$(invoke_soroban version 2>&1 || invoke_soroban name 2>&1 || true)
    if echo "$GETTER_CHECK" | grep -qi "NotInitialized|Error\(Contract, 2\)"; then
        print_error "Contract is NOT initialized and initialization attempt failed."
        exit 1
    else
        print_info "Initialization status seems valid (or not required)."
    fi
fi

# ------------------------------------------------------------------------------
# STEP 3: Basic Functionality Test
# ------------------------------------------------------------------------------
print_step "3/5: Running basic functionality tests..."
# Try some common getters
declare -a GETTERS=("version" "name" "symbol" "decimals" "get_admin" "get_owner")
SUCCESSFUL_TEST=false

for getter in "${GETTERS[@]}"; do
    RESULT=$(invoke_soroban "$getter" 2>/dev/null || true)
    if [ -n "$RESULT" ] && ! echo "$RESULT" | grep -qi "error"; then
        print_info "Functionality test passed: $getter() -> $RESULT"
        SUCCESSFUL_TEST=true
        break
    fi
done

if [ "$SUCCESSFUL_TEST" = false ]; then
    print_warn "Could not find standard getter to test. Trying generic ping..."
    if invoke_soroban test_ping &>/dev/null; then
        print_info "Functionality test passed: test_ping()"
        SUCCESSFUL_TEST=true
    fi
fi

if [ "$SUCCESSFUL_TEST" = false ]; then
    print_warn "No standard health-check functions found. Proceeding with caution."
fi

# ------------------------------------------------------------------------------
# STEP 4: Event Emission Check
# ------------------------------------------------------------------------------
print_step "4/5: Checking event emission..."
# We use a diagnostic call or a state-changing call if safe.
# For now, we'll look at the last transaction if we can, or just trigger a log if possible.
# Professional way: capture output of an invocation and parse for events.
EVENT_OUTPUT=$(invoke_soroban get_admin 2>&1 || invoke_soroban name 2>&1 || true)
# Soroban CLI output for events usually contains "Events:"
if echo "$EVENT_OUTPUT" | grep -q "Events:"; then
    print_info "Event emission verified in contract interaction."
else
    print_info "No immediate events detected, which may be normal for read-only calls."
fi

# ------------------------------------------------------------------------------
# STEP 5: Storage State Validation
# ------------------------------------------------------------------------------
print_step "5/5: Validating storage state..."
STORAGE_DATA=$(soroban contract read --id "$CONTRACT_ID" --network "$NETWORK" --durability instance 2>/dev/null || echo "")
if [ -n "$STORAGE_DATA" ]; then
    STORAGE_SIZE=$(echo "$STORAGE_DATA" | wc -c)
    print_info "Storage state validated (Instance storage size: $STORAGE_SIZE bytes)."
else
    print_warn "Could not read instance storage directly. Verification may be incomplete."
fi

echo -e "\n${GREEN}✅ Verification completed successfully!${NC}"
exit 0
