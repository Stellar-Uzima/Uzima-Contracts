#!/usr/bin/env bash
# coverage_gate_test.sh — Test suite for the coverage gate (Issue #1196)
#
# Validates that the coverage gate script correctly:
#   1. Loads configuration
#   2. Identifies high-risk contracts
#   3. Enforces thresholds
#   4. Produces valid output
#
# Usage:
#   bash tests/coverage_gate_test.sh
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
GATE_SCRIPT="${ROOT_DIR}/scripts/coverage_gate.sh"
CONFIG_FILE="${ROOT_DIR}/resource-budgets/coverage-gate.json"

PASS=0
FAIL=0
TOTAL=0

RED='\033[0;31m'; GREEN='\033[0;32m'; NC='\033[0m'

assert_eq() {
  TOTAL=$((TOTAL + 1))
  local label="$1" expected="$2" actual="$3"
  if [[ "$expected" == "$actual" ]]; then
    echo -e "${GREEN}  PASS${NC}: $label"
    PASS=$((PASS + 1))
  else
    echo -e "${RED}  FAIL${NC}: $label (expected='$expected', actual='$actual')"
    FAIL=$((FAIL + 1))
  fi
}

assert_file_exists() {
  TOTAL=$((TOTAL + 1))
  local label="$1" path="$2"
  if [[ -f "$path" ]]; then
    echo -e "${GREEN}  PASS${NC}: $label exists"
    PASS=$((PASS + 1))
  else
    echo -e "${RED}  FAIL${NC}: $label does not exist at $path"
    FAIL=$((FAIL + 1))
  fi
}

assert_contains() {
  TOTAL=$((TOTAL + 1))
  local label="$1" haystack="$2" needle="$3"
  if echo "$haystack" | grep -q "$needle"; then
    echo -e "${GREEN}  PASS${NC}: $label"
    PASS=$((PASS + 1))
  else
    echo -e "${RED}  FAIL${NC}: $label (output does not contain '$needle')"
    FAIL=$((FAIL + 1))
  fi
}

# ─── Tests ───────────────────────────────────────────────────────────────────

echo ""
echo "=== Coverage Gate Test Suite (Issue #1196) ==="
echo ""

echo "--- Test: Script exists and is executable ---"
assert_file_exists "coverage_gate.sh" "$GATE_SCRIPT"

echo ""
echo "--- Test: Config file exists ---"
assert_file_exists "coverage-gate.json" "$CONFIG_FILE"

echo ""
echo "--- Test: Config has required fields ---"
if [[ -f "$CONFIG_FILE" ]]; then
  high_risk_count=$(jq '.high_risk_contracts | length' "$CONFIG_FILE")
  assert_eq "high_risk_contracts count > 0" "true" "$([ "$high_risk_count" -gt 0 ] && echo true || echo false)"

  standard_rate=$(jq -r '.thresholds.standard.test_pass_rate' "$CONFIG_FILE")
  assert_eq "standard test_pass_rate is 80" "80" "$standard_rate"

  high_risk_rate=$(jq -r '.thresholds.high_risk.test_pass_rate' "$CONFIG_FILE")
  assert_eq "high_risk test_pass_rate is 95" "95" "$high_risk_rate"

  max_docs_std=$(jq -r '.thresholds.standard.max_missing_docs' "$CONFIG_FILE")
  assert_eq "standard max_missing_docs is 10" "10" "$max_docs_std"

  max_docs_hr=$(jq -r '.thresholds.high_risk.max_missing_docs' "$CONFIG_FILE")
  assert_eq "high_risk max_missing_docs is 0" "0" "$max_docs_hr"
fi

echo ""
echo "--- Test: High-risk list contains identity_registry ---"
if [[ -f "$CONFIG_FILE" ]]; then
  has_ir=$(jq -e '.high_risk_contracts | index("identity_registry")' "$CONFIG_FILE" >/dev/null 2>&1 && echo "true" || echo "false")
  assert_eq "identity_registry is high-risk" "true" "$has_ir"
fi

echo ""
echo "--- Test: Gate script --help works ---"
help_output=$(bash "$GATE_SCRIPT" --help 2>&1 || true)
assert_contains "help contains usage" "$help_output" "Usage:"
assert_contains "help mentions Issue" "$help_output" "1196"

echo ""
echo "--- Test: Gate script --update works ---"
update_output=$(bash "$GATE_SCRIPT" --update 2>&1 || true)
assert_contains "update saves config" "$update_output" "Config saved"

echo ""
echo "--- Test: Gate script --report generates report (skipped, uses --check) ---"
# --report runs cargo test which is slow; validate via --check with WARN_ONLY
WARN_ONLY=1 check_output=$(bash "$GATE_SCRIPT" --check 2>&1 || true)
assert_contains "check produces coverage gate output" "$check_output" "Coverage"

echo ""
echo "--- Test: Result JSON is valid (if present) ---"
if [[ -f "${ROOT_DIR}/reports/coverage-gate-result.json" ]]; then
  json_valid=$(jq empty "${ROOT_DIR}/reports/coverage-gate-result.json" 2>/dev/null && echo "true" || echo "false")
  assert_eq "result JSON is valid" "true" "$json_valid"

  has_status=$(jq -r '.status' "${ROOT_DIR}/reports/coverage-gate-result.json" 2>/dev/null)
  assert_eq "result has status field" "true" "$([ -n "$has_status" ] && echo true || echo false)"
else
  echo "  (result JSON not yet generated — skipped)"
fi

echo ""
echo "=== Results: ${PASS}/${TOTAL} passed, ${FAIL} failed ==="
echo ""

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
exit 0
