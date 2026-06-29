#!/bin/bash

# deploy_all.sh - Deploy all enabled contracts for a given environment
# Usage: ./scripts/deploy_all.sh <environment> <network> [identity]

set -euo pipefail

# --- Configuration ---
CONFIG_DIR="config"
DEFAULT_CONFIG="$CONFIG_DIR/default.json"

# Source the logger script
source "$(dirname "$0")/logger.sh"

# --- Argument validation ---
if [ $# -lt 2 ]; then
    log "ERROR" "Usage: $0 <environment> <network> [identity]"
    log "ERROR" "Example: $0 development local"
    log "ERROR" "Available environments: development, staging, production"
    exit 1
fi

ENVIRONMENT="$1"
NETWORK="$2"
IDENTITY="${3:-"default"}"
ENV_CONFIG="$CONFIG_DIR/$ENVIRONMENT.json"

# --- Pre-flight checks ---
if ! command -v jq &> /dev/null; then
    log "ERROR" "'jq' is not installed. Please install it to continue."
    exit 1
fi

if [ ! -f "$ENV_CONFIG" ]; then
    log "ERROR" "Configuration file for environment '$ENVIRONMENT' not found at '$ENV_CONFIG'"
    exit 1
fi

# --- Main script ---
log "INFO" "Starting deployment for environment: $ENVIRONMENT on $NETWORK network"

# Merge configurations
# This uses jq to deeply merge the default and environment-specific configs
CONFIG=$(jq -s '.[0] * .[1]' "$DEFAULT_CONFIG" "$ENV_CONFIG")

# Get a list of all enabled contracts
ENABLED_CONTRACTS=$(echo "$CONFIG" | jq -r '.contracts | to_entries[] | select(.value.enabled == true) | .key')

if [ -z "$ENABLED_CONTRACTS" ]; then
    log "INFO" "No contracts are enabled for deployment in the '$ENVIRONMENT' configuration."
    exit 0
fi

log "INFO" "Enabled contracts for deployment: $ENABLED_CONTRACTS"

# Loop and deploy each contract
for CONTRACT_NAME in $ENABLED_CONTRACTS; do
    log "INFO" "Deploying contract: $CONTRACT_NAME"
    
    # Construct deployment command
    DEPLOY_CMD="./scripts/deploy.sh $CONTRACT_NAME $NETWORK $IDENTITY"
    
    # Execute the deployment script
    if ! $DEPLOY_CMD; then
        log "ERROR" "Deployment of '$CONTRACT_NAME' failed. Aborting."
        exit 1
    fi
    
    log "INFO" "Successfully deployed contract: $CONTRACT_NAME"
done

log "INFO" "All enabled contracts deployed successfully for '$ENVIRONMENT' environment. ✅"