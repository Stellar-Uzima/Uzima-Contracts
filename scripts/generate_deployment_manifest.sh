#!/usr/bin/env bash
# generate_deployment_manifest.sh
#
# Generates or validates a registry-driven deployment manifest for all
# Uzima contracts across supported networks.
#
# Usage:
#   ./scripts/generate_deployment_manifest.sh [--validate] [--network NETWORK]
#
# Options:
#   --validate       Validate existing manifest against current Cargo workspace
#   --network NAME   Filter output for a single network (local|testnet|futurenet|mainnet)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$REPO_ROOT/deployments/deployment-manifest.json"
CONTRACTS_DIR="$REPO_ROOT/contracts"

VALIDATE=false
NETWORK=""

for arg in "$@"; do
  case "$arg" in
    --validate) VALIDATE=true ;;
    --network) shift; NETWORK="$1" ;;
  esac
done

echo "=== Uzima Deployment Manifest Generator ==="
echo ""

if [[ "$VALIDATE" == "true" ]]; then
  echo "Validating deployment manifest against workspace..."
  echo ""

  # Check every contract in manifest exists on disk
  if command -v jq &>/dev/null; then
    REGISTERED=$(jq -r '.contracts[].name' "$MANIFEST" 2>/dev/null || echo "")
    MISSING=0
    while IFS= read -r contract; do
      if [[ ! -d "$CONTRACTS_DIR/$contract" ]]; then
        echo "  WARNING: Manifest references '$contract' but directory not found"
        MISSING=$((MISSING + 1))
      fi
    done <<< "$REGISTERED"

    if [[ "$MISSING" -eq 0 ]]; then
      echo "  ✅ All manifest contracts found in workspace"
    else
      echo "  ⚠️  $MISSING contracts referenced in manifest but not found locally"
    fi

    # Check for contracts on disk not in manifest
    UNREGISTERED=0
    for contract_dir in "$CONTRACTS_DIR"/*/; do
      contract_name=$(basename "$contract_dir")
      if [[ -f "$contract_dir/Cargo.toml" ]]; then
        if ! jq -e ".contracts[] | select(.name == \"$contract_name\")" "$MANIFEST" &>/dev/null; then
          echo "  INFO: '$contract_name' not registered in manifest (optional)"
          UNREGISTERED=$((UNREGISTERED + 1))
        fi
      fi
    done
    echo ""
    echo "  Registered: $(jq '.contracts | length' "$MANIFEST") contracts"
    echo "  Unregistered: $UNREGISTERED contracts (not required to be in manifest)"
  else
    echo "  jq not installed — skipping deep validation"
  fi

  echo ""
  echo "Validation complete."
  exit 0
fi

# Show manifest summary for requested network (or all)
if [[ -n "$NETWORK" ]]; then
  echo "Deployment plan for network: $NETWORK"
  echo ""
  if command -v jq &>/dev/null; then
    jq --arg net "$NETWORK" \
      '[.contracts[] | select(.networks[$net] != null) | {name: .name, tier: .tier, order: .deploy_order, init_required: .networks[$net].init_required}] | sort_by(.order)' \
      "$MANIFEST"
  else
    echo "  Install jq for formatted output."
    echo "  See deployments/deployment-manifest.json for full manifest."
  fi
else
  echo "Manifest: $MANIFEST"
  echo ""
  if command -v jq &>/dev/null; then
    echo "Contract count: $(jq '.contracts | length' "$MANIFEST")"
    echo "Networks: $(jq -r '.networks | keys | join(", ")' "$MANIFEST")"
    echo ""
    echo "Deploy order (all contracts):"
    jq -r '.contracts | sort_by(.deploy_order) | .[] | "  \(.deploy_order). [\(.tier)] \(.name)"' "$MANIFEST"
  else
    cat "$MANIFEST"
  fi
fi

echo ""
echo "Run with --validate to check manifest against workspace."
echo "Run with --network <name> to show the plan for a specific network."
