#!/bin/bash

# Deploy Identity Registry Contract on Stellar/Soroban
# Usage: ./scripts/deploy_identity_registry.sh [OWNER_ADDRESS]

set -euo pipefail  # Exit on error, undefined vars, or pipe fail

# Input validation
if [ $# -gt 1 ]; then
    echo "❌ Error: Too many arguments. Usage: $0 [OWNER_ADDRESS]"
    exit 1
fi

# Configuration with defaults
NETWORK=${NETWORK:-"testnet"}
OWNER_ADDRESS=${1:-"GDIY6AQQ75WMD4W46EYB7O6UYMHOCGQHLAQGQTKHDX4J2DYQCHVCR4W4"}
if [[ ! "$OWNER_ADDRESS" =~ ^G[A-Z0-9]{55}$ ]]; then
    echo "❌ Error: Invalid OWNER_ADDRESS format. Must be a valid Stellar address (starts with G, 56 chars)."
    exit 1
fi

echo "🚀 Deploying Identity Registry Contract to Stellar $NETWORK..."
echo "Owner Address: $OWNER_ADDRESS"

# Build the contract
echo "📦 Building contract..."
if ! cd contracts/identity_registry; then
    echo "❌ Error: Failed to cd into contracts/identity_registry"
    exit 1
fi
if ! soroban contract build; then
    echo "❌ Error: Contract build failed"
    exit 1
fi
cd - > /dev/null || { echo "❌ Error: Failed to cd back"; exit 1; }

# Deploy the contract
echo "🌐 Deploying to $NETWORK..."
CONTRACT_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/identity_registry.wasm \
    --source-account "$OWNER_ADDRESS" \
    --network "$NETWORK") || { echo "❌ Error: Deployment failed"; exit 1; }

echo "Contract deployed with ID: $CONTRACT_ID"

# Initialize the contract
echo "🔧 Initializing contract..."
if ! soroban contract invoke \
    --id "$CONTRACT_ID" \
    --source-account "$OWNER_ADDRESS" \
    --network "$NETWORK" \
    -- \
    initialize \
    --owner "$OWNER_ADDRESS"; then
    echo "❌ Error: Contract initialization failed"
    exit 1
fi

echo "✅ Identity Registry Contract deployed and initialized successfully!"
echo "📋 Contract ID: $CONTRACT_ID"
echo "📋 Contract Features:"
echo "   - Identity hash registration with metadata (32-byte hashes)"
echo "   - Role-based verifier system with owner controls"
echo "   - Attestation creation and revocation by verified entities"
echo "   - Event emission for all operations (IdentityRegistered, Attested, Revoked)"
echo "   - Secure storage patterns with proper access controls"
echo "   - Support for multiple attestations per subject"
echo ""
echo "🔧 Usage Examples:"
echo "   # Register identity hash:"
echo "   soroban contract invoke --id $CONTRACT_ID --network $NETWORK -- register_identity_hash --hash <32_byte_hash> --subject <address> --meta \"Healthcare License\""
echo ""
echo "   # Add verifier (owner only):"
echo "   soroban contract invoke --id $CONTRACT_ID --network $NETWORK -- add_verifier --verifier <address>"
echo ""
echo "   # Create attestation (verifiers only):"
echo "   soroban contract invoke --id $CONTRACT_ID --network $NETWORK -- attest --subject <address> --claim_hash <32_byte_hash>"