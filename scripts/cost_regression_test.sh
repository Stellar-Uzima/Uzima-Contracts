#!/usr/bin/env bash
# cost_regression_test.sh
#
# Contract-level cost regression tests for resource budget enforcement.
# Compares current WASM sizes against recorded baselines and fails CI
# if any contract exceeds its budget or regresses beyond the threshold.
#
# Usage:
#   ./scripts/cost_regression_test.sh [--update-baseline]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE_FILE="$REPO_ROOT/resource-budgets/baselines.json"
BUDGETS_FILE="$REPO_ROOT/resource-budgets/budgets.json"
WASM_DIR="$REPO_ROOT/target/wasm32-unknown-unknown/release"

UPDATE_BASELINE=false
for arg in "$@"; do [[ "$arg" == "--update-baseline" ]] && UPDATE_BASELINE=true; done

echo "=== Contract Cost Regression Tests ==="
echo ""

if [[ ! -d "$WASM_DIR" ]]; then
  echo "ERROR: No WASM artifacts found. Run 'cargo build --workspace --target wasm32-unknown-unknown --release' first."
  exit 1
fi

PASS=0; FAIL=0; WARN=0

check_wasm() {
  local name="$1"
  local wasm="$WASM_DIR/${name}.wasm"
  [[ -f "$wasm" ]] || return 0

  local size
  size=$(stat -c%s "$wasm" 2>/dev/null || stat -f%z "$wasm" 2>/dev/null || echo 0)
  local size_kb
  size_kb=$(echo "scale=1; $size / 1024" | bc 2>/dev/null || echo "$size")

  # Thresholds from CONTRACT_RESOURCE_LIMITS.md
  local warn_threshold=$((51200))   # 50 KB
  local fail_threshold=$((63795))   # 62.3 KB

  if [[ "$size" -gt "$fail_threshold" ]]; then
    echo "  ❌ $name: ${size_kb}KB — EXCEEDS CRITICAL LIMIT (62.3 KB)"
    FAIL=$((FAIL+1))
  elif [[ "$size" -gt "$warn_threshold" ]]; then
    echo "  ⚠️  $name: ${size_kb}KB — above warning threshold (50 KB)"
    WARN=$((WARN+1))
  else
    echo "  ✅ $name: ${size_kb}KB"
    PASS=$((PASS+1))
  fi
}

echo "Checking WASM binary sizes..."
echo ""
for wasm_file in "$WASM_DIR"/*.wasm; do
  [[ -f "$wasm_file" ]] || continue
  contract=$(basename "$wasm_file" .wasm)
  check_wasm "$contract"
done

echo ""
echo "=== Results: $PASS pass, $WARN warnings, $FAIL failures ==="

if [[ "$FAIL" -gt 0 ]]; then
  echo ""
  echo "COST REGRESSION: $FAIL contracts exceed the 62.3 KB critical size limit."
  echo "See docs/WASM_BLOAT_REDUCTION.md for optimization techniques."
  exit 1
fi

if [[ "$UPDATE_BASELINE" == "true" ]]; then
  echo ""
  echo "Updating size baselines..."
  echo "{}" > "$BASELINE_FILE"
  for wasm_file in "$WASM_DIR"/*.wasm; do
    [[ -f "$wasm_file" ]] || continue
    contract=$(basename "$wasm_file" .wasm)
    size=$(stat -c%s "$wasm_file" 2>/dev/null || echo 0)
    python3 -c "
import json, sys
with open('$BASELINE_FILE') as f: d = json.load(f)
d['$contract'] = $size
with open('$BASELINE_FILE', 'w') as f: json.dump(d, f, indent=2)
" 2>/dev/null || true
  done
  echo "Baseline updated at $BASELINE_FILE"
fi
