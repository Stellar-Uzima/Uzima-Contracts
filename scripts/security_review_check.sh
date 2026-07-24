#!/usr/bin/env bash
# security_review_check.sh
#
# Automated security review checks for new contract submissions.
# Runs as part of CI on every PR that touches contracts/.
#
# Usage:
#   ./scripts/security_review_check.sh [--contract CONTRACT_NAME]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$REPO_ROOT/contracts"
THREAT_MODELS_DIR="$REPO_ROOT/docs/threat_models"
CONTRACT_FILTER=""

for arg in "$@"; do
  case "$arg" in
    --contract) shift; CONTRACT_FILTER="$1" ;;
  esac
done

PASS=0
WARN=0
FAIL=0

check() {
  local name="$1" result="$2" detail="$3"
  if [[ "$result" == "pass" ]]; then
    echo "  ✅ $name"
    PASS=$((PASS+1))
  elif [[ "$result" == "warn" ]]; then
    echo "  ⚠️  $name — $detail"
    WARN=$((WARN+1))
  else
    echo "  ❌ $name — $detail"
    FAIL=$((FAIL+1))
  fi
}

echo "=== Uzima Security Review Checker ==="
echo ""

scan_contract() {
  local contract="$1"
  local dir="$CONTRACTS_DIR/$contract/src"

  [[ -d "$dir" ]] || return 0

  echo "Contract: $contract"

  # Check 1: require_auth present in lib.rs
  if grep -rq 'require_auth' "$dir" 2>/dev/null; then
    check "require_auth present" "pass" ""
  else
    check "require_auth present" "warn" "No require_auth() calls found — verify all entrypoints are guarded"
  fi

  # Check 2: No bare unwrap in non-test code
  unwrap_count=$(grep -rn '\.unwrap()' "$dir" 2>/dev/null | grep -v '#\[cfg(test)\]' | grep -v '//.*unwrap' | wc -l || echo 0)
  if [[ "$unwrap_count" -eq 0 ]]; then
    check "No bare unwrap()" "pass" ""
  else
    check "No bare unwrap()" "fail" "$unwrap_count bare unwrap() calls in production code"
  fi

  # Check 3: Events emitted on mutations (heuristic)
  if grep -rq 'env\.events()\.publish' "$dir" 2>/dev/null; then
    check "Events emitted on mutations" "pass" ""
  else
    check "Events emitted on mutations" "warn" "No event emissions found — verify audit trail"
  fi

  # Check 4: String length validation
  if grep -rq 'len()' "$dir" 2>/dev/null || grep -rq 'validate_string' "$dir" 2>/dev/null; then
    check "String length validation present" "pass" ""
  else
    check "String length validation present" "warn" "No string length checks found — consider adding bounds"
  fi

  # Check 5: Threat model exists
  if [[ -f "$THREAT_MODELS_DIR/$contract.md" ]]; then
    check "Threat model exists" "pass" ""
  else
    check "Threat model exists" "warn" "No docs/threat_models/$contract.md — required for High/Critical contracts"
  fi

  echo ""
}

if [[ -n "$CONTRACT_FILTER" ]]; then
  scan_contract "$CONTRACT_FILTER"
else
  for contract_dir in "$CONTRACTS_DIR"/*/; do
    contract=$(basename "$contract_dir")
    [[ -f "$contract_dir/Cargo.toml" ]] && scan_contract "$contract"
  done
fi

echo "=== Results: $PASS passed, $WARN warnings, $FAIL failures ==="
echo ""

if [[ "$FAIL" -gt 0 ]]; then
  echo "Security review FAILED. Fix $FAIL critical issues before merging."
  exit 1
elif [[ "$WARN" -gt 0 ]]; then
  echo "Security review passed with $WARN warnings. Review and address where possible."
fi
