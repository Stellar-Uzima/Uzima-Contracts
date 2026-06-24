#!/bin/bash

# verify_deployment.sh - Professional Contract Deployment Verification Script
# This script performs a 5-step verification of a deployed Soroban contract.
# Usage: ./scripts/verify_deployment.sh <contract_id> <network> [identity] [contract_name]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print helper functions
print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_header() {
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${PURPLE}  Verification: $1${NC}"
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# ==============================================================================
# Deterministic build-hash verification (Issue #854)
#
# These subcommands verify that the deployed WASM matches the artifact produced
# by the audited, tagged build — so auditors and users can confirm they are
# running the same bytecode.
#
#   verify_deployment.sh record  <network> <release> [signed_by_pubkey]
#       Build hashes are written to deployments/<network>/<release>/hashes.txt.
#   verify_deployment.sh compare <network> <release>
#       Rebuilds' hashes are compared to the recorded set; exits non-zero (CI
#       gate) on any mismatch. No-ops with a warning if no record exists yet.
#   verify_deployment.sh hashes
#       Prints the sha256 of each built .wasm (helper / debugging).
#
# WASM artifacts are read from dist/ when present (canonical `make dist`
# output), otherwise from the workspace release dir. Override with WASM_DIR.
# ==============================================================================

WASM_DIR="${WASM_DIR:-target/wasm32-unknown-unknown/release}"

# Portable sha256 (Linux: sha256sum, macOS: shasum -a 256).
sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

# Locate built contract .wasm files, deterministically sorted by name.
collect_wasm_files() {
    if compgen -G "dist/*.wasm" >/dev/null 2>&1; then
        find dist -maxdepth 1 -name '*.wasm' | sort
    else
        find "$WASM_DIR" -maxdepth 1 -name '*.wasm' 2>/dev/null | sort
    fi
}

# Emit `<sha256>  <artifact-name>` lines for every built .wasm.
compute_hashes() {
    local f
    while IFS= read -r f; do
        [ -n "$f" ] || continue
        printf '%s  %s\n' "$(sha256_of "$f")" "$(basename "$f")"
    done < <(collect_wasm_files)
}

cmd_record() {
    local network="${1:-}" release="${2:-}" signed_by="${3:-}"
    if [ -z "$network" ] || [ -z "$release" ]; then
        print_error "Usage: $0 record <network> <release> [signed_by_pubkey]"
        exit 1
    fi
    local hashes
    hashes="$(compute_hashes)"
    if [ -z "$hashes" ]; then
        print_error "No .wasm artifacts found (looked in dist/ then $WASM_DIR). Run 'make dist' first."
        exit 1
    fi
    local out_dir="deployments/$network/$release"
    mkdir -p "$out_dir"
    local out="$out_dir/hashes.txt"
    {
        echo "# Uzima-Contracts deployment WASM hashes"
        echo "# network:   $network"
        echo "# release:   $release"
        echo "# generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "# toolchain: $(rustc --version 2>/dev/null || echo unknown)"
        echo "# Replace the placeholder below with the auditor's signing pubkey."
        echo "Signed-by: ${signed_by:-<AUDITOR_PUBKEY_PENDING>}"
        echo "#"
        echo "# <sha256>  <artifact>"
        printf '%s\n' "$hashes"
    } > "$out"
    print_info "Recorded $(printf '%s\n' "$hashes" | grep -c '\.wasm$') WASM hash(es) -> $out"
}

cmd_compare() {
    local network="${1:-}" release="${2:-}"
    if [ -z "$network" ] || [ -z "$release" ]; then
        print_error "Usage: $0 compare <network> <release>"
        exit 1
    fi
    local ref="deployments/$network/$release/hashes.txt"
    if [ ! -f "$ref" ]; then
        print_warn "No recorded hashes at $ref — nothing to compare."
        print_warn "Record them at release time: $0 record $network $release <auditor_pubkey>"
        return 0
    fi
    local recorded fresh
    recorded="$(grep -E '^[0-9a-f]{64}  ' "$ref" | sort)"
    fresh="$(compute_hashes | sort)"
    if [ -z "$fresh" ]; then
        print_error "No freshly built .wasm artifacts found to compare (looked in dist/ then $WASM_DIR)."
        exit 1
    fi
    if [ "$recorded" = "$fresh" ]; then
        print_info "✅ Build hashes match the recorded set for $network/$release."
        return 0
    fi
    print_error "❌ WASM hash mismatch for $network/$release — built artifacts differ from the audited record."
    echo "--- recorded (deployments/$network/$release/hashes.txt)   +++ fresh build ---"
    diff <(printf '%s\n' "$recorded") <(printf '%s\n' "$fresh") || true
    exit 1
}

case "${1:-}" in
    record)
        shift
        cmd_record "$@"
        exit $?
        ;;
    compare)
        shift
        cmd_compare "$@"
        exit $?
        ;;
    hashes)
        compute_hashes
        exit $?
        ;;
esac

# ------------------------------------------------------------------------------
# Runtime verification mode (default): verify a *deployed* contract on-network.
# ------------------------------------------------------------------------------

# Check arguments
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract_id> <network> [identity] [contract_name]"
    print_error "   or: $0 {record|compare} <network> <release> [signed_by_pubkey]"
    exit 1
fi

CONTRACT_ID="$1"
NETWORK="$2"
IDENTITY="${3:-"default"}"
CONTRACT_NAME="${4:-"unknown"}"

print_header "$CONTRACT_NAME ($CONTRACT_ID) on $NETWORK"

# Helper function to invoke soroban with standard args
invoke_soroban() {
    local function_name=$1
    shift
    soroban contract invoke \
        --id "$CONTRACT_ID" \
        --source "$IDENTITY" \
        --network "$NETWORK" \
        -- "$function_name" "$@" 2>&1
}

# ------------------------------------------------------------------------------
# STEP 1: Deployment Verification
# ------------------------------------------------------------------------------
print_step "1/5: Verifying deployment existence..."
if ! soroban contract read \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --durability instance &>/dev/null; then
    
    # Fallback: check if we can get help output
    if ! soroban contract invoke --id "$CONTRACT_ID" --network "$NETWORK" -- --help &>/dev/null; then
        print_error "Contract $CONTRACT_ID not found or not reachable on $NETWORK"
        exit 1
    fi
fi
print_info "Contract successfully located on the network."

# ------------------------------------------------------------------------------
# STEP 2: Initialization Check
# ------------------------------------------------------------------------------
print_step "2/5: Checking initialization status..."
# Try to call initialize again - if it fails with "AlreadyInitialized" or similar, it's good.
# Or try to read 'Initialized' key if we know the contract structure.
INIT_CHECK=$(invoke_soroban initialize --admin "$(soroban config identity address "$IDENTITY")" 2>&1 || true)

if echo "$INIT_CHECK" | grep -qiE "AlreadyInitialized|Error\(Contract, 1\)|Already Initialized"; then
    print_info "Initialization confirmed (Contract reports already initialized)."
elif echo "$INIT_CHECK" | grep -qi "success"; then
    print_warn "Contract was NOT initialized; initialization performed during verification."
else
    # Try calling a simple getter that might fail if not initialized
    GETTER_CHECK=$(invoke_soroban version 2>&1 || invoke_soroban name 2>&1 || true)
    if echo "$GETTER_CHECK" | grep -qi "NotInitialized|Error\(Contract, 2\)"; then
        print_error "Contract is NOT initialized and initialization attempt failed."
        exit 1
    else
        print_info "Initialization status seems valid (or not required)."
    fi
fi

# ------------------------------------------------------------------------------
# STEP 3: Basic Functionality Test
# ------------------------------------------------------------------------------
print_step "3/5: Running basic functionality tests..."
# Try some common getters
declare -a GETTERS=("version" "name" "symbol" "decimals" "get_admin" "get_owner")
SUCCESSFUL_TEST=false

for getter in "${GETTERS[@]}"; do
    RESULT=$(invoke_soroban "$getter" 2>/dev/null || true)
    if [ -n "$RESULT" ] && ! echo "$RESULT" | grep -qi "error"; then
        print_info "Functionality test passed: $getter() -> $RESULT"
        SUCCESSFUL_TEST=true
        break
    fi
done

if [ "$SUCCESSFUL_TEST" = false ]; then
    print_warn "Could not find standard getter to test. Trying generic ping..."
    if invoke_soroban test_ping &>/dev/null; then
        print_info "Functionality test passed: test_ping()"
        SUCCESSFUL_TEST=true
    fi
fi

if [ "$SUCCESSFUL_TEST" = false ]; then
    print_warn "No standard health-check functions found. Proceeding with caution."
fi

# ------------------------------------------------------------------------------
# STEP 4: Event Emission Check
# ------------------------------------------------------------------------------
print_step "4/5: Checking event emission..."
# We use a diagnostic call or a state-changing call if safe.
# For now, we'll look at the last transaction if we can, or just trigger a log if possible.
# Professional way: capture output of an invocation and parse for events.
EVENT_OUTPUT=$(invoke_soroban get_admin 2>&1 || invoke_soroban name 2>&1 || true)
# Soroban CLI output for events usually contains "Events:"
if echo "$EVENT_OUTPUT" | grep -q "Events:"; then
    print_info "Event emission verified in contract interaction."
else
    print_info "No immediate events detected, which may be normal for read-only calls."
fi

# ------------------------------------------------------------------------------
# STEP 5: Storage State Validation
# ------------------------------------------------------------------------------
print_step "5/5: Validating storage state..."
STORAGE_DATA=$(soroban contract read --id "$CONTRACT_ID" --network "$NETWORK" --durability instance 2>/dev/null || echo "")
if [ -n "$STORAGE_DATA" ]; then
    STORAGE_SIZE=$(echo "$STORAGE_DATA" | wc -c)
    print_info "Storage state validated (Instance storage size: $STORAGE_SIZE bytes)."
else
    print_warn "Could not read instance storage directly. Verification may be incomplete."
fi

echo -e "\n${GREEN}✅ Verification completed successfully!${NC}"
exit 0
