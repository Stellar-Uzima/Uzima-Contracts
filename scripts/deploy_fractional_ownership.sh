#!/bin/bash

# =============================================================================
# FractionalOwnership Deployment Script
# =============================================================================
# Builds, deploys and initializes the fractional_ownership contract.
#
# Usage: ./scripts/deploy_fractional_ownership.sh <network> <payment_token> [identity]
#   network:       local, testnet, futurenet, or mainnet
#   payment_token: contract id of the token used for purchases and payouts
#   identity:      (optional) Stellar identity to deploy with (default: "default")
#
# Example:
#   ./scripts/deploy_fractional_ownership.sh testnet CDLZ...TOKEN my-deployer
# =============================================================================

set -e

NETWORK=${1:-testnet}
PAYMENT_TOKEN=$2
IDENTITY=${3:-default}
WASM=target/wasm32-unknown-unknown/release/fractional_ownership.wasm
OPTIMIZED_WASM=target/wasm32-unknown-unknown/release/fractional_ownership.optimized.wasm

if [ -z "$PAYMENT_TOKEN" ]; then
    echo "Usage: $0 <network> <payment_token> [identity]"
    exit 1
fi

case $NETWORK in
    local)
        RPC_URL="http://localhost:8000/soroban/rpc"
        NETWORK_PASSPHRASE="Standalone Network ; February 2017"
        ;;
    testnet)
        RPC_URL="https://soroban-testnet.stellar.org"
        NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
        ;;
    futurenet)
        RPC_URL="https://rpc-futurenet.stellar.org"
        NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
        ;;
    mainnet)
        RPC_URL="https://soroban-mainnet.stellar.org"
        NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
        read -p "Deploying to MAINNET costs real XLM. Continue? (y/N) " -n 1 -r
        echo
        [[ $REPLY =~ ^[Yy]$ ]] || exit 1
        ;;
    *)
        echo "Invalid network: $NETWORK (use local, testnet, futurenet or mainnet)"
        exit 1
        ;;
esac

stellar network add \
    --global "$NETWORK" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    2>/dev/null || true

if ! stellar keys address "$IDENTITY" &>/dev/null; then
    stellar keys generate --global "$IDENTITY" --network "$NETWORK"
fi
ADMIN_ADDRESS=$(stellar keys address "$IDENTITY")

cd "$(dirname "$0")/.."

echo "==> Building fractional_ownership..."
cargo build --release --target wasm32-unknown-unknown -p fractional_ownership
stellar contract optimize --wasm "$WASM" --wasm-out "$OPTIMIZED_WASM"

echo "==> Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm "$OPTIMIZED_WASM" \
    --source "$IDENTITY" \
    --network "$NETWORK")

echo "==> Initializing..."
stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$ADMIN_ADDRESS" \
    --payment_token "$PAYMENT_TOKEN"

echo ""
echo "FractionalOwnership deployed"
echo "  Contract ID:   $CONTRACT_ID"
echo "  Network:       $NETWORK"
echo "  Admin:         $ADMIN_ADDRESS"
echo "  Payment token: $PAYMENT_TOKEN"
