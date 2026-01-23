#!/bin/bash

# =============================================================================
# DID-Enhanced Identity Registry Deployment Script
# =============================================================================
# This script deploys the W3C DID-compliant identity registry contract
# to the specified Stellar network.
#
# Features:
# - DID document management (create, resolve, update, deactivate)
# - Verifiable credentials for healthcare professionals
# - Identity recovery with guardian-based multisig
# - Key rotation mechanisms
# - Service endpoint management
#
# Usage: ./scripts/deploy_did_registry.sh <network> [identity]
#   network: local, testnet, futurenet, or mainnet
#   identity: (optional) Stellar identity to use for deployment
#
# Example:
#   ./scripts/deploy_did_registry.sh testnet
#   ./scripts/deploy_did_registry.sh mainnet my-deployer
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Parse arguments
NETWORK=${1:-testnet}
IDENTITY=${2:-default}

echo ""
echo "=============================================="
echo "  DID-Enhanced Identity Registry Deployment"
echo "=============================================="
echo ""

# Validate network
case $NETWORK in
    local|testnet|futurenet|mainnet)
        print_step "Deploying to network: $NETWORK"
        ;;
    *)
        print_error "Invalid network: $NETWORK"
        echo "Valid networks: local, testnet, futurenet, mainnet"
        exit 1
        ;;
esac

# Set network RPC URL
case $NETWORK in
    local)
        RPC_URL="http://localhost:8000/soroban/rpc"
        NETWORK_PASSPHRASE="Standalone Network ; February 2017"
        NETWORK_ID="local"
        ;;
    testnet)
        RPC_URL="https://soroban-testnet.stellar.org"
        NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
        NETWORK_ID="testnet"
        ;;
    futurenet)
        RPC_URL="https://rpc-futurenet.stellar.org"
        NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
        NETWORK_ID="futurenet"
        ;;
    mainnet)
        RPC_URL="https://soroban-mainnet.stellar.org"
        NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
        NETWORK_ID="mainnet"
        print_warning "Deploying to MAINNET - this will cost real XLM!"
        read -p "Are you sure you want to continue? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
        ;;
esac

# Configure network
print_step "Configuring network..."
stellar network add \
    --global $NETWORK \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    2>/dev/null || true
print_success "Network configured"

# Check identity
print_step "Checking identity: $IDENTITY"
if ! stellar keys address $IDENTITY &>/dev/null; then
    print_warning "Identity '$IDENTITY' not found, generating new identity..."
    stellar keys generate --global $IDENTITY --network $NETWORK
    print_success "Generated new identity"
fi

DEPLOYER_ADDRESS=$(stellar keys address $IDENTITY)
print_success "Using deployer address: $DEPLOYER_ADDRESS"

# Build the contract
print_step "Building identity_registry contract..."
cd "$(dirname "$0")/.."

cargo build --release --target wasm32-unknown-unknown -p identity_registry
print_success "Contract built"

# Optimize the WASM
print_step "Optimizing WASM..."
stellar contract optimize \
    --wasm target/wasm32-unknown-unknown/release/identity_registry.wasm \
    --wasm-out target/wasm32-unknown-unknown/release/identity_registry.optimized.wasm
print_success "WASM optimized"

# Deploy the contract
print_step "Deploying contract to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/identity_registry.optimized.wasm \
    --source $IDENTITY \
    --network $NETWORK)
print_success "Contract deployed: $CONTRACT_ID"

# Initialize the contract
print_step "Initializing contract with DID support..."
stellar contract invoke \
    --id $CONTRACT_ID \
    --source $IDENTITY \
    --network $NETWORK \
    -- \
    initialize \
    --owner $DEPLOYER_ADDRESS \
    --network_id $NETWORK_ID
print_success "Contract initialized"

# Output deployment info
echo ""
echo "=============================================="
echo "  Deployment Complete!"
echo "=============================================="
echo ""
echo "Contract ID:     $CONTRACT_ID"
echo "Network:         $NETWORK"
echo "Network ID:      $NETWORK_ID"
echo "Owner Address:   $DEPLOYER_ADDRESS"
echo ""
echo "DID Features Enabled:"
echo "  - W3C DID Document Management"
echo "  - Verifiable Credentials"
echo "  - Identity Recovery (24h timelock)"
echo "  - Key Rotation (1h cooldown)"
echo "  - Service Endpoint Management"
echo ""
echo "DID Method Format:"
echo "  did:stellar:uzima:$NETWORK_ID:<address_hash>"
echo ""
echo "Next Steps:"
echo "  1. Create DIDs for healthcare providers"
echo "  2. Issue verifiable credentials"
echo "  3. Link identity registry to medical records contract"
echo ""

# Save deployment info
DEPLOY_INFO_FILE="deployments/${NETWORK}_identity_registry.json"
mkdir -p deployments
cat > $DEPLOY_INFO_FILE << EOF
{
    "contract_id": "$CONTRACT_ID",
    "network": "$NETWORK",
    "network_id": "$NETWORK_ID",
    "owner": "$DEPLOYER_ADDRESS",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "features": {
        "did_documents": true,
        "verifiable_credentials": true,
        "identity_recovery": true,
        "key_rotation": true,
        "service_endpoints": true
    },
    "did_method": "did:stellar:uzima"
}
EOF
print_success "Deployment info saved to $DEPLOY_INFO_FILE"
