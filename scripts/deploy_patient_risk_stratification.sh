#!/bin/bash

# Patient Risk Stratification Contract Deployment Script
# Usage: ./scripts/deploy_patient_risk_stratification.sh <network> [contract_id]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

NETWORK=$1
CONTRACT_ID=$2

if [ -z "$NETWORK" ]; then
    echo "Usage: $0 <network> [contract_id]"
    echo "Networks: local, testnet, futurenet"
    exit 1
fi

# Check if Soroban CLI is available
if ! command -v soroban &> /dev/null; then
    echo "Error: Soroban CLI not found. Please install it first."
    exit 1
fi

# Set network
case $NETWORK in
    local)
        NETWORK_PASSPHRASE="Standalone Network ; February 2017"
        RPC_URL="http://localhost:8000/soroban/rpc"
        ;;
    testnet)
        NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
        RPC_URL="https://soroban-testnet.stellar.org:443"
        ;;
    futurenet)
        NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
        RPC_URL="https://rpc-futurenet.stellar.org:443"
        ;;
    *)
        echo "Error: Unknown network '$NETWORK'"
        exit 1
        ;;
esac

echo "Deploying Patient Risk Stratification contract to $NETWORK..."

# Build the contract
echo "Building contract..."
cd "$PROJECT_ROOT"
cargo build --release --target wasm32-unknown-unknown

# Deploy the contract
echo "Deploying contract..."
if [ -n "$CONTRACT_ID" ]; then
    # If contract ID is provided, install the WASM and create contract
    WASM_HASH=$(soroban contract install \
        --wasm target/wasm32-unknown-unknown/release/patient_risk_stratification.wasm \
        --network $NETWORK \
        --source default)

    echo "WASM installed with hash: $WASM_HASH"

    soroban contract deploy \
        --wasm-hash $WASM_HASH \
        --id $CONTRACT_ID \
        --network $NETWORK \
        --source default

    CONTRACT_ID=$CONTRACT_ID
else
    # Deploy without specifying ID
    CONTRACT_ID=$(soroban contract deploy \
        --wasm target/wasm32-unknown-unknown/release/patient_risk_stratification.wasm \
        --network $NETWORK \
        --source default)
fi

echo "Contract deployed successfully!"
echo "Contract ID: $CONTRACT_ID"
echo "Network: $NETWORK"

# Initialize the contract
echo "Initializing contract..."
ADMIN_ADDRESS=$(soroban config identity address --source default)

soroban contract invoke \
    --id $CONTRACT_ID \
    --network $NETWORK \
    --source default \
    -- \
    initialize \
    --admin "$ADMIN_ADDRESS"

echo "Contract initialized successfully!"
echo ""
echo "Deployment Summary:"
echo "==================="
echo "Contract ID: $CONTRACT_ID"
echo "Network: $NETWORK"
echo "Admin: $ADMIN_ADDRESS"
echo ""
echo "To interact with the contract, use:"
echo "soroban contract invoke --id $CONTRACT_ID --network $NETWORK --source default -- <function> [args]"