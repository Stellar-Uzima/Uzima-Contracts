#!/bin/bash
# preflight_check.sh - Contract preflight validation for deployment readiness

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_DIR="$PROJECT_ROOT/config"
DEPLOYMENTS_DIR="$PROJECT_ROOT/deployments"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNED=0

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_pass() { echo -e "${GREEN}[PASS]${NC} $1"; CHECKS_PASSED=$((CHECKS_PASSED + 1)); }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; CHECKS_WARNED=$((CHECKS_WARNED + 1)); }
log_fail() { echo -e "${RED}[FAIL]${NC} $1"; CHECKS_FAILED=$((CHECKS_FAILED + 1)); }

NETWORK=""
CONTRACT=""
IDENTITY="default"
ENVIRONMENT=""

show_help() {
    cat <<EOF
Contract Preflight Validation for deployment readiness.

Usage: $0 [OPTIONS]

Options:
    --network <network>      Target network (testnet, mainnet, futurenet, local)
    --contract <name>        Specific contract to validate
    --identity <name>        Identity to use (default: default)
    --environment <env>      Environment configuration
    --help                   Show this help message
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --network) NETWORK="$2"; shift 2 ;;
            --contract) CONTRACT="$2"; shift 2 ;;
            --identity) IDENTITY="$2"; shift 2 ;;
            --environment) ENVIRONMENT="$2"; shift 2 ;;
            --help|-h) show_help; exit 0 ;;
            *) log_fail "Unknown option: $1"; show_help; exit 1 ;;
        esac
    done
    if [[ -z "$NETWORK" ]]; then
        log_fail "Network is required. Use --network <testnet|mainnet|futurenet|local>"
        exit 1
    fi
}

validate_network_config() {
    log_info "Validating network configuration..."
    local networks_file="$CONFIG_DIR/networks.toml"
    if [[ ! -f "$networks_file" ]]; then
        log_fail "Network configuration file not found: $networks_file"
        return 1
    fi
    log_pass "Network configuration file exists"
    if command -v python3 &>/dev/null; then
        if python3 -c "
import tomllib
with open('$networks_file', 'rb') as f:
    config = tomllib.load(f)
if 'networks' not in config: exit(1)
if '$NETWORK' not in config['networks']: exit(2)
net = config['networks']['$NETWORK']
for field in ['rpc-url', 'network-passphrase']:
    if field not in net: exit(3)
" 2>/dev/null; then
            log_pass "Network '$NETWORK' configuration is valid"
        else
            log_fail "Network '$NETWORK' configuration is incomplete or invalid"
            return 1
        fi
    fi
}

validate_network_connectivity() {
    log_info "Checking network connectivity..."
    local rpc_url
    rpc_url=$(python3 -c "
import tomllib
with open('$CONFIG_DIR/networks.toml', 'rb') as f:
    config = tomllib.load(f)
print(config['networks']['$NETWORK']['rpc-url'])
" 2>/dev/null || echo "")
    if [[ -z "$rpc_url" ]]; then
        log_warn "Could not extract RPC URL for $NETWORK"
        return 0
    fi
    if curl -s --connect-timeout 5 --max-time 10 "$rpc_url" >/dev/null 2>&1; then
        log_pass "Network '$NETWORK' is reachable"
    else
        log_warn "Network '$NETWORK' is not reachable (may be expected for local)"
    fi
}

validate_deployment_manifest() {
    log_info "Validating deployment manifest..."
    local manifest="$DEPLOYMENTS_DIR/deployment-manifest.json"
    if [[ ! -f "$manifest" ]]; then
        log_warn "Deployment manifest not found"
        return 0
    fi
    if command -v python3 &>/dev/null; then
        if python3 -m json.tool "$manifest" >/dev/null 2>&1; then
            log_pass "Deployment manifest is valid JSON"
        else
            log_fail "Deployment manifest is not valid JSON"
            return 1
        fi
    fi
}

validate_contract_exists() {
    [[ -z "$CONTRACT" ]] && return 0
    log_info "Validating contract: $CONTRACT"
    local contract_dir="$PROJECT_ROOT/contracts/$CONTRACT"
    if [[ ! -d "$contract_dir" ]]; then
        log_fail "Contract directory not found: $contract_dir"
        return 1
    fi
    log_pass "Contract directory exists: $CONTRACT"
    if [[ ! -f "$contract_dir/Cargo.toml" ]]; then
        log_fail "Cargo.toml not found for contract: $CONTRACT"
        return 1
    fi
    log_pass "Cargo.toml found for contract: $CONTRACT"
}

validate_contract_dependencies() {
    [[ -z "$CONTRACT" ]] && return 0
    log_info "Checking contract dependencies..."
    local cargo_toml="$PROJECT_ROOT/contracts/$CONTRACT/Cargo.toml"
    [[ ! -f "$cargo_toml" ]] && return 0
    local deps
    deps=$(grep -oE 'path\s*=\s*"\.\./\.\./([^"]+)"' "$cargo_toml" 2>/dev/null | grep -oE '\.\./\.\./([^"]+)' | sed 's|../../||' || true)
    if [[ -n "$deps" ]]; then
        local all_found=true
        while IFS= read -r dep; do
            if [[ -n "$dep" && ! -d "$PROJECT_ROOT/contracts/$dep" ]]; then
                log_fail "Dependency not found: $dep"
                all_found=false
            fi
        done <<< "$deps"
        [[ "$all_found" == "true" ]] && log_pass "All contract dependencies found"
    else
        log_pass "No external path dependencies"
    fi
}

validate_wasm_build() {
    [[ -z "$CONTRACT" ]] && return 0
    log_info "Checking WASM build artifact..."
    local wasm_file="$PROJECT_ROOT/target/wasm32-unknown-unknown/release/${CONTRACT}.wasm"
    if [[ -f "$wasm_file" ]]; then
        local size
        size=$(stat -f%z "$wasm_file" 2>/dev/null || stat --printf="%s" "$wasm_file" 2>/dev/null || echo "0")
        if [[ "$size" -gt 0 ]]; then
            log_pass "WASM artifact found ($size bytes): $CONTRACT"
        else
            log_fail "WASM artifact is empty: $CONTRACT"
        fi
    else
        log_warn "WASM artifact not found (run cargo build first): $CONTRACT"
    fi
}

validate_environment_config() {
    [[ -z "$ENVIRONMENT" ]] && return 0
    log_info "Validating environment configuration: $ENVIRONMENT"
    local env_config="$CONFIG_DIR/$ENVIRONMENT.json"
    if [[ ! -f "$env_config" ]]; then
        log_fail "Environment configuration not found: $env_config"
        return 1
    fi
    if command -v python3 &>/dev/null; then
        if python3 -m json.tool "$env_config" >/dev/null 2>&1; then
            log_pass "Environment configuration is valid JSON"
        else
            log_fail "Environment configuration is not valid JSON"
            return 1
        fi
    fi
}

validate_identity_config() {
    log_info "Validating identity configuration: $IDENTITY"
    if command -v soroban &>/dev/null; then
        if soroban config identity show "$IDENTITY" >/dev/null 2>&1; then
            local address
            address=$(soroban config identity address "$IDENTITY" 2>/dev/null || echo "")
            log_pass "Identity '$IDENTITY' configured ($address)"
        else
            log_warn "Identity '$IDENTITY' not found (will be generated during deployment)"
        fi
    else
        log_warn "Soroban CLI not found, skipping identity validation"
    fi
}

validate_resource_budgets() {
    [[ -z "$CONTRACT" ]] && return 0
    log_info "Checking resource budget..."
    local budgets_file="$PROJECT_ROOT/resource-budgets/budgets.json"
    if [[ ! -f "$budgets_file" ]]; then
        log_warn "Resource budgets file not found"
        return 0
    fi
    if command -v python3 &>/dev/null; then
        if python3 -c "
import json
with open('$budgets_file') as f:
    budgets = json.load(f)
if '$CONTRACT' in budgets.get('contracts', {}):
    budget = budgets['contracts']['$CONTRACT']
    print(f'Budget: {budget.get(\"max_wasm_bytes\", 65536)} bytes')
else:
    exit(1)
" 2>/dev/null; then
            log_pass "Resource budget defined for $CONTRACT"
        else
            log_warn "No resource budget defined for $CONTRACT"
        fi
    fi
}

validate_signing_config() {
    log_info "Validating signing configuration..."
    local signing_config="$DEPLOYMENTS_DIR/signing-config.json"
    if [[ -f "$signing_config" ]]; then
        if command -v python3 &>/dev/null; then
            if python3 -m json.tool "$signing_config" >/dev/null 2>&1; then
                log_pass "Signing configuration is valid"
            else
                log_warn "Signing configuration is not valid JSON"
            fi
        fi
    else
        log_warn "Signing configuration not found"
    fi
}

generate_report() {
    local report_file="$PROJECT_ROOT/reports/preflight_report.json"
    mkdir -p "$(dirname "$report_file")"
    cat > "$report_file" <<EOF
{
    "preflight_version": "1.0.0",
    "executed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "network": "$NETWORK",
    "contract": "${CONTRACT:-all}",
    "environment": "${ENVIRONMENT:-default}",
    "identity": "$IDENTITY",
    "results": {
        "passed": $CHECKS_PASSED,
        "failed": $CHECKS_FAILED,
        "warned": $CHECKS_WARNED,
        "total": $((CHECKS_PASSED + CHECKS_FAILED + CHECKS_WARNED))
    }
}
EOF
    echo ""
    echo "=========================================="
    echo "  Preflight Check Report"
    echo "=========================================="
    echo "Network: $NETWORK"
    [[ -n "$CONTRACT" ]] && echo "Contract: $CONTRACT"
    [[ -n "$ENVIRONMENT" ]] && echo "Environment: $ENVIRONMENT"
    echo "Identity: $IDENTITY"
    echo ""
    echo -e "Passed: ${GREEN}$CHECKS_PASSED${NC}"
    echo -e "Failed: ${RED}$CHECKS_FAILED${NC}"
    echo -e "Warnings: ${YELLOW}$CHECKS_WARNED${NC}"
    echo ""
    echo "Report: $report_file"
    echo "=========================================="
}

main() {
    parse_args "$@"
    echo "=========================================="
    echo "  Contract Preflight Validation"
    echo "  Network: $NETWORK"
    [[ -n "$CONTRACT" ]] && echo "  Contract: $CONTRACT"
    echo "=========================================="
    echo ""
    validate_network_config
    validate_network_connectivity
    validate_deployment_manifest
    echo ""
    validate_contract_exists
    validate_contract_dependencies
    validate_wasm_build
    echo ""
    validate_environment_config
    validate_identity_config
    echo ""
    validate_resource_budgets
    validate_signing_config
    generate_report
    [[ $CHECKS_FAILED -gt 0 ]] && exit 1
}

trap 'log_fail "Script failed at line $LINENO"' ERR
main "$@"
