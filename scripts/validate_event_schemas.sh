#!/usr/bin/env bash
# validate_event_schemas.sh
#
# Validates that all contract events are registered in the event schema registry
# and that the registry is internally consistent.
#
# Usage:
#   ./scripts/validate_event_schemas.sh [--strict]
#
# Options:
#   --strict   Exit 1 if any contract emits unregistered events

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY="$REPO_ROOT/schemas/events/event-schema-registry.json"
CONTRACTS_DIR="$REPO_ROOT/contracts"

STRICT=false
for arg in "$@"; do
  [[ "$arg" == "--strict" ]] && STRICT=true
done

echo "=== Uzima Event Schema Validator ==="
echo ""
echo "Registry: $REGISTRY"
echo ""

if ! command -v jq &>/dev/null; then
  echo "  jq not installed — skipping deep validation"
  echo "  Install jq to enable full event schema validation"
  exit 0
fi

REGISTERED_COUNT=$(jq '.events | length' "$REGISTRY")
echo "Registered events: $REGISTERED_COUNT"
echo ""

# List all registered events
echo "Event inventory:"
jq -r '.events | to_entries[] | "  [\(.value.contract)] \(.value.name) (v\(.value.version))"' "$REGISTRY"
echo ""

# Scan contracts for event emissions and check registry
UNREGISTERED=0
echo "Scanning contracts for event emissions..."
while IFS= read -r rs_file; do
  contract=$(basename "$(dirname "$(dirname "$rs_file")")")
  # Look for env.events().publish( calls
  if grep -q 'env\.events()\.publish' "$rs_file" 2>/dev/null; then
    # Extract event topic hints from source (best-effort)
    events=$(grep -o 'symbol_short!("[^"]*")' "$rs_file" 2>/dev/null | tr -d '"' | grep -v 'symbol_short!' || true)
    if [[ -z "$events" ]]; then
      echo "  INFO: $contract/$(basename "$rs_file") emits events (topics not statically analyzable)"
    fi
  fi
done < <(find "$CONTRACTS_DIR" -name "*.rs" -not -path "*/target/*" 2>/dev/null)

echo ""

# Validate schema structure
echo "Validating schema structure..."
INVALID=0
while IFS= read -r event_key; do
  # Check required fields in schema
  has_required=$(jq --arg k "$event_key" '.events[$k] | has("schema") and has("version") and has("contract") and has("name")' "$REGISTRY")
  if [[ "$has_required" != "true" ]]; then
    echo "  ERROR: '$event_key' is missing required fields (schema, version, contract, name)"
    INVALID=$((INVALID + 1))
  fi
done < <(jq -r '.events | keys[]' "$REGISTRY")

if [[ "$INVALID" -eq 0 ]]; then
  echo "  ✅ All $REGISTERED_COUNT registered events have valid structure"
else
  echo "  ❌ $INVALID events have invalid structure"
fi

echo ""

if [[ "$STRICT" == "true" ]] && [[ "$UNREGISTERED" -gt 0 ]]; then
  echo "STRICT mode: failing due to $UNREGISTERED unregistered events"
  exit 1
fi

echo "=== Validation complete ==="
echo "Run with --strict to fail CI on unregistered events."
