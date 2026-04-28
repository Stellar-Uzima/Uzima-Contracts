#!/bin/bash

# network_manager.sh - Comprehensive Soroban Network Configuration Manager
# Provides network validation, auto-detection, and safety features

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
SOROBAN_CONFIG="$HOME/.config/soroban/config.toml"

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

print_debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        echo -e "${PURPLE}[DEBUG]${NC} $1"
    fi
}

# Check dependencies
check_dependencies() {
    local deps=("soroban" "curl" "jq")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            print_error "Required dependency '$dep' is not installed"
            exit 1
        fi
    done
}

# Parse TOML configuration (simplified parser)
parse_network_config() {
    local network="$1"
    local field="$2"
    
    if [[ ! -f "$NETWORKS_CONFIG" ]]; then
        print_error "Network configuration file not found: $NETWORKS_CONFIG"
        exit 1
    fi
    
    # Simple TOML parsing using awk and sed
    local result
    result=$(awk -v network="$network" -v field="$field" '
        /^\[networks\./ {
            current_network = substr($0, 11, length($0)-11)
            gsub(/\]/, "", current_network)
        }
        current_network == network && $0 ~ field {
            gsub(/.*= *"/, "")
            gsub(/".*/, "")
            print
        }
    ' "$NETWORKS_CONFIG")
    
    echo "$result"
}

# Validate network connectivity
validate_network_connectivity() {
    local network="$1"
    local rpc_url
    rpc_url=$(parse_network_config "$network" "rpc-url")
    
    print_step "Validating connectivity to $network network..."
    
    if curl -s --connect-timeout 10 "$rpc_url" > /dev/null; then
        print_success "✓ Network $network is reachable"
        return 0
    else
        print_warning "⚠ Network $network is not reachable"
        return 1
    fi
}

# Auto-detect available networks
auto_detect_networks() {
    print_status "Auto-detecting available networks..."
    
    local networks=("local" "testnet" "futurenet" "mainnet")
    local available_networks=()
    
    for network in "${networks[@]}"; do
        if validate_network_connectivity "$network" 2>/dev/null; then
            available_networks+=("$network")
        fi
    done
    
    if [[ ${#available_networks[@]} -eq 0 ]]; then
        print_warning "No networks are currently reachable"
        return 1
    fi
    
    print_success "Available networks: ${available_networks[*]}"
    printf '%s\n' "${available_networks[@]}"
}

# Configure network in Soroban
configure_network() {
    local network="$1"
    local force="${2:-false}"
    
    local rpc_url network_passphrase name
    rpc_url=$(parse_network_config "$network" "rpc-url")
    network_passphrase=$(parse_network_config "$network" "network-passphrase")
    name=$(parse_network_config "$network" "name")
    
    if [[ -z "$rpc_url" || -z "$network_passphrase" ]]; then
        print_error "Invalid network configuration for $network"
        exit 1
    fi
    
    print_step "Configuring network: $name ($network)"
    
    # Check if network already exists
    if soroban config network show "$network" &>/dev/null && [[ "$force" != "true" ]]; then
        print_warning "Network '$network' already configured"
        return 0
    fi
    
    # Add network configuration
    if soroban config network add "$network" \
        --rpc-url "$rpc_url" \
        --network-passphrase "$network_passphrase"; then
        print_success "✓ Network '$network' configured successfully"
    else
        if [[ "$force" == "true" ]]; then
            soroban config network remove "$network" 2>/dev/null || true
            soroban config network add "$network" \
                --rpc-url "$rpc_url" \
                --network-passphrase "$network_passphrase"
            print_success "✓ Network '$network' reconfigured successfully"
        else
            print_error "Failed to configure network '$network'"
            exit 1
        fi
    fi
}

# Safety checks for mainnet operations
mainnet_safety_check() {
    local operation="$1"
    
    print_warning "🚨 MAINNET SAFETY CHECK 🚨"
    echo "You are about to perform a '$operation' operation on MAINNET"
    echo "This will use REAL funds and cannot be undone!"
    echo
    
    # Check if dry-run is enabled
    if [[ "${DRY_RUN:-false}" == "true" ]]; then
        print_status "DRY-RUN MODE: No actual transactions will be executed"
        return 0
    fi
    
    # Check if simulation is enabled
    if [[ "${SIMULATION:-false}" == "true" ]]; then
        print_status "SIMULATION MODE: Transactions will be simulated only"
        return 0
    fi
    
    # Interactive confirmation
    echo -e "${RED}Type 'CONFIRM' to proceed with mainnet operation:${NC}"
    read -r confirmation
    
    if [[ "$confirmation" != "CONFIRM" ]]; then
        print_error "Mainnet operation cancelled"
        exit 1
    fi
    
    print_success "Mainnet operation confirmed"
}

# Validate configuration
validate_configuration() {
    local network="$1"
    
    print_step "Validating configuration for network: $network"
    
    # Check if network exists in config
    if ! parse_network_config "$network" "name" >/dev/null; then
        print_error "Network '$network' not found in configuration"
        exit 1
    fi
    
    # Validate required fields
    local required_fields=("rpc-url" "network-passphrase" "name")
    for field in "${required_fields[@]}"; do
        local value
        value=$(parse_network_config "$network" "$field")
        if [[ -z "$value" ]]; then
            print_error "Required field '$field' is missing for network '$network'"
            exit 1
        fi
    done
    
    print_success "✓ Configuration validation passed"
}

# Fallback mechanism
get_fallback_network() {
    local preferred_network="$1"
    
    print_status "Attempting fallback from '$preferred_network'..."
    
    # Try local first, then testnet
    local fallback_networks=("local" "testnet")
    
    for network in "${fallback_networks[@]}"; do
        if validate_network_connectivity "$network" 2>/dev/null; then
            print_status "Using fallback network: $network"
            echo "$network"
            return 0
        fi
    done
    
    print_error "No fallback networks available"
    exit 1
}

# Environment detection
detect_environment() {
    if [[ "${CI:-false}" == "true" ]]; then
        echo "ci"
    elif [[ -f ".env" ]] && grep -q "NODE_ENV=production" .env; then
        echo "production"
    elif [[ -f ".env" ]] && grep -q "NODE_ENV=test" .env; then
        echo "testing"
    else
        echo "development"
    fi
}

# Main configuration function
configure_all_networks() {
    local force="${1:-false}"
    
    print_status "Configuring all networks..."
    
    local networks=("local" "testnet" "futurenet" "mainnet")
    
    for network in "${networks[@]}"; do
        if validate_configuration "$network"; then
            configure_network "$network" "$force"
        fi
    done
    
    print_success "✓ All networks configured"
}

# Show network status
show_network_status() {
    local network="${1:-}"
    
    if [[ -n "$network" ]]; then
        # Show specific network status
        if soroban config network show "$network" &>/dev/null; then
            print_status "Network '$network' is configured"
            local name description
            name=$(parse_network_config "$network" "name")
            description=$(parse_network_config "$network" "description")
            echo "  Name: $name"
            echo "  Description: $description"
            
            # Test connectivity
            if validate_network_connectivity "$network"; then
                echo "  Status: ✅ Connected"
            else
                echo "  Status: ❌ Disconnected"
            fi
        else
            print_warning "Network '$network' is not configured"
        fi
    else
        # Show all networks status
        print_status "Network Configuration Status"
        echo
        
        local networks=("local" "testnet" "futurenet" "mainnet")
        for network in "${networks[@]}"; do
            show_network_status "$network"
            echo
        done
    fi
}

# Help function
show_help() {
    cat << EOF
Soroban Network Configuration Manager

Usage: $0 <command> [options]

Commands:
    configure <network>     Configure a specific network
    configure-all           Configure all networks
    status [network]        Show network status
    validate <network>      Validate network configuration
    detect                  Auto-detect available networks
    fallback <network>      Get fallback network
    help                    Show this help message

Environment Variables:
    DRY_RUN=true            Enable dry-run mode
    SIMULATION=true         Enable simulation mode
    DEBUG=true              Enable debug output
    FORCE=true              Force reconfiguration

Examples:
    $0 configure testnet
    $0 configure-all
    $0 status
    $0 validate mainnet
    $0 detect
    DRY_RUN=true $0 configure mainnet

EOF
}

# Main execution
main() {
    check_dependencies
    
    case "${1:-}" in
        "configure")
            if [[ -z "${2:-}" ]]; then
                print_error "Network name required"
                show_help
                exit 1
            fi
            validate_configuration "$2"
            if [[ "$2" == "mainnet" ]]; then
                mainnet_safety_check "configure"
            fi
            configure_network "$2" "${FORCE:-false}"
            ;;
        "configure-all")
            configure_all_networks "${FORCE:-false}"
            ;;
        "status")
            show_network_status "${2:-}"
            ;;
        "validate")
            if [[ -z "${2:-}" ]]; then
                print_error "Network name required"
                show_help
                exit 1
            fi
            validate_configuration "$2"
            validate_network_connectivity "$2"
            ;;
        "detect")
            auto_detect_networks
            ;;
        "fallback")
            if [[ -z "${2:-}" ]]; then
                print_error "Preferred network required"
                show_help
                exit 1
            fi
            get_fallback_network "$2"
            ;;
        "help"|"--help"|"-h")
            show_help
            ;;
        *)
            print_error "Unknown command: ${1:-}"
            show_help
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"
