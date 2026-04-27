#!/bin/bash

# deploy_enhanced.sh - Enhanced Soroban Contract Deployment with Network Configuration Management
# Usage: ./scripts/deploy_enhanced.sh <contract_name> <network> [identity] [options]

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
NETWORK_MANAGER="$SCRIPT_DIR/network_manager.sh"

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

# Parse command line arguments
parse_arguments() {
    CONTRACT_NAME=""
    NETWORK=""
    IDENTITY="default"
    DRY_RUN=false
    SIMULATION=false
    FORCE=false
    AUTO_FALLBACK=false
    SKIP_BUILD=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --simulation)
                SIMULATION=true
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --auto-fallback)
                AUTO_FALLBACK=true
                shift
                ;;
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --debug)
                DEBUG=true
                shift
                ;;
            --identity)
                IDENTITY="$2"
                shift 2
                ;;
            -*)
                print_error "Unknown option: $1"
                exit 1
                ;;
            *)
                if [[ -z "$CONTRACT_NAME" ]]; then
                    CONTRACT_NAME="$1"
                elif [[ -z "$NETWORK" ]]; then
                    NETWORK="$1"
                else
                    print_error "Too many arguments"
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Validate required arguments
    if [[ -z "$CONTRACT_NAME" || -z "$NETWORK" ]]; then
        print_error "Usage: $0 <contract_name> <network> [--identity <name>] [--dry-run] [--simulation] [--force] [--auto-fallback] [--skip-build] [--debug]"
        exit 1
    fi
}

# Validate contract exists
validate_contract() {
    local contract_dir="contracts/$CONTRACT_NAME"
    if [[ ! -d "$contract_dir" ]]; then
        print_error "Contract directory '$contract_dir' does not exist"
        exit 1
    fi
    
    # Check if Cargo.toml exists
    if [[ ! -f "$contract_dir/Cargo.toml" ]]; then
        print_error "Cargo.toml not found in contract directory"
        exit 1
    fi
    
    print_success "✓ Contract validation passed"
}

# Build contract
build_contract() {
    if [[ "$SKIP_BUILD" == "true" ]]; then
        print_status "Skipping build as requested"
        return 0
    fi
    
    print_step "Building contract: $CONTRACT_NAME"
    
    # Clean previous builds
    print_debug "Cleaning previous builds..."
    # cargo clean -p "$CONTRACT_NAME" || print_warning "Cargo clean failed, continuing..."
    
    # Build for WebAssembly target
    if ! cargo build -p "$CONTRACT_NAME" --target wasm32-unknown-unknown --release; then
        print_error "Cargo build failed"
        exit 1
    fi
    
    # Check if build was successful
    local wasm_file="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
    if [[ ! -f "$wasm_file" ]]; then
        print_error "Build failed: $wasm_file not found"
        exit 1
    fi
    
    # Optimize the contract
    if command -v soroban &> /dev/null; then
        print_step "Optimizing contract..."
        if soroban contract optimize --wasm "$wasm_file"; then
            print_success "✓ Contract optimized"
        else
            print_warning "⚠ Optimization failed, continuing with unoptimized contract"
        fi
    fi
    
    print_success "✓ Contract built successfully"
}

# Setup network configuration
setup_network() {
    print_step "Setting up network configuration..."
    
    # Validate network configuration
    if ! "$NETWORK_MANAGER" validate "$NETWORK"; then
        if [[ "$AUTO_FALLBACK" == "true" ]]; then
            print_status "Auto-fallback enabled, finding alternative network..."
            NETWORK=$("$NETWORK_MANAGER" fallback "$NETWORK")
            print_status "Using fallback network: $NETWORK"
        else
            print_error "Network validation failed"
            exit 1
        fi
    fi
    
    # Configure network
    local force_flag=""
    if [[ "$FORCE" == "true" ]]; then
        force_flag="--force"
    fi
    
    if ! FORCE=$FORCE "$NETWORK_MANAGER" configure "$NETWORK"; then
        print_error "Failed to configure network"
        exit 1
    fi
    
    print_success "✓ Network configured: $NETWORK"
}

# Setup identity
setup_identity() {
    print_step "Setting up identity: $IDENTITY"
    
    # Ensure identity exists
    if ! soroban config identity show "$IDENTITY" &> /dev/null; then
        print_warning "Identity '$IDENTITY' not found, generating new one..."
        if ! soroban config identity generate "$IDENTITY"; then
            print_error "Failed to generate identity '$IDENTITY'"
            exit 1
        fi
    fi
    
    # Get identity address
    local identity_address
    identity_address=$(soroban config identity address "$IDENTITY")
    print_success "✓ Identity ready: $IDENTITY ($identity_address)"
}

# Fund account if needed
fund_account() {
    # Check if network requires funding
    local requires_funding
    requires_funding=$("$NETWORK_MANAGER" parse "$NETWORK" "requires-funding" 2>/dev/null || echo "false")
    
    if [[ "$requires_funding" == "true" ]]; then
        print_step "Funding account on $NETWORK..."
        if soroban config identity fund "$IDENTITY" --network "$NETWORK"; then
            print_success "✓ Account funded successfully"
        else
            print_warning "⚠ Failed to fund account, continuing anyway..."
        fi
    else
        print_status "Network does not require funding"
    fi
}

# Simulate deployment
simulate_deployment() {
    if [[ "$SIMULATION" == "true" || "$DRY_RUN" == "true" ]]; then
        print_step "Simulating deployment..."
        
        local wasm_file="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
        local simulation_result
        
        simulation_result=$(soroban contract deploy \
            --wasm "$wasm_file" \
            --source "$IDENTITY" \
            --network "$NETWORK" \
            --dry-run 2>&1) || {
            print_error "Simulation failed"
            echo "$simulation_result"
            exit 1
        }
        
        print_success "✓ Simulation completed successfully"
        print_debug "Simulation result: $simulation_result"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            print_status "DRY-RUN MODE: Deployment not executed"
            exit 0
        fi
    fi
}

# Deploy contract
deploy_contract() {
    print_step "Deploying contract to $NETWORK..."
    
    local wasm_file="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
    local contract_id
    
    if [[ "$SIMULATION" == "true" ]]; then
        print_status "SIMULATION MODE: Using simulated deployment"
        contract_id="SIMULATED_CONTRACT_ID_$(date +%s)"
    else
        contract_id=$(soroban contract deploy \
            --wasm "$wasm_file" \
            --source "$IDENTITY" \
            --network "$NETWORK" 2>/dev/null) || {
            print_error "Deployment command failed"
            exit 1
        }
    fi
    
    if [[ -n "$contract_id" ]]; then
        print_success "✓ Contract deployed successfully!"
        echo "Contract ID: $contract_id"
        echo "Network: $NETWORK"
        echo "Identity: $IDENTITY"
        
        # Save deployment info
        save_deployment_info "$contract_id"
    else
        print_error "Deployment failed: No contract ID returned"
        exit 1
    fi
}

# Save deployment information
save_deployment_info() {
    local contract_id="$1"
    local deployment_file="deployments/${CONTRACT_NAME}_${NETWORK}.json"
    
    # Create deployments directory if it doesn't exist
    mkdir -p "deployments"
    
    # Create deployment info
    cat > "$deployment_file" << EOF
{
  "contract_name": "$CONTRACT_NAME",
  "contract_id": "$contract_id",
  "network": "$NETWORK",
  "identity": "$IDENTITY",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "wasm_file": "target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm",
  "deployment_type": "enhanced",
  "simulation": $SIMULATION,
  "dry_run": $DRY_RUN
}
EOF
    
    print_success "✓ Deployment info saved to: $deployment_file"
}

# Safety check for mainnet
mainnet_safety_check() {
    if [[ "$NETWORK" == "mainnet" && "$DRY_RUN" != "true" && "$SIMULATION" != "true" ]]; then
        print_warning "🚨 MAINNET DEPLOYMENT SAFETY CHECK 🚨"
        echo "You are about to deploy '$CONTRACT_NAME' to MAINNET"
        echo "This will use REAL funds and cannot be undone!"
        echo
        
        if [[ "$FORCE" != "true" ]]; then
            echo -e "${RED}Type 'CONFIRM' to proceed with mainnet deployment:${NC}"
            read -r confirmation
            
            if [[ "$confirmation" != "CONFIRM" ]]; then
                print_error "Mainnet deployment cancelled"
                exit 1
            fi
        fi
        
        print_success "Mainnet deployment confirmed"
    fi
}

# Main execution
main() {
    print_status "Enhanced Soroban Contract Deployment"
    echo "Contract: $CONTRACT_NAME"
    echo "Network: $NETWORK"
    echo "Identity: $IDENTITY"
    echo "Dry Run: $DRY_RUN"
    echo "Simulation: $SIMULATION"
    echo
    
    # Check dependencies
    if [[ ! -f "$NETWORK_MANAGER" ]]; then
        print_error "Network manager not found: $NETWORK_MANAGER"
        exit 1
    fi
    
    # Execute deployment steps
    validate_contract
    mainnet_safety_check
    build_contract
    setup_network
    setup_identity
    fund_account
    simulate_deployment
    deploy_contract
    
    print_success "🎉 Deployment completed successfully!"
}

# Parse arguments and run main
parse_arguments "$@"
main
