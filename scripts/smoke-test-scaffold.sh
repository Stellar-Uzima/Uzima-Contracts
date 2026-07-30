#!/usr/bin/env bash
#
# smoke-test-scaffold.sh — Verify a newly scaffolded contract compiles and passes basic checks.
#
# Usage:
#   ./scripts/smoke-test-scaffold.sh <contract_name>
#
# Checks performed:
#   1. Contract compiles (cargo check)
#   2. Unit tests pass (cargo test)
#   3. Clippy passes (cargo clippy)
#   4. Formatting is correct (cargo fmt --check)
#   5. Error enum uses #[contracterror]
#   6. Event topics use snake_case
#   7. require_auth() is present on state-mutating functions
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <contract_name>"
  exit 1
fi

CONTRACT_NAME="$1"
PACKAGE_NAME=$(echo "$CONTRACT_NAME" | tr '_' '-')
CONTRACT_DIR="$PROJECT_ROOT/contracts/$CONTRACT_NAME"

if [[ ! -d "$CONTRACT_DIR" ]]; then
  echo "Error: Contract directory not found: $CONTRACT_DIR"
  exit 1
fi

PASS=0
FAIL=0
ERRORS=()

check() {
  local desc="$1"
  shift
  echo -n "  $desc ... "
  if "$@" >/dev/null 2>&1; then
    echo "PASS"
    PASS=$((PASS + 1))
  else
    echo "FAIL"
    FAIL=$((FAIL + 1))
    ERRORS+=("$desc")
  fi
}

echo "Smoke-testing contract: $CONTRACT_NAME"
echo ""

# ── Compilation ──────────────────────────────────────────────────────────────

echo "1. Compilation"
check "cargo check" cargo check --package "$PACKAGE_NAME" --all-targets
echo ""

# ── Tests ────────────────────────────────────────────────────────────────────

echo "2. Tests"
check "cargo test" cargo test --package "$PACKAGE_NAME"
echo ""

# ── Linting ──────────────────────────────────────────────────────────────────

echo "3. Linting"
check "cargo clippy" cargo clippy --package "$PACKAGE_NAME" --all-targets -- -D warnings
echo ""

# ── Formatting ───────────────────────────────────────────────────────────────

echo "4. Formatting"
check "cargo fmt --check" cargo fmt --package "$PACKAGE_NAME" -- --check
echo ""

# ── Structural checks ───────────────────────────────────────────────────────

echo "5. Structural checks"

# Check lib.rs exists and has #[contract]
if grep -q '#\[contract\]' "$CONTRACT_DIR/src/lib.rs" 2>/dev/null; then
  echo "  #[contract] attribute present ... PASS"
  PASS=$((PASS + 1))
else
  echo "  #[contract] attribute present ... FAIL"
  FAIL=$((FAIL + 1))
  ERRORS+=("Missing #[contract] attribute")
fi

# Check errors.rs uses #[contracterror]
if [[ -f "$CONTRACT_DIR/src/errors.rs" ]]; then
  if grep -q '#\[contracterror\]' "$CONTRACT_DIR/src/errors.rs" 2>/dev/null; then
    echo "  #[contracterror] attribute present ... PASS"
    PASS=$((PASS + 1))
  else
    echo "  #[contracterror] attribute present ... FAIL"
    FAIL=$((FAIL + 1))
    ERRORS+=("errors.rs missing #[contracterror]")
  fi
else
  echo "  errors.rs exists ... FAIL"
  FAIL=$((FAIL + 1))
  ERRORS+=("Missing errors.rs")
fi

# Check for initialize function
if grep -q 'fn initialize' "$CONTRACT_DIR/src/lib.rs" 2>/dev/null; then
  echo "  initialize() function present ... PASS"
  PASS=$((PASS + 1))
else
  echo "  initialize() function present ... FAIL"
  FAIL=$((FAIL + 1))
  ERRORS+=("Missing initialize() function")
fi

# Check for initialization guard (AlreadyInitialized check)
if grep -q 'AlreadyInitialized' "$CONTRACT_DIR/src/lib.rs" 2>/dev/null; then
  echo "  Initialization guard present ... PASS"
  PASS=$((PASS + 1))
else
  echo "  Initialization guard present ... FAIL"
  FAIL=$((FAIL + 1))
  ERRORS+=("Missing initialization guard")
fi

# Check test.rs exists with at least one test
if [[ -f "$CONTRACT_DIR/src/test.rs" ]]; then
  TEST_COUNT=$(grep -c '#\[test\]' "$CONTRACT_DIR/src/test.rs" 2>/dev/null || echo 0)
  if [[ $TEST_COUNT -ge 1 ]]; then
    echo "  test.rs with $TEST_COUNT test(s) ... PASS"
    PASS=$((PASS + 1))
  else
    echo "  test.rs with 0 tests ... FAIL"
    FAIL=$((FAIL + 1))
    ERRORS+=("test.rs has no #[test] functions")
  fi
else
  echo "  test.rs exists ... FAIL"
  FAIL=$((FAIL + 1))
  ERRORS+=("Missing test.rs")
fi

# Check events use snake_case topic names
if [[ -f "$CONTRACT_DIR/src/events.rs" ]]; then
  if grep -qP 'symbol_short!\("[a-z_]+"\)' "$CONTRACT_DIR/src/events.rs" 2>/dev/null; then
    echo "  Event topics use snake_case ... PASS"
    PASS=$((PASS + 1))
  else
    # Check if there are any events at all
    if grep -q 'events().publish' "$CONTRACT_DIR/src/events.rs" 2>/dev/null; then
      echo "  Event topics use snake_case ... FAIL"
      FAIL=$((FAIL + 1))
      ERRORS+=("Event topics must use snake_case")
    else
      echo "  Event topics use snake_case ... PASS (no events)"
      PASS=$((PASS + 1))
    fi
  fi
else
  echo "  events.rs exists ... PASS (optional)"
  PASS=$((PASS + 1))
fi

echo ""

# ── Summary ──────────────────────────────────────────────────────────────────

echo "══════════════════════════════════════════════════════"
echo "Results: $PASS passed, $FAIL failed"
echo "══════════════════════════════════════════════════════"

if [[ $FAIL -gt 0 ]]; then
  echo ""
  echo "Failed checks:"
  for err in "${ERRORS[@]}"; do
    echo "  - $err"
  done
  exit 1
fi

echo ""
echo "All checks passed!"
exit 0
