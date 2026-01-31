#!/bin/bash

# interact.sh - Soroban Contract Interaction Script
# Usage: ./scripts/interact.sh <contract_id> <network> <function> [args...]

set -euo pipefail  # Exit on error, undefined vars, or pipe fail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if required arguments are provided
if [ $# -lt 3 ]; then
    print_error "Usage: $0 <contract_id> <network> <function> [args...]"
    print_error "Example: $0 CBQHNAXSI55GX2GN6D67GK7BHQHNAXSI55GX2GN6D67GK7BH testnet get_record --id 1"
    print_error "Available networks: local, testnet, futurenet, mainnet"
    exit 1
fi

CONTRACT_ID="$1"
NETWORK="$2"
FUNCTION="$3"
shift 3
ARGS=("$@")

IDENTITY=${SOROBAN_IDENTITY:-"default"}

print_status "Interacting with contract '$CONTRACT_ID' on '$NETWORK' network"

# Validate network
case $NETWORK in
    "local"|"testnet"|"futurenet"|"mainnet")
        ;;
    *)
        print_error "Unknown network: $NETWORK"
        print_error "Available networks: local, testnet, futurenet, mainnet"
        exit 1
        ;;
esac

# Check if identity exists
if ! soroban config identity show "$IDENTITY" &> /dev/null; then
    print_error "Identity '$IDENTITY' not found"
    print_error "Generate one with: soroban config identity generate $IDENTITY"
    exit 1
fi

IDENTITY_ADDRESS=$(soroban config identity address "$IDENTITY")
print_status "Using identity: $IDENTITY ($IDENTITY_ADDRESS)"

# Build the command
CMD="soroban contract invoke --id $CONTRACT_ID --source $IDENTITY --network $NETWORK"

# Add function and arguments
if [ ${#ARGS[@]} -eq 0 ]; then
    CMD="$CMD -- $FUNCTION"
else
    CMD="$CMD -- $FUNCTION ${ARGS[*]}"
fi

print_step "Executing: $CMD"

# Execute the command
if ! eval "$CMD"; then
    print_error "Contract interaction failed ❌"
    exit 1
fi

print_status "Contract interaction completed successfully! ✅"