#!/bin/bash
set -euo pipefail

# upgrade_contract.sh - Install a new WASM hash and scaffold a migration plan.
# Usage: ./scripts/upgrade_contract.sh <contract_name> <network> <new_wasm_path> <from_version> <to_version> [--identity <name>]

CONTRACT_NAME="${1:-}"
NETWORK="${2:-}"
WASM_PATH="${3:-}"
FROM_VERSION="${4:-}"
TO_VERSION="${5:-}"
IDENTITY="default"

shift $(( $# > 5 ? 5 : $# ))

while [[ $# -gt 0 ]]; do
    case "$1" in
        --identity)
            IDENTITY="${2:-}"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Usage: $0 <contract_name> <network> <new_wasm_path> <from_version> <to_version> [--identity <name>]" >&2
            exit 1
            ;;
    esac
done

if [[ -z "$CONTRACT_NAME" || -z "$NETWORK" || -z "$WASM_PATH" || -z "$FROM_VERSION" || -z "$TO_VERSION" ]]; then
    echo "Usage: $0 <contract_name> <network> <new_wasm_path> <from_version> <to_version> [--identity <name>]" >&2
    exit 1
fi

if ! command -v soroban >/dev/null 2>&1; then
    echo "soroban CLI is required" >&2
    exit 1
fi

PLAN_DIR="deployments/$NETWORK/$CONTRACT_NAME"
PLAN_PATH="$PLAN_DIR/plan.json"

echo "--- Preparing upgrade assets for $CONTRACT_NAME on $NETWORK ---"
echo "Installing new WASM..."
WASM_HASH=$(soroban contract install --wasm "$WASM_PATH" --source "$IDENTITY" --network "$NETWORK")
echo "New WASM Hash: $WASM_HASH"

mkdir -p "$PLAN_DIR"

cat > "$PLAN_PATH" <<EOF
{
  "contract_name": "$CONTRACT_NAME",
  "network": "$NETWORK",
  "contract_id": "REPLACE_WITH_DEPLOYED_CONTRACT_ID",
  "admin_identity": "$IDENTITY",
  "upgrade_entrypoint": "upgrade",
  "caller_arg_name": "caller",
  "new_wasm_hash": "$WASM_HASH",
  "version_bump": {
    "from": $FROM_VERSION,
    "to": $TO_VERSION
  },
  "expected_storage_migration_steps": [
    {
      "action": "update",
      "key": "ContractVersion",
      "from": "$FROM_VERSION",
      "to": "$TO_VERSION",
      "description": "Replace this placeholder with the actual schema bump and migration details."
    }
  ],
  "expected_gas": {
    "cpu_instructions": 0,
    "read_bytes": 0,
    "write_bytes": 0,
    "transaction_fee_stroops": 0
  },
  "expected_event_emissions": [
    {
      "topic": "Upgrade",
      "data": "Replace with the expected upgrade event payload."
    }
  ],
  "notes": [
    "Fill in contract_id, gas, storage diff, and event expectations before running migrate_contract.sh."
  ]
}
EOF

echo "Migration plan scaffolded at $PLAN_PATH"
echo "Next steps:"
echo "  1. Review and complete $PLAN_PATH"
echo "  2. Run ./scripts/migrate_contract.sh $CONTRACT_NAME --network $NETWORK --dry-run"
echo "  3. Run ./scripts/migrate_contract.sh $CONTRACT_NAME --network $NETWORK --i-understand-this-is-live"

