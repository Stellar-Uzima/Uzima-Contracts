#!/bin/bash

# deploy.sh - Soroban Contract Deployment Script
# Usage: ./scripts/deploy.sh <contract_name> <network> [identity]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Safer printing
print_status()  { printf "${GREEN}[INFO]${NC} %s\n" "$1"; }
print_warning() { printf "${YELLOW}[WARNING]${NC} %s\n" "$1"; }
print_error()   { printf "${RED}[ERROR]${NC} %s\n" "$1"; }
print_step()    { printf "${BLUE}[STEP]${NC} %s\n" "$1"; }

# Check args
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract_name> <network> [identity]"
    print_error "Example: $0 medical_records testnet alice"
    exit 1
fi

CONTRACT_NAME="$1"
NETWORK="$2"
IDENTITY="${3:-"default"}"

# Validate directory
CONTRACT_DIR="contracts/$CONTRACT_NAME"
if [ ! -d "$CONTRACT_DIR" ]; then
    print_error "Contract directory '$CONTRACT_DIR' does not exist"
    exit 1
fi

print_status "Starting deployment of '$CONTRACT_NAME' to '$NETWORK' network"

# Build
print_step "Building contract..."
# Use subshell to avoid changing directory for the whole script
(
    cd "$CONTRACT_DIR" || exit 1
    cargo clean
    cargo build --target wasm32-unknown-unknown --release
)

WASM_FILE="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
if [ ! -f "$WASM_FILE" ]; then
    print_error "Build failed: $WASM_FILE not found"
    exit 1
fi

print_status "Contract built successfully"

# Optimize
if command -v soroban &> /dev/null; then
    print_step "Optimizing contract..."
    soroban contract optimize --wasm "$WASM_FILE" || print_warning "Optimization skipped/failed"
fi

# Deploy
print_step "Deploying contract..."
# Capture output but allow failure to be caught
if ! CONTRACT_ID=$(soroban contract deploy --wasm "$WASM_FILE" --source "$IDENTITY" --network "$NETWORK"); then
    print_error "Deployment failed."
    exit 1
fi

print_status "Contract deployed successfully!"
print_status "Contract ID: $CONTRACT_ID"

# Save info
mkdir -p deployments
DEPLOY_INFO_FILE="deployments/${NETWORK}_${CONTRACT_NAME}.json"

cat > "$DEPLOY_INFO_FILE" << EOF
{
    "contract_name": "$CONTRACT_NAME",
    "contract_id": "$CONTRACT_ID",
    "network": "$NETWORK",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

print_status "Deployment complete! 🚀"