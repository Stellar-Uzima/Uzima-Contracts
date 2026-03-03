#!/bin/bash

# deploy_all.sh - Deploy all enabled contracts for a given environment
# Usage: ./scripts/deploy_all.sh <environment> <network> [identity]

set -euo pipefail

# --- Configuration ---
CONFIG_DIR="config"
DEFAULT_CONFIG="$CONFIG_DIR/default.json"

# --- Colors for output ---
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# --- Helper functions ---
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# --- Argument validation ---
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <environment> <network> [identity]"
    print_error "Example: $0 development local"
    print_error "Available environments: development, staging, production"
fi

ENVIRONMENT="$1"
NETWORK="$2"
IDENTITY="${3:-"default"}"
ENV_CONFIG="$CONFIG_DIR/$ENVIRONMENT.json"

# --- Pre-flight checks ---
if ! command -v jq &> /dev/null; then
    print_error "'jq' is not installed. Please install it to continue."
fi

if [ ! -f "$ENV_CONFIG" ]; then
    print_error "Configuration file for environment '$ENVIRONMENT' not found at '$ENV_CONFIG'"
fi

# --- Main script ---
print_status "Starting deployment for environment: $ENVIRONMENT on $NETWORK network"

# Merge configurations
# This uses jq to deeply merge the default and environment-specific configs
CONFIG=$(jq -s '.[0] * .[1]' "$DEFAULT_CONFIG" "$ENV_CONFIG")

# Get a list of all enabled contracts
ENABLED_CONTRACTS=$(echo "$CONFIG" | jq -r '.contracts | to_entries[] | select(.value.enabled == true) | .key')

if [ -z "$ENABLED_CONTRACTS" ]; then
    print_status "No contracts are enabled for deployment in the '$ENVIRONMENT' configuration."
    exit 0
fi

print_status "Enabled contracts for deployment: 
$ENABLED_CONTRACTS"

# Loop and deploy each contract
for CONTRACT_NAME in $ENABLED_CONTRACTS; do
    print_step "Deploying contract: $CONTRACT_NAME"
    
    # Construct deployment command
    DEPLOY_CMD="./scripts/deploy.sh $CONTRACT_NAME $NETWORK $IDENTITY"
    
    # Execute the deployment script
    if ! $DEPLOY_CMD; then
        print_error "Deployment of '$CONTRACT_NAME' failed. Aborting."
    fi
    
    print_status "Successfully deployed contract: $CONTRACT_NAME"
done

print_status "All enabled contracts deployed successfully for '$ENVIRONMENT' environment. ✅"
