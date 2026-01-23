#!/bin/bash

# deploy_with_rollback.sh - Enhanced deployment script with rollback support
# Usage: ./scripts/deploy_with_rollback.sh <contract_name> <network> [identity] [--no-rollback]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
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

# Check if required arguments are provided
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <contract_name> <network> [identity] [--no-rollback]"
    print_error "Example: $0 medical_records testnet alice"
    exit 1
fi

CONTRACT_NAME="$1"
NETWORK="$2"
IDENTITY="${3:-"default"}"
ENABLE_ROLLBACK=true

# Check for --no-rollback flag
if [[ "$*" == *"--no-rollback"* ]]; then
    ENABLE_ROLLBACK=false
fi

# Validate contract exists
CONTRACT_DIR="contracts/$CONTRACT_NAME"
if [ ! -d "$CONTRACT_DIR" ]; then
    print_error "Contract directory '$CONTRACT_DIR' does not exist"
    exit 1
fi

# Create deployments directory
DEPLOYMENTS_DIR="deployments"
mkdir -p "$DEPLOYMENTS_DIR"

# Backup file for rollback
BACKUP_FILE="$DEPLOYMENTS_DIR/${NETWORK}_${CONTRACT_NAME}_backup_$(date +%Y%m%d_%H%M%S).json"
DEPLOYMENT_FILE="$DEPLOYMENTS_DIR/${NETWORK}_${CONTRACT_NAME}.json"

# Function to get current deployment
get_current_deployment() {
    if [ -f "$DEPLOYMENT_FILE" ]; then
        jq -r '.contract_id' "$DEPLOYMENT_FILE" 2>/dev/null || echo ""
    else
        echo ""
    fi
}

# Function to save backup
save_backup() {
    local current_id=$(get_current_deployment)
    if [ -n "$current_id" ] && [ "$current_id" != "null" ]; then
        print_step "Saving backup of current deployment..."
        cat > "$BACKUP_FILE" << EOF
{
    "contract_name": "$CONTRACT_NAME",
    "contract_id": "$current_id",
    "network": "$NETWORK",
    "backed_up_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "backup_reason": "pre_deployment"
}
EOF
        print_status "Backup saved to: $BACKUP_FILE"
        echo "$BACKUP_FILE"
    else
        echo ""
    fi
}

# Function to rollback
rollback() {
    local backup_file="$1"
    if [ ! -f "$backup_file" ]; then
        print_error "Backup file not found: $backup_file"
        return 1
    fi
    
    print_warning "Rolling back to previous deployment..."
    local old_contract_id=$(jq -r '.contract_id' "$backup_file")
    
    if [ -n "$old_contract_id" ] && [ "$old_contract_id" != "null" ]; then
        print_status "Previous contract ID: $old_contract_id"
        # Restore deployment file
        jq --arg id "$old_contract_id" \
           --arg name "$CONTRACT_NAME" \
           --arg network "$NETWORK" \
           --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
           '{
             contract_name: $name,
             contract_id: $id,
             network: $network,
             deployed_at: $timestamp,
             rolled_back: true
           }' <<< "{}" > "$DEPLOYMENT_FILE"
        print_status "Rollback complete. Deployment file restored."
    else
        print_error "Invalid backup file format"
        return 1
    fi
}

# Function to verify deployment
verify_deployment() {
    local contract_id="$1"
    print_step "Verifying deployment..."
    
    # Try to invoke a simple function or check contract exists
    if soroban contract invoke \
        --id "$contract_id" \
        --source "$IDENTITY" \
        --network "$NETWORK" \
        -- --help &> /dev/null; then
        print_status "Contract verification successful"
        return 0
    else
        print_warning "Contract verification failed (may be expected for some contracts)"
        return 0  # Don't fail on verification, as some contracts may not have --help
    fi
}

# Main deployment function
main() {
    print_status "Starting deployment of '$CONTRACT_NAME' to '$NETWORK' network"
    
    # Save backup if rollback is enabled
    local backup_file=""
    if [ "$ENABLE_ROLLBACK" = true ]; then
        backup_file=$(save_backup)
    fi
    
    # Build the contract
    print_step "Building contract..."
    cd "$CONTRACT_DIR" || exit 1
    
    cargo clean || true
    if ! cargo build --target wasm32-unknown-unknown --release; then
        print_error "Build failed"
        exit 1
    fi
    
    WASM_FILE="target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
    if [ ! -f "$WASM_FILE" ]; then
        print_error "Build failed: $WASM_FILE not found"
        exit 1
    fi
    
    # Optimize if available
    if command -v soroban &> /dev/null; then
        print_step "Optimizing contract..."
        soroban contract optimize --wasm "$WASM_FILE" || print_warning "Optimization failed, continuing..."
    fi
    
    cd - > /dev/null || exit 1
    
    # Configure network
    print_step "Configuring network..."
    case $NETWORK in
        "local")
            soroban config network add local \
                --rpc-url http://localhost:8000/soroban/rpc \
                --network-passphrase "Standalone Network ; February 2017" 2>/dev/null || true
            ;;
        "testnet")
            soroban config network add testnet \
                --rpc-url https://soroban-testnet.stellar.org:443 \
                --network-passphrase "Test SDF Network ; September 2015" 2>/dev/null || true
            ;;
        "futurenet")
            soroban config network add futurenet \
                --rpc-url https://rpc-futurenet.stellar.org:443 \
                --network-passphrase "Test SDF Future Network ; October 2022" 2>/dev/null || true
            ;;
        "mainnet")
            soroban config network add mainnet \
                --rpc-url https://soroban-rpc.stellar.org:443 \
                --network-passphrase "Public Global Stellar Network ; September 2015" 2>/dev/null || true
            ;;
        *)
            print_error "Unknown network: $NETWORK"
            exit 1
            ;;
    esac
    
    # Ensure identity exists
    print_step "Checking identity..."
    if ! soroban config identity show "$IDENTITY" &> /dev/null; then
        print_warning "Identity '$IDENTITY' not found, generating..."
        soroban config identity generate "$IDENTITY" || exit 1
    fi
    
    # Fund account for testnet/futurenet
    if [ "$NETWORK" = "testnet" ] || [ "$NETWORK" = "futurenet" ]; then
        print_step "Funding account..."
        soroban config identity fund "$IDENTITY" --network "$NETWORK" || print_warning "Funding failed, continuing..."
    fi
    
    # Deploy the contract
    print_step "Deploying contract..."
    CONTRACT_ID=$(soroban contract deploy \
        --wasm "$CONTRACT_DIR/$WASM_FILE" \
        --source "$IDENTITY" \
        --network "$NETWORK" 2>&1) || {
        print_error "Deployment failed"
        if [ "$ENABLE_ROLLBACK" = true ] && [ -n "$backup_file" ]; then
            rollback "$backup_file"
        fi
        exit 1
    }
    
    # Extract contract ID if it's in JSON format
    if echo "$CONTRACT_ID" | jq -e '.contract_id' &> /dev/null; then
        CONTRACT_ID=$(echo "$CONTRACT_ID" | jq -r '.contract_id')
    fi
    
    if [ -z "$CONTRACT_ID" ] || [ "$CONTRACT_ID" = "null" ]; then
        print_error "Deployment failed: empty contract ID"
        if [ "$ENABLE_ROLLBACK" = true ] && [ -n "$backup_file" ]; then
            rollback "$backup_file"
        fi
        exit 1
    fi
    
    print_status "Contract deployed: $CONTRACT_ID"
    
    # Verify deployment
    if ! verify_deployment "$CONTRACT_ID"; then
        print_warning "Verification failed, but continuing..."
    fi
    
    # Save deployment info
    IDENTITY_ADDRESS=$(soroban config identity address "$IDENTITY")
    cat > "$DEPLOYMENT_FILE" << EOF
{
    "contract_name": "$CONTRACT_NAME",
    "contract_id": "$CONTRACT_ID",
    "network": "$NETWORK",
    "deployer": "$IDENTITY",
    "deployer_address": "$IDENTITY_ADDRESS",
    "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "wasm_hash": "$(sha256sum "$CONTRACT_DIR/$WASM_FILE" | cut -d' ' -f1)",
    "commit_sha": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
    "backup_file": "${backup_file:-null}"
}
EOF
    
    print_status "Deployment info saved to: $DEPLOYMENT_FILE"
    print_status "Deployment complete! 🚀"
    
    # Clean up old backups (keep last 5)
    if [ "$ENABLE_ROLLBACK" = true ]; then
        print_step "Cleaning up old backups..."
        ls -t "$DEPLOYMENTS_DIR/${NETWORK}_${CONTRACT_NAME}_backup_"*.json 2>/dev/null | tail -n +6 | xargs rm -f || true
    fi
}

# Trap errors and rollback if enabled
trap 'if [ $? -ne 0 ] && [ "$ENABLE_ROLLBACK" = true ] && [ -n "${backup_file:-}" ] && [ -f "${backup_file:-}" ]; then rollback "$backup_file"; fi' EXIT

# Run main function
main "$@"

