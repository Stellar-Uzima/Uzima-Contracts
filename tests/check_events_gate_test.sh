#!/usr/bin/env bash
# tests/check_events_gate_test.sh
#
# Integration and unit tests for scripts/check_events.sh
# Verifies enforcement of:
# 1. Mandatory issue references on allowlist entries (Issue #1513)
# 2. Rejection of unallowlisted state-changing pub fns missing events
# 3. Acceptance of read-only prefixed functions
# 4. Acceptance of validly allowlisted functions with issue references
# 5. Clean pass on repository contracts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECK_SCRIPT="$REPO_ROOT/scripts/check_events.sh"

TESTS_PASSED=0
TESTS_FAILED=0

assert_success() {
    local desc="$1"
    shift
    echo -n "[TEST] $desc ... "
    if "$@" > /dev/null 2>&1; then
        echo "PASS"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "FAIL (expected exit code 0)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

assert_failure() {
    local desc="$1"
    shift
    echo -n "[TEST] $desc ... "
    if "$@" > /dev/null 2>&1; then
        echo "FAIL (expected non-zero exit code)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    else
        echo "PASS"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi
}

echo "=========================================================="
echo " Running Event Emission Gate Test Suite (Issue #1513)"
echo "=========================================================="

# Test 1: Real repository check must pass
assert_success "Repository event emission audit passes cleanly" "$CHECK_SCRIPT"

# Setup temporary sandbox for synthetic tests
SANDBOX_DIR="$(mktemp -d /tmp/event_gate_test.XXXXXX)"
trap 'rm -rf "$SANDBOX_DIR"' EXIT

SANDBOX_CONTRACTS="$SANDBOX_DIR/contracts"
SANDBOX_ALLOWLIST="$SANDBOX_DIR/allowlist.txt"
mkdir -p "$SANDBOX_CONTRACTS/synthetic_contract/src"

# Test 2: Allowlist entry missing issue reference MUST FAIL
cat << 'ALLOW_EOF' > "$SANDBOX_ALLOWLIST"
# Valid header
synthetic_contract::legacy_fn
ALLOW_EOF

cat << 'CODE_EOF' > "$SANDBOX_CONTRACTS/synthetic_contract/src/lib.rs"
#![no_std]
pub struct SyntheticContract;
impl SyntheticContract {
    pub fn legacy_fn(env: Env) -> Result<(), Error> {
        Ok(())
    }
}
CODE_EOF

assert_failure \
    "Allowlist entry without issue reference is rejected" \
    env ROOT_DIR="$SANDBOX_DIR" CONTRACTS_DIR="$SANDBOX_CONTRACTS" ALLOWLIST_FILE="$SANDBOX_ALLOWLIST" "$CHECK_SCRIPT"

# Test 3: Allowlist entry with valid issue reference passes
cat << 'ALLOW_EOF' > "$SANDBOX_ALLOWLIST"
# Allowlist with valid issue ref
synthetic_contract::legacy_fn # #1513
ALLOW_EOF

assert_success \
    "Allowlist entry with '# #1513' issue reference is accepted" \
    env ROOT_DIR="$SANDBOX_DIR" CONTRACTS_DIR="$SANDBOX_CONTRACTS" ALLOWLIST_FILE="$SANDBOX_ALLOWLIST" "$CHECK_SCRIPT"

# Test 4: Unallowlisted function without events MUST FAIL
cat << 'ALLOW_EOF' > "$SANDBOX_ALLOWLIST"
# Empty allowlist
ALLOW_EOF

cat << 'CODE_EOF' > "$SANDBOX_CONTRACTS/synthetic_contract/src/lib.rs"
#![no_std]
pub struct SyntheticContract;
impl SyntheticContract {
    pub fn state_mutating_fn(env: Env, amount: u64) -> Result<(), Error> {
        env.storage().instance().set(&DataKey::Val, &amount);
        Ok(())
    }
}
CODE_EOF

assert_failure \
    "Unallowlisted function without event emission is rejected" \
    env ROOT_DIR="$SANDBOX_DIR" CONTRACTS_DIR="$SANDBOX_CONTRACTS" ALLOWLIST_FILE="$SANDBOX_ALLOWLIST" "$CHECK_SCRIPT"

# Test 5: Function with .events().publish(...) passes without allowlist
cat << 'CODE_EOF' > "$SANDBOX_CONTRACTS/synthetic_contract/src/lib.rs"
#![no_std]
pub struct SyntheticContract;
impl SyntheticContract {
    pub fn compliant_fn(env: Env, amount: u64) -> Result<(), Error> {
        env.storage().instance().set(&DataKey::Val, &amount);
        env.events().publish((symbol_short!("TEST"),), amount);
        Ok(())
    }
}
CODE_EOF

assert_success \
    "Compliant function with env.events().publish(...) is accepted" \
    env ROOT_DIR="$SANDBOX_DIR" CONTRACTS_DIR="$SANDBOX_CONTRACTS" ALLOWLIST_FILE="$SANDBOX_ALLOWLIST" "$CHECK_SCRIPT"

# Test 6: Read-only prefix functions are skipped and pass without events
cat << 'CODE_EOF' > "$SANDBOX_CONTRACTS/synthetic_contract/src/lib.rs"
#![no_std]
pub struct SyntheticContract;
impl SyntheticContract {
    pub fn get_value(env: Env) -> u64 {
        100
    }
    pub fn is_active(env: Env) -> bool {
        true
    }
    pub fn has_permission(env: Env, addr: Address) -> bool {
        true
    }
    pub fn query_status(env: Env) -> u32 {
        1
    }
    pub fn view_balance(env: Env) -> i128 {
        500
    }
}
CODE_EOF

assert_success \
    "Read-only prefixed functions (get_, is_, has_, query_, view_) are skipped" \
    env ROOT_DIR="$SANDBOX_DIR" CONTRACTS_DIR="$SANDBOX_CONTRACTS" ALLOWLIST_FILE="$SANDBOX_ALLOWLIST" "$CHECK_SCRIPT"

echo "=========================================================="
echo " Results: ${TESTS_PASSED} passed, ${TESTS_FAILED} failed"
echo "=========================================================="

if (( TESTS_FAILED > 0 )); then
    exit 1
fi
