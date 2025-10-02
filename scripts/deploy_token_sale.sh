#!/bin/bash

# Deploy Token Sale Contract to Stellar Testnet
# Make sure you have soroban CLI installed and configured

set -e

echo "🚀 Deploying SUT Token Sale Contract..."

# Build contracts
echo "📦 Building contracts..."
soroban contract build

# Deploy SUT Token
echo "🪙 Deploying SUT Token..."
SUT_TOKEN_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/sut_token.wasm \
    --source alice \
    --network testnet)

echo "SUT Token deployed at: $SUT_TOKEN_ID"

# Deploy Token Sale Contract
echo "💰 Deploying Token Sale Contract..."
TOKEN_SALE_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/token_sale.wasm \
    --source alice \
    --network testnet)

echo "Token Sale Contract deployed at: $TOKEN_SALE_ID"

# Deploy Vesting Contract
echo "⏰ Deploying Vesting Contract..."
VESTING_ID=$(soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/token_sale.wasm \
    --source alice \
    --network testnet)

echo "Vesting Contract deployed at: $VESTING_ID"

# Initialize SUT Token
echo "🔧 Initializing SUT Token..."
soroban contract invoke \
    --id $SUT_TOKEN_ID \
    --source alice \
    --network testnet \
    -- \
    initialize \
    --admin alice \
    --decimal 6 \
    --name "Stellar Utility Token" \
    --symbol "SUT"

# Get treasury address (replace with your multisig)
TREASURY_ADDRESS="GCDNJUBQSX7AJWLJACMJ7I4BC3Z47BQUTMHEICZLE6MU4KQBRYG5JY6B"

# Initialize Token Sale
echo "🔧 Initializing Token Sale..."
soroban contract invoke \
    --id $TOKEN_SALE_ID \
    --source alice \
    --network testnet \
    -- \
    initialize \
    --owner alice \
    --token_address $SUT_TOKEN_ID \
    --treasury $TREASURY_ADDRESS \
    --soft_cap 100000000000 \
    --hard_cap 1000000000000

# Add USDC as supported payment token (Testnet USDC)
USDC_ADDRESS="CAQCFVLOBK5GIULPNZRGATJJMIZL5BSP7X5YJVMGBGQVNV3NPHQZPQXB"
echo "💳 Adding USDC as supported payment token..."
soroban contract invoke \
    --id $TOKEN_SALE_ID \
    --source alice \
    --network testnet \
    -- \
    add_supported_token \
    --token $USDC_ADDRESS

# Add initial sale phase (30 days from now)
START_TIME=$(($(date +%s) + 86400))  # 1 day from now
END_TIME=$((START_TIME + 2592000))   # 30 days duration
PRICE_PER_TOKEN=1000000              # 1 USDC per SUT token (6 decimals)
MAX_TOKENS=500000000000              # 500,000 SUT tokens
PER_ADDRESS_CAP=10000000000          # 10,000 USDC per address

echo "📅 Adding initial sale phase..."
soroban contract invoke \
    --id $TOKEN_SALE_ID \
    --source alice \
    --network testnet \
    -- \
    add_sale_phase \
    --start_time $START_TIME \
    --end_time $END_TIME \
    --price_per_token $PRICE_PER_TOKEN \
    --max_tokens $MAX_TOKENS \
    --per_address_cap $PER_ADDRESS_CAP

echo "✅ Deployment complete!"
echo ""
echo "📋 Contract Addresses:"
echo "SUT Token: $SUT_TOKEN_ID"
echo "Token Sale: $TOKEN_SALE_ID"
echo "Vesting: $VESTING_ID"
echo ""
echo "🔗 Add these to your frontend configuration"