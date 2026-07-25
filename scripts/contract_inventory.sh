#!/usr/bin/env bash
#
# contract_inventory.sh — Scan contracts/ and generate a JSON inventory.
#
# Usage:
#   ./scripts/contract_inventory.sh [output_path]
#
# Output defaults to stdout if no path is given.
# The script is idempotent and safe to re-run.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"
OUTPUT_PATH="${1:-}"

# Collect contract entries
ENTRIES=""

for cargo_toml in "$CONTRACTS_DIR"/*/Cargo.toml; do
  [[ -f "$cargo_toml" ]] || continue
  CONTRACT_DIR="$(dirname "$cargo_toml")"
  CONTRACT_NAME="$(basename "$CONTRACT_DIR")"

  # Skip directories without src/
  [[ -d "$CONTRACT_DIR/src" ]] || continue

  # Extract metadata from Cargo.toml
  PKG_NAME=$(grep -m1 '^name' "$cargo_toml" | sed 's/.*= *"\(.*\)"/\1/' || echo "$CONTRACT_NAME")
  PKG_VERSION=$(grep -m1 '^version' "$cargo_toml" | sed 's/.*= *"\(.*\)"/\1/' || echo "unknown")
  if echo "$PKG_VERSION" | grep -q 'workspace'; then
    PKG_VERSION="workspace"
  fi

  # Collect dependencies
  DEPS="[]"
  if grep -q '\[dependencies\]' "$cargo_toml"; then
    DEP_LIST=$(awk '/^\[dependencies\]/{found=1; next} /^\[/{found=0} found && /^[a-z_]/{gsub(/[ =].*/, ""); gsub(/"/, ""); printf "%s\n", $0}' "$cargo_toml" || true)
    if [[ -n "$DEP_LIST" ]]; then
      DEPS_ITEMS=""
      while IFS= read -r dep; do
        [[ -z "$dep" ]] && continue
        if [[ -n "$DEPS_ITEMS" ]]; then
          DEPS_ITEMS="$DEPS_ITEMS,"
        fi
        DEPS_ITEMS="$DEPS_ITEMS\"$dep\""
      done <<< "$DEP_LIST"
      DEPS="[$DEPS_ITEMS]"
    fi
  fi

  # Check WASM target availability
  HAS_WASM="false"
  if [[ -f "$CONTRACT_DIR/src/lib.rs" ]] && grep -q 'no_std' "$CONTRACT_DIR/src/lib.rs" 2>/dev/null; then
    HAS_WASM="true"
  fi

  # Health checks
  CAN_BUILD="false"
  HAS_TESTS="false"
  HAS_DOCS="false"

  if [[ -f "$CONTRACT_DIR/src/lib.rs" ]]; then
    CAN_BUILD="true"
  fi

  if [[ -f "$CONTRACT_DIR/src/test.rs" ]] || find "$CONTRACT_DIR" -name '*test*' -maxdepth 3 | grep -q . 2>/dev/null; then
    HAS_TESTS="true"
  fi

  # Check for doc comments in lib.rs
  if [[ -f "$CONTRACT_DIR/src/lib.rs" ]] && head -20 "$CONTRACT_DIR/src/lib.rs" | grep -q '//!' 2>/dev/null; then
    HAS_DOCS="true"
  fi

  # Relative path from project root
  REL_PATH="contracts/$CONTRACT_NAME"

  ENTRY="{
    \"name\": \"$PKG_NAME\",
    \"path\": \"$REL_PATH\",
    \"version\": \"$PKG_VERSION\",
    \"dependencies\": $DEPS,
    \"wasm_target\": $HAS_WASM,
    \"health\": {
      \"can_build\": $CAN_BUILD,
      \"has_tests\": $HAS_TESTS,
      \"has_docs\": $HAS_DOCS
    }
  }"

  if [[ -n "$ENTRIES" ]]; then
    ENTRIES="$ENTRIES,"
  fi
  ENTRIES="$ENTRIES$ENTRY"
done

INVENTORY="{
  \"generated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
  \"project\": \"Uzima-Contracts\",
  \"contracts\": [$ENTRIES]
}"

if [[ -n "$OUTPUT_PATH" ]]; then
  mkdir -p "$(dirname "$OUTPUT_PATH")"
  echo "$INVENTORY" > "$OUTPUT_PATH"
  echo "Inventory written to $OUTPUT_PATH"
else
  echo "$INVENTORY"
fi
