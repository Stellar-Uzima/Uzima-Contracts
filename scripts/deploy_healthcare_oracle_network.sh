#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

NETWORK="${1:-testnet}"
SOURCE_ACCOUNT="${2:-}"
ADMIN_ADDRESS="${3:-}"
ARBITER_ADDRESS="${4:-}"
MIN_SUBMISSIONS="${5:-2}"

if [[ -z "$SOURCE_ACCOUNT" || -z "$ADMIN_ADDRESS" || -z "$ARBITER_ADDRESS" ]]; then
  echo "Usage: $0 <network> <source_account> <admin_address> <arbiter_address> [min_submissions]"
  echo "Example: $0 testnet alice GADMIN... GARB... 2"
  exit 1
fi

echo "Building healthcare_oracle_network contract..."
cd "$PROJECT_ROOT"
cargo build --release -p healthcare_oracle_network

WASM_PATH="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/healthcare_oracle_network.wasm"
if [[ ! -f "$WASM_PATH" ]]; then
  echo "WASM not found at: $WASM_PATH"
  exit 1
fi

echo "Deploying to network: $NETWORK"
CONTRACT_ID=$(soroban contract deploy \
  --wasm "$WASM_PATH" \
  --source-account "$SOURCE_ACCOUNT" \
  --network "$NETWORK")

echo "Deployed healthcare_oracle_network contract: $CONTRACT_ID"

echo "Initializing contract..."
soroban contract invoke \
  --id "$CONTRACT_ID" \
  --source-account "$SOURCE_ACCOUNT" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN_ADDRESS" \
  --arbiters "[\"$ARBITER_ADDRESS\"]" \
  --min_submissions "$MIN_SUBMISSIONS"

echo "Initialization complete."
echo "Contract ID: $CONTRACT_ID"
