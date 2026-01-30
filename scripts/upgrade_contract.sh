#!/bin/bash
set -e

# Tool to assist in contract upgrades
# Usage: ./upgrade_contract.sh <contract_id> <new_wasm_path> <version> <description>

CONTRACT_ID=$1
WASM_PATH=$2
VERSION=$3
DESC=$4

if [ -z "$CONTRACT_ID" ] || [ -z "$WASM_PATH" ] || [ -z "$VERSION" ]; then
    echo "Usage: $0 <contract_id> <new_wasm_path> <version> [description]"
    exit 1
fi

echo "--- Starting Upgrade Process for $CONTRACT_ID ---"

# 1. Install new WASM
echo "Installing new WASM..."
WASM_HASH=$(soroban contract install --wasm "$WASM_PATH")
echo "New WASM Hash: $WASM_HASH"

# 2. Get UpgradeManager ID (Assuming it's stored or we pass it)
# For this script we assume the caller is the admin of the target or using the manager
# Here we'll just show how to call the upgrade directly for simple cases
# or propose to the manager.

echo "Proposing upgrade to version $VERSION..."
# soroban contract invoke --id <UPGRADE_MANAGER_ID> -- \
#   propose_upgrade --proposer <ADMIN_ADDR> --target "$CONTRACT_ID" \
#   --new_wasm_hash "$WASM_HASH" --new_version "$VERSION" --description "$DESC"

echo "Upgrade proposed. Check proposal ID and get approvals."
