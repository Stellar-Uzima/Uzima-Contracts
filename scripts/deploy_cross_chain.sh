#!/bin/bash

# deploy_cross_chain.sh - Cross-Chain Medical Records Interoperability Deployment Script
# This script deploys and configures the cross-chain bridge, identity, and access contracts
# Usage: ./scripts/deploy_cross_chain.sh <network> [identity]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_status() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }
print_header() { echo -e "${CYAN}=== $1 ===${NC}"; }

# Check arguments
if [ $# -lt 1 ]; then
    print_error "Usage: $0 <network> [identity]"
    print_error "Example: $0 testnet deployer"
    print_error "Available networks: local, testnet, futurenet, mainnet"
    exit 1
fi

NETWORK="$1"
IDENTITY="${2:-"cross_chain_deployer"}"

# Contract names
BRIDGE_CONTRACT="cross_chain_bridge"
IDENTITY_CONTRACT="cross_chain_identity"
ACCESS_CONTRACT="cross_chain_access"
MEDICAL_CONTRACT="medical_records"

# Directories
CONTRACTS_DIR="contracts"
DEPLOYMENTS_DIR="deployments"

print_header "Cross-Chain Medical Records Deployment"
print_status "Network: $NETWORK"
print_status "Identity: $IDENTITY"

# Function to build a single contract
build_contract() {
    local contract_name=$1
    local contract_dir="$CONTRACTS_DIR/$contract_name"

    print_step "Building $contract_name..."

    if [ ! -d "$contract_dir" ]; then
        print_error "Contract directory '$contract_dir' does not exist"
        return 1
    fi

    cd "$contract_dir"
    cargo build --target wasm32-unknown-unknown --release
    cd - > /dev/null

    local wasm_file="$contract_dir/target/wasm32-unknown-unknown/release/${contract_name}.wasm"
    if [ ! -f "$wasm_file" ]; then
        print_error "Build failed: $wasm_file not found"
        return 1
    fi

    # Optimize if possible
    if command -v soroban &> /dev/null; then
        soroban contract optimize --wasm "$wasm_file" 2>/dev/null || true
    fi

    print_status "$contract_name built successfully"
}

# Function to deploy a single contract
deploy_contract() {
    local contract_name=$1
    local wasm_file="$CONTRACTS_DIR/$contract_name/target/wasm32-unknown-unknown/release/${contract_name}.wasm"

    print_step "Deploying $contract_name..."

    local contract_id
    contract_id=$(soroban contract deploy \
        --wasm "$wasm_file" \
        --source "$IDENTITY" \
        --network "$NETWORK")

    if [ -z "$contract_id" ]; then
        print_error "Failed to deploy $contract_name"
        return 1
    fi

    print_status "$contract_name deployed: $contract_id"
    echo "$contract_id"
}

# Configure network
print_header "Network Configuration"
case $NETWORK in
    "local")
        soroban config network add local \
            --rpc-url http://localhost:8000/soroban/rpc \
            --network-passphrase "Standalone Network ; February 2017" 2>/dev/null || true
        ;;
    "testnet")
        soroban config network add testnet \
            --rpc-url https://soroban-testnet.stellar.org:443 \
            --network-passphrase "Test SDF Network ; September 2015" 2>/dev/null || true
        ;;
    "futurenet")
        soroban config network add futurenet \
            --rpc-url https://rpc-futurenet.stellar.org:443 \
            --network-passphrase "Test SDF Future Network ; October 2022" 2>/dev/null || true
        ;;
    "mainnet")
        soroban config network add mainnet \
            --rpc-url https://soroban-rpc.stellar.org:443 \
            --network-passphrase "Public Global Stellar Network ; September 2015" 2>/dev/null || true
        ;;
    *)
        print_error "Unknown network: $NETWORK"
        exit 1
        ;;
esac

# Ensure identity exists
print_step "Setting up identity..."
if ! soroban config identity show "$IDENTITY" &> /dev/null; then
    print_warning "Identity '$IDENTITY' not found, generating..."
    soroban config identity generate "$IDENTITY"
fi

IDENTITY_ADDRESS=$(soroban config identity address "$IDENTITY")
print_status "Deployer address: $IDENTITY_ADDRESS"

# Fund account on testnet/futurenet
if [ "$NETWORK" = "testnet" ] || [ "$NETWORK" = "futurenet" ]; then
    print_step "Funding account..."
    soroban config identity fund "$IDENTITY" --network "$NETWORK" || true
fi

# Build all contracts
print_header "Building Contracts"
build_contract "$BRIDGE_CONTRACT"
build_contract "$IDENTITY_CONTRACT"
build_contract "$ACCESS_CONTRACT"

# Deploy all contracts
print_header "Deploying Contracts"
BRIDGE_ID=$(deploy_contract "$BRIDGE_CONTRACT")
IDENTITY_ID=$(deploy_contract "$IDENTITY_CONTRACT")
ACCESS_ID=$(deploy_contract "$ACCESS_CONTRACT")

# Check if medical_records is already deployed
print_step "Checking for existing medical_records deployment..."
MEDICAL_ID=""
MEDICAL_DEPLOY_FILE="$DEPLOYMENTS_DIR/${NETWORK}_${MEDICAL_CONTRACT}.json"
if [ -f "$MEDICAL_DEPLOY_FILE" ]; then
    MEDICAL_ID=$(jq -r '.contract_id' "$MEDICAL_DEPLOY_FILE" 2>/dev/null || echo "")
    if [ -n "$MEDICAL_ID" ]; then
        print_status "Found existing medical_records: $MEDICAL_ID"
    fi
fi

# Initialize contracts
print_header "Initializing Contracts"

# Initialize Bridge Contract
print_step "Initializing $BRIDGE_CONTRACT..."
soroban contract invoke \
    --id "$BRIDGE_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- initialize \
    --admin "$IDENTITY_ADDRESS" \
    --medical_contract "$MEDICAL_ID" \
    --identity_contract "$IDENTITY_ID" \
    --access_contract "$ACCESS_ID" || print_warning "Bridge initialization may have failed"

# Initialize Identity Contract
print_step "Initializing $IDENTITY_CONTRACT..."
soroban contract invoke \
    --id "$IDENTITY_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- initialize \
    --admin "$IDENTITY_ADDRESS" \
    --bridge_contract "$BRIDGE_ID" || print_warning "Identity initialization may have failed"

# Initialize Access Contract
print_step "Initializing $ACCESS_CONTRACT..."
soroban contract invoke \
    --id "$ACCESS_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- initialize \
    --admin "$IDENTITY_ADDRESS" \
    --bridge_contract "$BRIDGE_ID" \
    --identity_contract "$IDENTITY_ID" || print_warning "Access initialization may have failed"

# Configure medical_records with cross-chain contracts (if exists)
if [ -n "$MEDICAL_ID" ]; then
    print_step "Configuring medical_records with cross-chain contracts..."
    soroban contract invoke \
        --id "$MEDICAL_ID" \
        --source "$IDENTITY" \
        --network "$NETWORK" \
        -- set_cross_chain_contracts \
        --caller "$IDENTITY_ADDRESS" \
        --bridge_contract "$BRIDGE_ID" \
        --identity_contract "$IDENTITY_ID" \
        --access_contract "$ACCESS_ID" || print_warning "Medical records cross-chain config may have failed"
fi

# Save deployment information
print_header "Saving Deployment Info"
mkdir -p "$DEPLOYMENTS_DIR"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Save individual contract deployments
for contract_name in "$BRIDGE_CONTRACT" "$IDENTITY_CONTRACT" "$ACCESS_CONTRACT"; do
    contract_var="${contract_name^^}_ID"
    contract_var="${contract_var//_/}"

    case $contract_name in
        "$BRIDGE_CONTRACT") contract_id="$BRIDGE_ID" ;;
        "$IDENTITY_CONTRACT") contract_id="$IDENTITY_ID" ;;
        "$ACCESS_CONTRACT") contract_id="$ACCESS_ID" ;;
    esac

    cat > "$DEPLOYMENTS_DIR/${NETWORK}_${contract_name}.json" << EOF
{
    "contract_name": "$contract_name",
    "contract_id": "$contract_id",
    "network": "$NETWORK",
    "deployer": "$IDENTITY",
    "deployer_address": "$IDENTITY_ADDRESS",
    "deployed_at": "$TIMESTAMP"
}
EOF
done

# Save cross-chain configuration summary
cat > "$DEPLOYMENTS_DIR/${NETWORK}_cross_chain_config.json" << EOF
{
    "network": "$NETWORK",
    "deployer": "$IDENTITY",
    "deployer_address": "$IDENTITY_ADDRESS",
    "deployed_at": "$TIMESTAMP",
    "contracts": {
        "bridge": "$BRIDGE_ID",
        "identity": "$IDENTITY_ID",
        "access": "$ACCESS_ID",
        "medical_records": "${MEDICAL_ID:-"not_deployed"}"
    },
    "configuration": {
        "bridge_initialized": true,
        "identity_initialized": true,
        "access_initialized": true,
        "medical_records_configured": ${MEDICAL_ID:+true}${MEDICAL_ID:-false}
    }
}
EOF

print_status "Deployment info saved to $DEPLOYMENTS_DIR/${NETWORK}_cross_chain_config.json"

# Print summary
print_header "Deployment Summary"
echo ""
echo "Cross-Chain Bridge:    $BRIDGE_ID"
echo "Cross-Chain Identity:  $IDENTITY_ID"
echo "Cross-Chain Access:    $ACCESS_ID"
if [ -n "$MEDICAL_ID" ]; then
    echo "Medical Records:       $MEDICAL_ID (configured)"
else
    echo "Medical Records:       Not deployed"
fi
echo ""
print_status "Deployment complete!"

# Print usage examples
print_header "Usage Examples"
echo ""
echo "# Add a validator to the bridge:"
echo "soroban contract invoke --id $BRIDGE_ID --source $IDENTITY --network $NETWORK -- add_validator --caller $IDENTITY_ADDRESS --validator_address <VALIDATOR_ADDRESS> --public_key <PUBLIC_KEY_32_BYTES> --initial_stake 1000"
echo ""
echo "# Add a supported chain (e.g., Avalanche):"
echo "soroban contract invoke --id $BRIDGE_ID --source $IDENTITY --network $NETWORK -- add_supported_chain --caller $IDENTITY_ADDRESS --chain Avalanche"
echo ""
echo "# Grant cross-chain access to a record:"
echo "soroban contract invoke --id $ACCESS_ID --source $IDENTITY --network $NETWORK -- grant_access --grantor <PATIENT_ADDRESS> --grantee_chain Ethereum --grantee_address '0x...' --permission_level Read --record_scope AllRecords --duration 2592000 --conditions '[]'"
echo ""
