#!/bin/bash

# deploy.sh - Soroban Contract Deployment Script
# Usage: ./scripts/deploy.sh <contract_name> <network> [identity]

set -euo pipefail  # Exit on error, undefined vars, or pipe fail

# Source the logger script
source "$(dirname "$0")/logger.sh"

# Check if required arguments are provided
if [ $# -lt 2 ]; then
    log "ERROR" "Usage: $0 <contract_name> <network> [identity]"
    log "ERROR" "Example: $0 medical_records testnet alice"
    log "ERROR" "Available networks: local, testnet, futurenet, mainnet"
    exit 1
fi

CONTRACT_NAME="$1"
NETWORK="$2"
IDENTITY="${3:-"default"}"

# Validate contract exists
CONTRACT_DIR="contracts/$CONTRACT_NAME"
if [ ! -d "$CONTRACT_DIR" ]; then
    log "ERROR" "Contract directory '$CONTRACT_DIR' does not exist"
    exit 1
fi

log "INFO" "Starting deployment of '$CONTRACT_NAME' to '$NETWORK' network"

# Build the contract
log "INFO" "Building contract..."

# Clean previous builds
# cargo clean -p "$CONTRACT_NAME" || { log "ERROR" "Cargo clean failed"; exit 1; }

# Build for WebAssembly target
if ! cargo build -p "$CONTRACT_NAME" --target wasm32-unknown-unknown --release; then
    log "ERROR" "Cargo build failed"
    exit 1
fi

# Check if build was successful
WASM_FILE="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
if [ ! -f "$WASM_FILE" ]; then
    log "ERROR" "Build failed: $WASM_FILE not found"
    exit 1
fi

log "INFO" "Contract built successfully"

# Optimize the contract (if soroban contract optimize is available)
if command -v soroban &> /dev/null; then
    log "INFO" "Optimizing contract..."
    if ! soroban contract optimize --wasm "$WASM_FILE"; then
        log "WARN" "Optimization failed, continuing with unoptimized contract"
    fi
fi

# Configure network if not already configured
log "INFO" "Configuring network..."
case $NETWORK in
    "local")
        if ! soroban config network add local \
            --rpc-url http://localhost:8000/soroban/rpc \
            --network-passphrase "Standalone Network ; February 2017" 2>/dev/null; then
            log "WARN" "Network 'local' already configured"
        fi
        ;;
    "testnet")
        if ! soroban config network add testnet \
            --rpc-url https://soroban-testnet.stellar.org:443 \
            --network-passphrase "Test SDF Network ; September 2015" 2>/dev/null; then
            log "WARN" "Network 'testnet' already configured"
        fi
        ;;
    "futurenet")
        if ! soroban config network add futurenet \
            --rpc-url https://rpc-futurenet.stellar.org:443 \
            --network-passphrase "Test SDF Future Network ; October 2022" 2>/dev/null; then
            log "WARN" "Network 'futurenet' already configured"
        fi
        ;;
    "mainnet")
        if ! soroban config network add mainnet \
            --rpc-url https://soroban-rpc.stellar.org:443 \
            --network-passphrase "Public Global Stellar Network ; September 2015" 2>/dev/null; then
            log "WARN" "Network 'mainnet' already configured"
        fi
        ;;
    *)
        log "ERROR" "Unknown network: $NETWORK"
        log "ERROR" "Available networks: local, testnet, futurenet, mainnet"
        exit 1
        ;;
esac

# Ensure identity exists
log "INFO" "Checking identity..."
if ! soroban config identity show "$IDENTITY" &> /dev/null; then
    log "WARN" "Identity '$IDENTITY' not found, generating new one..."
    if ! soroban config identity generate "$IDENTITY"; then
        log "ERROR" "Failed to generate identity '$IDENTITY'"
        exit 1
    fi
fi

# Get identity address
IDENTITY_ADDRESS=$(soroban config identity address "$IDENTITY")
log "INFO" "Using identity: $IDENTITY ($IDENTITY_ADDRESS)"

# Fund account for testnet/futurenet
if [ "$NETWORK" = "testnet" ] || [ "$NETWORK" = "futurenet" ]; then
    log "INFO" "Funding account on $NETWORK..."
    if ! soroban config identity fund "$IDENTITY" --network "$NETWORK"; then
        log "WARN" "Failed to fund account, continuing anyway..."
    fi
fi

# Deploy the contract
log "INFO" "Deploying contract..."
CONTRACT_ID=$(soroban contract deploy \
    --wasm "$WASM_FILE" \
    --source "$IDENTITY" \
    --network "$NETWORK" 2>/dev/null) || { log "ERROR" "Deployment command failed"; exit 1; }

if [ -n "$CONTRACT_ID" ]; then
    log "INFO" "Contract deployed successfully!"
    log "INFO" "Contract ID: $CONTRACT_ID"
    log "INFO" "Network: $NETWORK"
    log "INFO" "Deployer: $IDENTITY ($IDENTITY_ADDRESS)"
     
    # Save deployment info
    DEPLOY_INFO_FILE="deployments/${NETWORK}_${CONTRACT_NAME}.json"
    mkdir -p deployments || { log "ERROR" "Failed to create deployments dir"; exit 1; }
     
    cat > "$DEPLOY_INFO_FILE" << EOF
{
    "contract_name": "$CONTRACT_NAME",
    "contract_id": "$CONTRACT_ID",
    "network": "$NETWORK",
    "deployer": "$IDENTITY",
    "deployer_address": "$IDENTITY_ADDRESS",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "wasm_hash": "$(sha256sum "$CONTRACT_DIR/$WASM_FILE" | cut -d' ' -f1)"
}
EOF
     
    log "INFO" "Deployment info saved to: $DEPLOY_INFO_FILE"
     
    # Initialize contract if it has an initialize function
    log "INFO" "Attempting to initialize contract..."
    if soroban contract invoke \
        --id "$CONTRACT_ID" \
        --source "$IDENTITY" \
        --network "$NETWORK" \
        -- initialize 2>/dev/null; then
        log "INFO" "Contract initialized successfully"
    else
        log "WARN" "Contract initialization failed or not required"
    fi
     
    # Automated Verification Hook
    log "INFO" "Running automated verification..."
    if ! ./scripts/verify_deployment.sh "$CONTRACT_ID" "$NETWORK" "$IDENTITY" "$CONTRACT_NAME"; then
        log "ERROR" "Automated verification failed!"
        exit 1
    fi
     
else
    log "ERROR" "Contract deployment failed (empty CONTRACT_ID)"
    exit 1
fi

log "INFO" "Deployment complete! 🚀"
log "INFO" "You can now interact with your contract using:"
log "INFO" "soroban contract invoke --id $CONTRACT_ID --source $IDENTITY --network $NETWORK -- <function_name> [args...]"