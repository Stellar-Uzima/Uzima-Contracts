#!/usr/bin/env bash
# check_template_compat.sh
#
# Validates that a contract directory follows the project template conventions.
# Can be used to check a single contract or all workspace-member contracts.
#
# Usage:
#   ./scripts/check_template_compat.sh <contract_name>    # check one contract
#   ./scripts/check_template_compat.sh --all              # check all contracts
#   ./scripts/check_template_compat.sh --all --strict     # fail on any deviation
#
# Exit codes:
#   0 — contract is compatible with the template
#   1 — one or more compatibility violations found

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACTS_DIR="$ROOT_DIR/contracts"
TEMPLATE_DIR="$CONTRACTS_DIR/contract_template"
STRICT=false
CHECK_ALL=false
TARGET_CONTRACT=""

# ---------------------------------------------------------------------------
# Template structure: required files and patterns
# ---------------------------------------------------------------------------
REQUIRED_FILES=(
    "Cargo.toml"
    "src/lib.rs"
)

RECOMMENDED_FILES=(
    "src/errors.rs"
    "src/events.rs"
    "src/types.rs"
)

# Required patterns in Cargo.toml (substring match)
CARGO_REQUIRED_PATTERNS=(
    'crate-type = ["cdylib"]'
    "soroban-sdk"
)

# Required patterns in lib.rs (substring match)
LIB_RS_REQUIRED_PATTERNS=(
    '#![no_std]'
    '#![forbid(alloc)]'
    "#[contract]"
    "#[contractimpl]"
)

# Required patterns in errors.rs
ERRORS_RS_REQUIRED_PATTERNS=(
    "#[contracterror]"
    "#[repr(u32)]"
)

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
for arg in "$@"; do
    case "$arg" in
        --all)    CHECK_ALL=true ;;
        --strict) STRICT=true ;;
        -*)       echo "Unknown option: $arg"; exit 1 ;;
        *)        TARGET_CONTRACT="$arg" ;;
    esac
done

if ! $CHECK_ALL && [[ -z "$TARGET_CONTRACT" ]]; then
    echo "Usage: $0 <contract_name> | --all [--strict]"
    echo ""
    echo "Examples:"
    echo "  $0 escrow           # check a single contract"
    echo "  $0 --all            # check all workspace contracts"
    echo "  $0 --all --strict   # check all, fail on any deviation"
    exit 1
fi

# ---------------------------------------------------------------------------
# Excluded contracts (non-contract directories, fuzz harnesses, etc.)
# ---------------------------------------------------------------------------
declare -A EXCLUDED=()
EXCLUDED["contract_behavior_fuzzing"]=1
EXCLUDED["governance_integration_tests"]=1
EXCLUDED["storage-snapshot"]=1
EXCLUDED["test-helpers"]=1
EXCLUDED["contract_template"]=1

# ---------------------------------------------------------------------------
# Check a single contract
# ---------------------------------------------------------------------------
check_contract() {
    local contract="$1"
    local contract_dir="$CONTRACTS_DIR/$contract"
    local violations=0
    local warnings=0

    echo "--- Checking: $contract ---"

    # 1. Check required files
    for file in "${REQUIRED_FILES[@]}"; do
        if [[ ! -f "$contract_dir/$file" ]]; then
            echo "  FAIL [missing file]: $file"
            violations=$((violations + 1))
        fi
    done

    # 2. Check recommended files (warnings in strict mode)
    for file in "${RECOMMENDED_FILES[@]}"; do
        if [[ ! -f "$contract_dir/$file" ]]; then
            if $STRICT; then
                echo "  WARN [missing recommended file]: $file"
                violations=$((violations + 1))
            else
                echo "  INFO [missing recommended file]: $file"
                warnings=$((warnings + 1))
            fi
        fi
    done

    # 3. Check Cargo.toml patterns
    if [[ -f "$contract_dir/Cargo.toml" ]]; then
        local cargo_content
        cargo_content="$(cat "$contract_dir/Cargo.toml")"
        for pattern in "${CARGO_REQUIRED_PATTERNS[@]}"; do
            if ! echo "$cargo_content" | grep -qF "$pattern"; then
                echo "  FAIL [Cargo.toml]: missing pattern: $pattern"
                violations=$((violations + 1))
            fi
        done
    fi

    # 4. Check lib.rs patterns
    if [[ -f "$contract_dir/src/lib.rs" ]]; then
        local lib_content
        lib_content="$(cat "$contract_dir/src/lib.rs")"
        for pattern in "${LIB_RS_REQUIRED_PATTERNS[@]}"; do
            if ! echo "$lib_content" | grep -qF "$pattern"; then
                echo "  FAIL [lib.rs]: missing pattern: $pattern"
                violations=$((violations + 1))
            fi
        done

        # Check snake_case naming: contract name in lib.rs should be PascalCase
        local pascal_name
        pascal_name="$(echo "$contract" | sed -r 's/(^|_)([a-z])/\U\2/g')"
        if ! echo "$lib_content" | grep -q "pub struct $pascal_name"; then
            echo "  WARN [lib.rs]: expected 'pub struct $pascal_name' (PascalCase from snake_case)"
            warnings=$((warnings + 1))
        fi
    fi

    # 5. Check errors.rs patterns
    if [[ -f "$contract_dir/src/errors.rs" ]]; then
        local errors_content
        errors_content="$(cat "$contract_dir/src/errors.rs")"
        for pattern in "${ERRORS_RS_REQUIRED_PATTERNS[@]}"; do
            if ! echo "$errors_content" | grep -qF "$pattern"; then
                echo "  FAIL [errors.rs]: missing pattern: $pattern"
                violations=$((violations + 1))
            fi
        done
    fi

    # 6. Check naming conventions in Cargo.toml
    if [[ -f "$contract_dir/Cargo.toml" ]]; then
        local expected_kebab
        expected_kebab="$(echo "$contract" | tr '_' '-')"
        local actual_name
        actual_name="$(grep '^name' "$contract_dir/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/')"
        # Handle workspace name reference
        if [[ "$actual_name" == *"{"* ]]; then
            actual_name="workspace"
        fi
        if [[ "$actual_name" != "workspace" && "$actual_name" != "$expected_kebab" ]]; then
            echo "  WARN [Cargo.toml]: package name '$actual_name' doesn't match expected '$expected_kebab'"
            warnings=$((warnings + 1))
        fi
    fi

    # Summary for this contract
    if (( violations > 0 )); then
        echo "  -> FAIL: $violations violation(s), $warnings warning(s)"
    elif (( warnings > 0 )); then
        echo "  -> WARN: $warnings warning(s)"
    else
        echo "  -> OK"
    fi

    return $violations
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
total_violations=0
total_warnings=0
checked=0

if $CHECK_ALL; then
    while IFS= read -r -d '' cargo_file; do
        contract="$(basename "$(dirname "$cargo_file")")"
        if [[ -v "EXCLUDED[$contract]" ]]; then
            continue
        fi
        checked=$((checked + 1))
        if ! check_contract "$contract"; then
            total_violations=$((total_violations + $?))
        fi
    done < <(find "$CONTRACTS_DIR" -maxdepth 2 -name 'Cargo.toml' -print0 | sort -z)
else
    if [[ ! -d "$CONTRACTS_DIR/$TARGET_CONTRACT" ]]; then
        echo "Error: contract directory not found: $CONTRACTS_DIR/$TARGET_CONTRACT"
        exit 1
    fi
    checked=1
    if ! check_contract "$TARGET_CONTRACT"; then
        total_violations=$?
    fi
fi

echo
echo "Template compatibility check results:"
echo "  Contracts checked: $checked"
echo "  Total violations:  $total_violations"
echo

if (( total_violations > 0 )); then
    echo "FAIL: $total_violations compatibility violation(s) found."
    echo "Fix the issues above or run 'cargo test --package <contract>' to verify."
    exit 1
fi

echo "OK: all checked contracts are compatible with the template."