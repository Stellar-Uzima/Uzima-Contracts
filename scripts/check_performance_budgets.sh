#!/usr/bin/env bash
#
# Validate contract WASM sizes against resource-budgets/budgets.json.
#
# Usage:
#   bash scripts/check_performance_budgets.sh [wasm_dir]
#
# If wasm_dir is not provided, defaults to target/wasm32-unknown-unknown/release.
#
# Exit codes:
#   0 - All contracts within budget
#   1 - One or more contracts exceed budget
#   2 - budgets.json missing or malformed
#
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUDGET_FILE="${ROOT_DIR}/resource-budgets/budgets.json"
WASM_DIR="${1:-${ROOT_DIR}/target/wasm32-unknown-unknown/release}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ ! -f "$BUDGET_FILE" ]; then
    echo -e "${RED}ERROR: Budget file not found: $BUDGET_FILE${NC}" >&2
    exit 2
fi

if ! command -v jq &>/dev/null; then
    echo -e "${YELLOW}WARN: jq not found, falling back to python3 for JSON parsing${NC}" >&2
    USE_PYTHON=1
else
    USE_PYTHON=0
fi

VIOLATIONS=0
CHECKED=0
SKIPPED=0

echo "=== Performance Budget Check ==="
echo "Budget file: $BUDGET_FILE"
echo "WASM directory: $WASM_DIR"
echo ""

read_budget_value() {
    local contract="$1"
    local field="$2"

    if [ "$USE_PYTHON" -eq 1 ]; then
        python3 -c "
import json, sys
with open('$BUDGET_FILE') as f:
    data = json.load(f)
entry = data.get('contracts', {}).get('$contract', data.get('defaults', {}))
print(entry.get('$field', 0))
" 2>/dev/null
    else
        jq -r "
            (.contracts[\"$contract\"] // .defaults) | .[\"$field\"] // 0
        " "$BUDGET_FILE"
    fi
}

if [ "$USE_PYTHON" -eq 1 ]; then
    CONTRACTS=$(python3 -c "
import json
with open('$BUDGET_FILE') as f:
    data = json.load(f)
for name in data.get('contracts', {}).keys():
    print(name)
" 2>/dev/null)
else
    CONTRACTS=$(jq -r '.contracts | keys[]' "$BUDGET_FILE")
fi

for contract in $CONTRACTS; do
    wasm_file="${WASM_DIR}/${contract}.wasm"

    max_bytes=$(read_budget_value "$contract" "max_wasm_bytes")
    tolerance=$(read_budget_value "$contract" "regression_tolerance_pct")
    notes=$(read_budget_value "$contract" "notes")

    if [ ! -f "$wasm_file" ]; then
        echo -e "  ${YELLOW}SKIP${NC}  ${contract}.wasm — not found in $WASM_DIR"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    actual_bytes=$(wc -c < "$wasm_file" | tr -d ' ')
    budget_with_tolerance=$(echo "$max_bytes * (100 + $tolerance) / 100" | bc 2>/dev/null || echo "$max_bytes")

    CHECKED=$((CHECKED + 1))

    if [ "$actual_bytes" -le "$max_bytes" ]; then
        echo -e "  ${GREEN}PASS${NC}   ${contract}  ${actual_bytes} / ${max_bytes} bytes"
    elif [ "$actual_bytes" -le "$budget_with_tolerance" ]; then
        echo -e "  ${YELLOW}WARN${NC}   ${contract}  ${actual_bytes} / ${max_bytes} bytes (within ${tolerance}% tolerance)"
    else
        echo -e "  ${RED}FAIL${NC}   ${contract}  ${actual_bytes} / ${max_bytes} bytes (exceeds budget by $((actual_bytes - max_bytes)) bytes)"
        VIOLATIONS=$((VIOLATIONS + 1))
    fi
done

echo ""
echo "=== Summary ==="
echo "  Checked: $CHECKED"
echo "  Skipped: $SKIPPED"
echo "  Violations: $VIOLATIONS"

if [ "$VIOLATIONS" -gt 0 ]; then
    echo ""
    echo -e "${RED}FAILED: $VIOLATIONS contract(s) exceed performance budget.${NC}"
    exit 1
else
    echo ""
    echo -e "${GREEN}PASSED: All checked contracts are within budget.${NC}"
    exit 0
fi
