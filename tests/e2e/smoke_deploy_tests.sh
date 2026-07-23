#!/bin/bash
# End-to-end local-network smoke suite for contract deployments
# Tests core contract lifecycle on a local Soroban network

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NETWORK="local"
PASSPHRASE="Standalone Network ; September 2022"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TESTS_PASSED=0
TESTS_FAILED=0

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; TESTS_PASSED=$((TESTS_PASSED + 1)); }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; TESTS_FAILED=$((TESTS_FAILED + 1)); }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

setup_network() {
    log_info "Starting local Soroban network..."
    if ! soroban network ls | grep -q "^local"; then
        soroban network add \
            --rpc-url http://localhost:8000 \
            --network-passphrase "$PASSPHRASE" \
            local
    fi
    log_info "Using local network"
}

deploy_contract() {
    local contract_name="$1"
    local wasm_path="$PROJECT_ROOT/dist/${contract_name}.wasm"

    if [[ ! -f "$wasm_path" ]]; then
        log_warn "WASM not found for $contract_name, skipping"
        return 1
    fi

    local contract_id
    contract_id=$(soroban contract deploy \
        --wasm "$wasm_path" \
        --network "$NETWORK" \
        --network-passphrase "$PASSPHRASE" 2>/dev/null) || {
        log_fail "Failed to deploy $contract_name"
        return 1
    }

    echo "$contract_id"
}

test_deploy_identity_registry() {
    log_info "Testing identity_registry deployment..."
    local contract_id
    contract_id=$(deploy_contract "identity_registry")
    if [[ -n "$contract_id" ]]; then
        log_success "identity_registry deployed: $contract_id"
    fi
}

test_deploy_patient_consent() {
    log_info "Testing patient_consent_management deployment..."
    local contract_id
    contract_id=$(deploy_contract "patient_consent_management")
    if [[ -n "$contract_id" ]]; then
        log_success "patient_consent_management deployed: $contract_id"
    fi
}

test_deploy_medical_records() {
    log_info "Testing medical_records deployment..."
    local contract_id
    contract_id=$(deploy_contract "medical_records")
    if [[ -n "$contract_id" ]]; then
        log_success "medical_records deployed: $contract_id"
    fi
}

test_deploy_audit() {
    log_info "Testing audit deployment..."
    local contract_id
    contract_id=$(deploy_contract "audit")
    if [[ -n "$contract_id" ]]; then
        log_success "audit deployed: $contract_id"
    fi
}

test_deploy_escrow() {
    log_info "Testing escrow deployment..."
    local contract_id
    contract_id=$(deploy_contract "escrow")
    if [[ -n "$contract_id" ]]; then
        log_success "escrow deployed: $contract_id"
    fi
}

test_deploy_rbac() {
    log_info "Testing rbac deployment..."
    local contract_id
    contract_id=$(deploy_contract "rbac")
    if [[ -n "$contract_id" ]]; then
        log_success "rbac deployed: $contract_id"
    fi
}

test_deploy_governor() {
    log_info "Testing governor deployment..."
    local contract_id
    contract_id=$(deploy_contract "governor")
    if [[ -n "$contract_id" ]]; then
        log_success "governor deployed: $contract_id"
    fi
}

test_deploy_common_auth() {
    log_info "Testing common_auth deployment..."
    local contract_id
    contract_id=$(deploy_contract "common_auth")
    if [[ -n "$contract_id" ]]; then
        log_success "common_auth deployed: $contract_id"
    fi
}

test_wasm_availability() {
    log_info "Checking WASM artifacts availability..."
    local wasm_count=0
    for wasm in "$PROJECT_ROOT/dist"/*.wasm; do
        if [[ -f "$wasm" ]]; then
            wasm_count=$((wasm_count + 1))
        fi
    done

    if [[ $wasm_count -gt 0 ]]; then
        log_success "Found $wasm_count WASM artifacts"
    else
        log_fail "No WASM artifacts found in dist/"
    fi
}

run_smoke_suite() {
    echo "=========================================="
    echo "  Uzima E2E Smoke Suite"
    echo "=========================================="
    echo ""

    test_wasm_availability

    setup_network

    test_deploy_identity_registry
    test_deploy_patient_consent
    test_deploy_medical_records
    test_deploy_audit
    test_deploy_escrow
    test_deploy_rbac
    test_deploy_governor
    test_deploy_common_auth

    echo ""
    echo "=========================================="
    echo "  Results: $TESTS_PASSED passed, $TESTS_FAILED failed"
    echo "=========================================="

    if [[ $TESTS_FAILED -gt 0 ]]; then
        exit 1
    fi
}

show_help() {
    cat <<EOF
End-to-end local-network smoke suite for contract deployments

Usage: $0 [OPTIONS]

Options:
    --help    Show this help message

Tests:
1. WASM artifact availability check
2. Identity registry deployment
3. Patient consent management deployment
4. Medical records deployment
5. Audit contract deployment
6. Escrow contract deployment
7. RBAC deployment
8. Governor deployment
9. Common auth deployment
EOF
}

main() {
    if [[ "${1:-}" == "--help" ]]; then
        show_help
        exit 0
    fi

    run_smoke_suite
}

main "$@"
