#!/bin/bash

# deploy.sh - Soroban Contract Deployment Script
# Usage: ./scripts/deploy.sh <contract_name> <network> [identity]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
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
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract_name> <network> [identity]"
    print_error "Example: $0 medical_records testnet alice"
    print_error "Available networks: local, testnet, futurenet, mainnet"
    exit 1
fi

CONTRACT_NAME=$1
NETWORK=$2
IDENTITY=${3:-"default"}

# Validate contract exists
CONTRACT_DIR="contracts/$CONTRACT_NAME"
if [ ! -d "$CONTRACT_DIR" ]; then
    print_error "Contract directory '$CONTRACT_DIR' does not exist"
    exit 1
fi

print_status "Starting deployment of '$CONTRACT_NAME' to '$NETWORK' network"

# Build the contract
print_step "Building contract..."
cd "$CONTRACT_DIR"

# Clean previous builds
cargo clean

# Build for WebAssembly target
cargo build --target wasm32-unknown-unknown --release

# Check if build was successful
WASM_FILE="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
if [ ! -f "$WASM_FILE" ]; then
    print_error "Build failed: $WASM_FILE not found"
    exit 1
fi

print_status "Contract built successfully"

# Optimize the contract (if soroban contract optimize is available)
if command -v soroban &> /dev/null; then
    print_step "Optimizing contract..."
    soroban contract optimize --wasm "$WASM_FILE" || {
        print_warning "Optimization failed, continuing with unoptimized contract"
    }
fi

cd - > /dev/null

# Configure network if not already configured
print_step "Configuring network..."
case $NETWORK in
    "local")
        soroban config network add local \
            --rpc-url http://localhost:8000/soroban/rpc \
            --network-passphrase "Standalone Network ; February 2017" \
            2>/dev/null || print_warning "Network 'local' already configured"
        ;;
    "testnet")
        soroban config network add testnet \
            --rpc-url https://soroban-testnet.stellar.org:443 \
            --network-passphrase "Test SDF Network ; September 2015" \
            2>/dev/null || print_warning "Network 'testnet' already configured"
        ;;
    "futurenet")
        soroban config network add futurenet \
            --rpc-url https://rpc-futurenet.stellar.org:443 \
            --network-passphrase "Test SDF Future Network ; October 2022" \
            2>/dev/null || print_warning "Network 'futurenet' already configured"
        ;;
    "mainnet")
        soroban config network add mainnet \
            --rpc-url https://soroban-rpc.stellar.org:443 \
            --network-passphrase "Public Global Stellar Network ; September 2015" \
            2>/dev/null || print_warning "Network 'mainnet' already configured"
        ;;
    *)
        print_error "Unknown network: $NETWORK"
        print_error "Available networks: local, testnet, futurenet, mainnet"
        exit 1
        ;;
esac

# Ensure identity exists
print_step "Checking identity..."
if ! soroban config identity show "$IDENTITY" &> /dev/null; then
    print_warning "Identity '$IDENTITY' not found, generating new one..."
    soroban config identity generate "$IDENTITY"
fi

# Get identity address
IDENTITY_ADDRESS=$(soroban config identity address "$IDENTITY")
print_status "Using identity: $IDENTITY ($IDENTITY_ADDRESS)"

# Fund account for testnet/futurenet
if [ "$NETWORK" = "testnet" ] || [ "$NETWORK" = "futurenet" ]; then
    print_step "Funding account on $NETWORK..."
    soroban config identity fund "$IDENTITY" --network "$NETWORK" || {
        print_warning "Failed to fund account, continuing anyway..."
    }
fi

# Deploy the contract
print_step "Deploying contract..."
CONTRACT_ID=$(soroban contract deploy \
    --wasm "$CONTRACT_DIR/$WASM_FILE" \
    --source "$IDENTITY" \
    --network "$NETWORK" 2>/dev/null)

if [ $? -eq 0 ] && [ -n "$CONTRACT_ID" ]; then
    print_status "Contract deployed successfully!"
    print_status "Contract ID: $CONTRACT_ID"
    print_status "Network: $NETWORK"
    print_status "Deployer: $IDENTITY ($IDENTITY_ADDRESS)"
    
    # Save deployment info
    DEPLOY_INFO_FILE="deployments/${NETWORK}_${CONTRACT_NAME}.json"
    mkdir -p deployments
    
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
    
    print_status "Deployment info saved to: $DEPLOY_INFO_FILE"
    
    # Initialize contract if it has an initialize function
    print_step "Attempting to initialize contract..."
    soroban contract invoke \
        --id "$CONTRACT_ID" \
        --source "$IDENTITY" \
        --network "$NETWORK" \
        -- initialize 2>/dev/null && {
        print_status "Contract initialized successfully"
    } || {
        print_warning "Contract initialization failed or not required"
    }
    
else
    print_error "Contract deployment failed"
    exit 1
fi

print_status "Deployment complete! 🚀"
print_status "You can now interact with your contract using:"
print_status "soroban contract invoke --id $CONTRACT_ID --source $IDENTITY --network $NETWORK -- <function_name> [args...]"