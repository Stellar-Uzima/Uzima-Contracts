#!/bin/bash

# Multi-Region Disaster Recovery System Deployment Script
# This script deploys all DR contracts and configures multi-region failover

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/target/wasm32-unknown-unknown/release"

# Configuration
NETWORK="${1:-testnet}"
DR_CONFIG="$PROJECT_ROOT/config/multi_region_dr.json"
DEPLOYER="${2:-deployer-$NETWORK}"
DEPLOYMENT_DIR="$PROJECT_ROOT/deployments"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Verify prerequisites
verify_prerequisites() {
    log_info "Verifying prerequisites..."
    
    if ! command -v soroban &> /dev/null; then
        log_error "Soroban CLI not found. Please install it first."
        exit 1
    fi
    
    if [ ! -f "$DR_CONFIG" ]; then
        log_error "Multi-region configuration file not found: $DR_CONFIG"
        exit 1
    fi
    
    if [ ! -d "$BUILD_DIR" ]; then
        log_warn "WASM build directory not found. Building contracts..."
        cd "$PROJECT_ROOT"
        cargo build --target wasm32-unknown-unknown --release
    fi
    
    log_success "Prerequisites verified"
}

# Build all DR contracts
build_contracts() {
    log_info "Building Disaster Recovery contracts..."
    
    cd "$PROJECT_ROOT"
    
    contracts=(
        "multi_region_orchestrator"
        "regional_node_manager"
        "failover_detector"
        "sync_manager"
    )
    
    for contract in "${contracts[@]}"; do
        log_info "Building $contract..."
        cargo build --target wasm32-unknown-unknown --release -p "$contract" 2>&1 | grep -E "(Compiling|Finished|error)" || true
    done
    
    # Optimize WASM
    log_info "Optimizing WASM binaries..."
    for contract in "${contracts[@]}"; do
        WASM_FILE="$BUILD_DIR/${contract//_/-}.wasm"
        if [ -f "$WASM_FILE" ]; then
            soroban contract optimize --wasm "$WASM_FILE" 2>/dev/null || true
        fi
    done
    
    log_success "Contracts built successfully"
}

# Configure network
configure_network() {
    log_info "Configuring network: $NETWORK"
    
    case "$NETWORK" in
        local)
            soroban config network add local \
                --rpc-url http://localhost:8000/soroban/rpc \
                --network-passphrase "Standalone Network ; February 2017" 2>/dev/null || true
            ;;
        testnet)
            soroban config network add testnet \
                --rpc-url https://soroban-testnet.stellar.org:443 \
                --network-passphrase "Test SDF Network ; September 2015" 2>/dev/null || true
            ;;
        futurenet)
            soroban config network add futurenet \
                --rpc-url https://rpc-futurenet.stellar.org:443 \
                --network-passphrase "Test SDF Future Network ; October 2022" 2>/dev/null || true
            ;;
        *)
            log_error "Unknown network: $NETWORK"
            exit 1
            ;;
    esac
    
    log_success "Network configured"
}

# Generate or select identity
setup_identity() {
    log_info "Setting up identity: $DEPLOYER"
    
    if ! soroban config identity ls | grep -q "$DEPLOYER"; then
        log_info "Generating new identity: $DEPLOYER"
        soroban config identity generate "$DEPLOYER"
    fi
    
    log_success "Identity ready: $DEPLOYER"
}

# Deploy contract
deploy_contract() {
    local contract_name=$1
    local wasm_name=$2
    
    log_info "Deploying $contract_name..."
    
    local wasm_file="$BUILD_DIR/${wasm_name}.wasm"
    
    if [ ! -f "$wasm_file" ]; then
        log_error "WASM file not found: $wasm_file"
        return 1
    fi
    
    local contract_id=$(soroban contract deploy \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --wasm "$wasm_file" 2>&1 | tail -1)
    
    if [[ ! "$contract_id" =~ ^C[A-Z0-9]{55}$ ]]; then
        log_error "Failed to deploy $contract_name"
        echo "$contract_id"
        return 1
    fi
    
    log_success "Deployed $contract_name: $contract_id"
    echo "$contract_id"
}

# Deploy all DR contracts
deploy_all_contracts() {
    log_info "Deploying all Disaster Recovery contracts..."
    
    mkdir -p "$DEPLOYMENT_DIR"
    
    local deployment_file="$DEPLOYMENT_DIR/${NETWORK}_dr_deployment_$(date +%s).json"
    
    # Deploy contracts
    local mro_id=$(deploy_contract "Multi Region Orchestrator" "multi-region-orchestrator")
    local rnm_id=$(deploy_contract "Regional Node Manager" "regional-node-manager")
    local fd_id=$(deploy_contract "Failover Detector" "failover-detector")
    local sm_id=$(deploy_contract "Sync Manager" "sync-manager")
    
    # Save deployment info
    cat > "$deployment_file" << EOF
{
  "network": "$NETWORK",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "contracts": {
    "multi_region_orchestrator": "$mro_id",
    "regional_node_manager": "$rnm_id",
    "failover_detector": "$fd_id",
    "sync_manager": "$sm_id"
  },
  "deployer": "$DEPLOYER"
}
EOF
    
    log_success "All contracts deployed successfully"
    log_info "Deployment details saved to: $deployment_file"
    
    echo "$mro_id|$rnm_id|$fd_id|$sm_id"
}

# Initialize contracts
initialize_contracts() {
    local mro_id=$1
    local rnm_id=$2
    local fd_id=$3
    local sm_id=$4
    
    log_info "Initializing contracts..."
    
    local admin=$(soroban config identity address "$DEPLOYER")
    
    # Initialize Multi Region Orchestrator
    log_info "Initializing Multi Region Orchestrator..."
    soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$mro_id" \
        -- initialize \
        --admin "$admin" > /dev/null 2>&1
    
    # Initialize Regional Node Manager
    log_info "Initializing Regional Node Manager..."
    soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$rnm_id" \
        -- initialize \
        --admin "$admin" > /dev/null 2>&1
    
    # Initialize Failover Detector
    log_info "Initializing Failover Detector..."
    soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$fd_id" \
        -- initialize \
        --admin "$admin" > /dev/null 2>&1
    
    # Initialize Sync Manager
    log_info "Initializing Sync Manager..."
    soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$sm_id" \
        -- initialize \
        --admin "$admin" > /dev/null 2>&1
    
    log_success "Contracts initialized successfully"
}

# Configure regions
configure_regions() {
    local mro_id=$1
    
    log_info "Configuring geographic regions..."
    
    local admin=$(soroban config identity address "$DEPLOYER")
    
    # Assign operator role
    soroban contract invoke \
        --network "$NETWORK" \
        --source "$DEPLOYER" \
        --id "$mro_id" \
        -- assign_role \
        --caller "$admin" \
        --user "$admin" \
        --role_mask 3 > /dev/null 2>&1 || true
    
    log_success "Regions configured"
}

# Main execution
main() {
    log_info "Starting Multi-Region DR deployment..."
    log_info "Network: $NETWORK"
    log_info "Deployer: $DEPLOYER"
    
    verify_prerequisites
    build_contracts
    configure_network
    setup_identity
    
    local contract_ids=$(deploy_all_contracts)
    local mro_id=$(echo "$contract_ids" | cut -d'|' -f1)
    local rnm_id=$(echo "$contract_ids" | cut -d'|' -f2)
    local fd_id=$(echo "$contract_ids" | cut -d'|' -f3)
    local sm_id=$(echo "$contract_ids" | cut -d'|' -f4)
    
    initialize_contracts "$mro_id" "$rnm_id" "$fd_id" "$sm_id"
    configure_regions "$mro_id"
    
    log_success "Multi-Region Disaster Recovery System deployed successfully!"
    log_info "Contract IDs:"
    echo "  Multi Region Orchestrator: $mro_id"
    echo "  Regional Node Manager: $rnm_id"
    echo "  Failover Detector: $fd_id"
    echo "  Sync Manager: $sm_id"
}

main "$@"
