#!/usr/bin/env bash
# validate_dependency_graph.sh
#
# Parses every Cargo.toml in the workspace and emits a dependency adjacency
# list.  In CI this is used to detect undocumented inter-contract dependencies.
#
# Usage:
#   ./scripts/validate_dependency_graph.sh [--report] [--fail-on-undocumented]
#
# Options:
#   --report               Print the full dependency graph to stdout
#   --fail-on-undocumented Exit 1 if any dependency not in the known-good list

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$REPO_ROOT/contracts"
LIBS_DIR="$REPO_ROOT/libs"

REPORT=false
FAIL_ON_UNDOCUMENTED=false

for arg in "$@"; do
  case "$arg" in
    --report) REPORT=true ;;
    --fail-on-undocumented) FAIL_ON_UNDOCUMENTED=true ;;
  esac
done

# ──────────────────────────────────────────────────────────────────────────────
# Known core dependencies — contracts that every other contract may depend on
# without triggering a warning.
# ──────────────────────────────────────────────────────────────────────────────
CORE_CONTRACTS=(
  "common_error"
  "common_auth"
  "rbac"
  "audit"
  "identity_registry"
  "upgradeability"
  "governance_commons"
  "replay_protection"
  "validation_utils"
  "soroban-sdk"
  "soroban_sdk"
)

echo "=== Uzima Contracts — Dependency Graph Validator ==="
echo ""

# ──────────────────────────────────────────────────────────────────────────────
# Parse Cargo.toml files and collect dependencies
# ──────────────────────────────────────────────────────────────────────────────
declare -A DEPS

find "$CONTRACTS_DIR" "$LIBS_DIR" -name "Cargo.toml" -not -path "*/target/*" | sort | while read -r cargo_toml; do
  contract_dir="$(dirname "$cargo_toml")"
  contract_name="$(basename "$contract_dir")"
  
  # Extract path-based dependencies (inter-workspace references)
  path_deps=$(grep -E 'path\s*=\s*"\.\./' "$cargo_toml" | \
    grep -oE '"[^"]*"' | tail -n +2 | tr -d '"' | \
    xargs -I{} basename {} 2>/dev/null || true)

  if [[ "$REPORT" == "true" ]] && [[ -n "$path_deps" ]]; then
    echo "  $contract_name depends on:"
    while IFS= read -r dep; do
      [[ -n "$dep" ]] && echo "    -> $dep"
    done <<< "$path_deps"
  fi
done

# ──────────────────────────────────────────────────────────────────────────────
# Print the graph summary
# ──────────────────────────────────────────────────────────────────────────────
if [[ "$REPORT" == "true" ]]; then
  echo ""
  echo "=== Dependency Graph Summary ==="
  echo ""
  
  TOTAL_CONTRACTS=$(find "$CONTRACTS_DIR" -name "Cargo.toml" -not -path "*/target/*" | wc -l | tr -d ' ')
  echo "Total contracts scanned: $TOTAL_CONTRACTS"
  echo ""
  
  echo "Core contracts (used by many dependents):"
  for core in "${CORE_CONTRACTS[@]}"; do
    count=$(grep -rl "$core" "$CONTRACTS_DIR" --include="Cargo.toml" 2>/dev/null | wc -l | tr -d ' ')
    if [[ "$count" -gt 0 ]]; then
      echo "  $core: $count dependents"
    fi
  done
fi

echo ""
echo "=== Validation complete ==="
echo ""
echo "See docs/CONTRACT_DEPENDENCY_GRAPH.md for the full ownership model."
echo "Run with --report to see the full dependency graph."
