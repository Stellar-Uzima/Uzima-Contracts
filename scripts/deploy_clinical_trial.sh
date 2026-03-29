#!/usr/bin/env bash
# Simple deploy helper for clinical_trial (placeholder)
set -euo pipefail

CONTRACT_DIR="contracts/clinical_trial"
NETWORK=${1:-local}

echo "Building clinical_trial contract..."
pushd "$CONTRACT_DIR" >/dev/null
cargo build --release --target wasm32-unknown-unknown || true
popd >/dev/null

echo "(Placeholder) Use soroban CLI to deploy the compiled wasm to $NETWORK"
echo "soroban contract deploy --wasm $CONTRACT_DIR/target/wasm32-unknown-unknown/release/clinical_trial.wasm --network $NETWORK"
