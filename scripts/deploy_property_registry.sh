#!/bin/bash

# Deploy the property_registry contract to a Stellar network.
#
# Usage:
#   ./scripts/deploy_property_registry.sh [network] [source_identity]
#
#   network          local | testnet | futurenet   (default: testnet)
#   source_identity  soroban CLI identity name     (default: alice)
#
# The script is idempotent per invocation only: it always uploads a fresh
# contract instance and writes the resulting id to
# `deployments/<network>_property_registry.json` so `scripts/rollback_deployment.sh`
# and `scripts/deployment_status.sh` can find it.

set -euo pipefail

NETWORK="${1:-testnet}"
SOURCE="${2:-alice}"

if [ "$NETWORK" != "local" ] && [ "$NETWORK" != "testnet" ] && [ "$NETWORK" != "futurenet" ]; then
    echo "❌ Error: unsupported network '$NETWORK' (expected local, testnet or futurenet)"
    exit 1
fi

WASM_PATH="target/wasm32-unknown-unknown/release/property_registry.wasm"
DEPLOYMENTS_DIR="deployments"
MANIFEST="${DEPLOYMENTS_DIR}/${NETWORK}_property_registry.json"

echo "🚀 Deploying property_registry contract"
echo "   network: ${NETWORK}"
echo "   source:  ${SOURCE}"
echo ""

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

echo "📦 Building contracts..."
if ! soroban contract build; then
    echo "❌ Error: contract build failed"
    exit 1
fi

if [ ! -f "$WASM_PATH" ]; then
    echo "❌ Error: expected wasm at ${WASM_PATH}"
    exit 1
fi

# ---------------------------------------------------------------------------
# Deploy
# ---------------------------------------------------------------------------

echo "📤 Uploading property_registry.wasm..."
property_registry_ID="$(soroban contract deploy \
    --wasm "$WASM_PATH" \
    --source "$SOURCE" \
    --network "$NETWORK")" || {
    echo "❌ Error: property_registry deployment failed"
    exit 1
}

echo "property_registry deployed at: ${property_registry_ID}"

# ---------------------------------------------------------------------------
# Initialize
# ---------------------------------------------------------------------------

echo "🔧 Initializing property_registry..."
if ! soroban contract invoke \
    --id "$property_registry_ID" \
    --source "$SOURCE" \
    --network "$NETWORK" \
    -- \
    initialize \
    --admin "$SOURCE" \
    --treasury "$SOURCE"; then
    echo "❌ Error: property_registry initialization failed"
    exit 1
fi

# ---------------------------------------------------------------------------
# Verify the upgrade-safety markers were written
# ---------------------------------------------------------------------------

echo "🔍 Verifying storage layout markers..."
STORAGE_VERSION="$(soroban contract invoke \
    --id "$property_registry_ID" \
    --source "$SOURCE" \
    --network "$NETWORK" \
    --rpc-result \
    -- \
    storage_version)"

if [ "$STORAGE_VERSION" != "1" ]; then
    echo "❌ Error: expected storage_version 1, got '${STORAGE_VERSION}'"
    exit 1
fi

echo "storage_version = ${STORAGE_VERSION}"

# ---------------------------------------------------------------------------
# Record
# ---------------------------------------------------------------------------

mkdir -p "$DEPLOYMENTS_DIR"

COMMIT_SHA="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
DEPLOYED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat > "$MANIFEST" <<EOF
{
  "contract_name": "property_registry",
  "contract_id": "${property_registry_ID}",
  "network": "${NETWORK}",
  "deployer": "${SOURCE}",
  "deployed_at": "${DEPLOYED_AT}",
  "commit_sha": "${COMMIT_SHA}",
  "storage_schema_version": 1
}
EOF

echo ""
echo "✅ Deployment complete"
echo "   contract: ${property_registry_ID}"
echo "   manifest: ${MANIFEST}"