#!/bin/bash

# validate_network_config.sh - Comprehensive Network Configuration Validation
# Validates network configurations, connectivity, and deployment readiness

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Global variables
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
NETWORKS_CONFIG="$PROJECT_ROOT/config/networks.toml"

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Print functions
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_test_result() {
    local test_name="$1"
    local result="$2"
    local message="${3:-}"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    if [[ "$result" == "PASS" ]]; then
        echo -e "  ${GREEN}✓ PASS${NC} $test_name"
        [[ -n "$message" ]] && echo -e "    ${CYAN}$message${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "  ${RED}✗ FAIL${NC} $test_name"
        [[ -n "$message" ]] && echo -e "    ${RED}$message${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Test if file exists and is readable
test_file_exists() {
    local file="$1"
    local description="$2"
    
    if [[ -f "$file" && -r "$file" ]]; then
        print_test_result "$description" "PASS" "File found and readable"
        return 0
    else
        print_test_result "$description" "FAIL" "File not found or not readable: $file"
        return 1
    fi
}

# Test TOML syntax (basic validation)
test_toml_syntax() {
    local file="$1"
    
    # Basic TOML syntax check using Python if available
    if command -v python3 &> /dev/null; then
        if python3 -c "
import tomllib
try:
    with open('$file', 'rb') as f:
        tomllib.load(f)
    print('TOML syntax is valid')
except Exception as e:
    print(f'TOML syntax error: {e}')
    exit(1)
" 2>/dev/null; then
            print_test_result "TOML Syntax Validation" "PASS" "Valid TOML syntax"
            return 0
        else
            print_test_result "TOML Syntax Validation" "FAIL" "Invalid TOML syntax"
            return 1
        fi
    else
        print_test_result "TOML Syntax Validation" "SKIP" "Python3 not available for validation"
        return 0
    fi
}

# Test network configuration completeness
test_network_config_completeness() {
    local network="$1"
    local required_fields=("rpc-url" "network-passphrase" "name" "description" "environment")
    
    local all_fields_present=true
    
    for field in "${required_fields[@]}"; do
        local value
        value=$(awk -v network="$network" -v field="$field" '
            /^\[networks\./ {
                current_network = substr($0, 11, length($0)-11)
                gsub(/\]/, "", current_network)
            }
            current_network == network && $0 ~ field {
                gsub(/.*= *"/, "")
                gsub(/".*/, "")
                print
                exit
            }
        ' "$NETWORKS_CONFIG")
        
        if [[ -z "$value" ]]; then
            print_test_result "Network $network - $field" "FAIL" "Missing required field"
            all_fields_present=false
        else
            print_test_result "Network $network - $field" "PASS" "Field present"
        fi
    done
    
    if [[ "$all_fields_present" == "true" ]]; then
        return 0
    else
        return 1
    fi
}

# Test network connectivity
test_network_connectivity() {
    local network="$1"
    local rpc_url
    rpc_url=$(awk -v network="$network" '
        /^\[networks\./ {
            current_network = substr($0, 11, length($0)-11)
            gsub(/\]/, "", current_network)
        }
        current_network == network && $0 ~ "rpc-url" {
            gsub(/.*= *"/, "")
            gsub(/".*/, "")
            print
            exit
        }
    ' "$NETWORKS_CONFIG")
    
    if [[ -z "$rpc_url" ]]; then
        print_test_result "Network $network - RPC URL" "FAIL" "RPC URL not found"
        return 1
    fi
    
    # Test connectivity with timeout
    if curl -s --connect-timeout 10 --max-time 30 "$rpc_url" > /dev/null 2>&1; then
        print_test_result "Network $network - Connectivity" "PASS" "Network reachable"
        return 0
    else
        print_test_result "Network $network - Connectivity" "FAIL" "Network not reachable"
        return 1
    fi
}

# Test Soroban CLI configuration
test_soroban_config() {
    local network="$1"
    
    if soroban config network show "$network" &> /dev/null; then
        print_test_result "Soroban Config - $network" "PASS" "Network configured in Soroban"
        return 0
    else
        print_test_result "Soroban Config - $network" "FAIL" "Network not configured in Soroban"
        return 1
    fi
}

# Test identity configuration
test_identity_config() {
    local identity="${1:-default}"
    
    if soroban config identity show "$identity" &> /dev/null; then
        local address
        address=$(soroban config identity address "$identity" 2>/dev/null || echo "")
        print_test_result "Identity $identity" "PASS" "Identity configured: $address"
        return 0
    else
        print_test_result "Identity $identity" "FAIL" "Identity not configured"
        return 1
    fi
}

# Test deployment prerequisites
test_deployment_prerequisites() {
    local contract_name="$1"
    local network="$2"
    
    # Test contract directory exists
    local contract_dir="contracts/$contract_name"
    if [[ -d "$contract_dir" ]]; then
        print_test_result "Contract Directory" "PASS" "Contract directory exists"
    else
        print_test_result "Contract Directory" "FAIL" "Contract directory not found: $contract_dir"
        return 1
    fi
    
    # Test Cargo.toml exists
    if [[ -f "$contract_dir/Cargo.toml" ]]; then
        print_test_result "Cargo.toml" "PASS" "Cargo.toml exists"
    else
        print_test_result "Cargo.toml" "FAIL" "Cargo.toml not found"
        return 1
    fi
    
    # Test if WASM can be built (dry run)
    if cargo check -p "$contract_name" --target wasm32-unknown-unknown 2>/dev/null; then
        print_test_result "Contract Build Check" "PASS" "Contract can be built"
    else
        print_test_result "Contract Build Check" "FAIL" "Contract build check failed"
        return 1
    fi
    
    return 0
}

# Test environment variables
test_environment_variables() {
    local required_vars=("SOROBAN_RPC_URL" "SOROBAN_NETWORK_PASSPHRASE")
    local all_vars_set=true
    
    for var in "${required_vars[@]}"; do
        if [[ -n "${!var:-}" ]]; then
            print_test_result "Environment Variable $var" "PASS" "Variable is set"
        else
            print_test_result "Environment Variable $var" "WARN" "Variable not set (optional)"
        fi
    done
    
    return 0
}

# Test security configurations
test_security_configurations() {
    local network="$1"
    
    # Test if mainnet requires confirmation
    if [[ "$network" == "mainnet" ]]; then
        local confirmation_required
        confirmation_required=$(awk -v network="$network" '
            /^\[networks\./ {
                current_network = substr($0, 11, length($0)-11)
                gsub(/\]/, "", current_network)
            }
            current_network == network && $0 ~ "confirmation-required" {
                gsub(/.*= */, "")
                gsub(/".*/, "")
                print
                exit
            }
        ' "$NETWORKS_CONFIG")
        
        if [[ "$confirmation_required" == "true" ]]; then
            print_test_result "Mainnet Safety" "PASS" "Confirmation required for mainnet"
        else
            print_test_result "Mainnet Safety" "WARN" "Mainnet confirmation not required"
        fi
    fi
    
    return 0
}

# Generate validation report
generate_report() {
    echo
    echo "========================================"
    echo "VALIDATION REPORT"
    echo "========================================"
    echo "Total Tests: $TESTS_TOTAL"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    
    local success_rate=0
    if [[ $TESTS_TOTAL -gt 0 ]]; then
        success_rate=$((TESTS_PASSED * 100 / TESTS_TOTAL))
    fi
    
    echo "Success Rate: $success_rate%"
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo -e "\n${GREEN}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "\n${RED}❌ Some tests failed. Please review the issues above.${NC}"
        return 1
    fi
}

# Main validation function
run_validation() {
    local network="${1:-}"
    local contract="${2:-}"
    local identity="${3:-default}"
    
    print_status "Starting comprehensive network configuration validation..."
    echo
    
    # Basic file tests
    test_file_exists "$NETWORKS_CONFIG" "Network Configuration File"
    test_toml_syntax "$NETWORKS_CONFIG"
    
    # Network tests
    local networks=("local" "testnet" "futurenet" "mainnet")
    if [[ -n "$network" ]]; then
        networks=("$network")
    fi
    
    for net in "${networks[@]}"; do
        echo
        print_step "Testing network: $net"
        test_network_config_completeness "$net"
        test_network_connectivity "$net"
        test_soroban_config "$net"
        test_security_configurations "$net"
    done
    
    # Identity tests
    echo
    print_step "Testing identity configuration"
    test_identity_config "$identity"
    
    # Environment tests
    echo
    print_step "Testing environment configuration"
    test_environment_variables
    
    # Deployment tests (if contract specified)
    if [[ -n "$contract" ]]; then
        echo
        print_step "Testing deployment prerequisites for: $contract"
        test_deployment_prerequisites "$contract" "${network:-testnet}"
    fi
    
    # Generate report
    generate_report
}

# Help function
show_help() {
    cat << EOF
Network Configuration Validation Tool

Usage: $0 [options]

Options:
    --network <name>       Validate specific network (default: all networks)
    --contract <name>      Validate deployment prerequisites for contract
    --identity <name>      Validate specific identity (default: default)
    --help                 Show this help message

Examples:
    $0                                    # Validate all configurations
    $0 --network testnet                 # Validate testnet only
    $0 --network mainnet --contract medical_records  # Validate mainnet deployment for medical_records
    $0 --contract medical_records --identity alice    # Validate deployment with specific identity

EOF
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --network)
                NETWORK="$2"
                shift 2
                ;;
            --contract)
                CONTRACT="$2"
                shift 2
                ;;
            --identity)
                IDENTITY="$2"
                shift 2
                ;;
            --help|--help|-h)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Main execution
main() {
    local NETWORK=""
    local CONTRACT=""
    local IDENTITY="default"
    
    parse_arguments "$@"
    run_validation "$NETWORK" "$CONTRACT" "$IDENTITY"
}

# Execute main function with all arguments
main "$@"
