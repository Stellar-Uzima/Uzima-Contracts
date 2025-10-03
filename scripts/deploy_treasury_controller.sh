#!/bin/bash

# Deploy Treasury Controller Contract to Stellar Testnet
# This script deploys and initializes the Multi-Sig Treasury Controller

set -e

echo "🏛️ Deploying Treasury Controller Contract..."

# Build contracts
echo "📦 Building contracts..."
soroban contract build

# Deploy Treasury Controller
echo "🔐 Deploying Treasury Controller..."
TREASURY_CONTROLLER_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/treasury_controller.wasm \
    --source alice \
    --network testnet)

echo "Treasury Controller deployed at: $TREASURY_CONTROLLER_ID"

# Deploy SUT Token for testing (if not already deployed)
echo "🪙 Deploying SUT Token for treasury management..."
SUT_TOKEN_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/sut_token.wasm \
    --source alice \
    --network testnet)

echo "SUT Token deployed at: $SUT_TOKEN_ID"

# Initialize SUT Token
echo "🔧 Initializing SUT Token..."
soroban contract invoke \
    --id $SUT_TOKEN_ID \
    --source alice \
    --network testnet \
    -- \
    initialize \
    --admin alice \
    --name "Stellar Utility Token" \
    --symbol "SUT" \
    --decimals 6 \
    --supply_cap 1000000000000

# Set up multisig signers (replace with actual addresses for production)
SIGNER1="GCDNJUBQSX7AJWLJACMJ7I4BC3Z47BQUTMHEICZLE6MU4KQBRYG5JY6B"  # Alice
SIGNER2="GDXLKEY5TR4IDEV7FZWYFG6MA6M24YDCX5HENQ7DTESBE233FOQIUWR"  # Bob  
SIGNER3="GCJXTTQNUCL7NLRX2E7DWLVP7V4RYDGB6E5CVFYFFGJGFQAZCXZFLXPN"  # Charlie

echo "🔧 Initializing Treasury Controller..."
soroban contract invoke \
    --id $TREASURY_CONTROLLER_ID \
    --source alice \
    --network testnet \
    -- \
    initialize \
    --admin alice \
    --signers "[$SIGNER1,$SIGNER2,$SIGNER3]" \
    --threshold 2 \
    --timelock_duration 3600 \
    --emergency_threshold 2 \
    --max_withdrawal_amount 1000000000

echo "✅ Adding SUT Token as supported token..."
soroban contract invoke \
    --id $TREASURY_CONTROLLER_ID \
    --source alice \
    --network testnet \
    -- \
    add_supported_token \
    --token_address $SUT_TOKEN_ID

echo "🎉 Treasury Controller setup complete!"
echo ""
echo "📋 Deployment Summary:"
echo "  Treasury Controller: $TREASURY_CONTROLLER_ID"
echo "  SUT Token:          $SUT_TOKEN_ID"
echo "  Multisig Threshold: 2-of-3"
echo "  Timelock Duration:  1 hour"
echo "  Max Withdrawal:     1,000,000,000 stroops"
echo ""
echo "🔗 Next Steps:"
echo "1. Fund the treasury with tokens"
echo "2. Create test proposals via the contract interface"
echo "3. Test the approval and execution workflow"
echo "4. Verify Gnosis Safe compatibility"
echo ""
echo "📖 Usage Examples:"
echo ""
echo "# Create a withdrawal proposal:"
echo "soroban contract invoke --id $TREASURY_CONTROLLER_ID --source alice --network testnet -- create_proposal --proposer alice --proposal_type Withdrawal --target_address RECIPIENT_ADDRESS --token_contract $SUT_TOKEN_ID --amount 1000000 --purpose \"Development Funding\" --metadata \"Q1 Budget\" --execution_data \"\")"
echo ""
echo "# Approve a proposal:"
echo "soroban contract invoke --id $TREASURY_CONTROLLER_ID --source alice --network testnet -- approve_proposal --signer alice --proposal_id 1"
echo ""
echo "# Check proposal status:"
echo "soroban contract invoke --id $TREASURY_CONTROLLER_ID --source alice --network testnet -- get_proposal --proposal_id 1"
