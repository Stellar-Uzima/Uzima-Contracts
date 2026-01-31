#!/usr/bin/env bash
# Healthcare Integration Deployment Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONTRACTS_DIR="$PROJECT_ROOT/contracts"
DEPLOYMENTS_DIR="$PROJECT_ROOT/deployments"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Healthcare Integration Deployment${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"

# Configuration
NETWORK=${1:-testnet}  # Network: testnet, mainnet, custom
ADMIN_ADDRESS=${2:-}  # Admin account address

if [ -z "$ADMIN_ADDRESS" ]; then
    echo -e "${RED}Error: Admin address required${NC}"
    echo "Usage: $0 <network> <admin_address>"
    exit 1
fi

# Build contracts
build_contracts() {
    echo -e "\n${YELLOW}Building healthcare integration contracts...${NC}"
    
    contracts=("fhir_integration" "emr_integration")
    
    for contract in "${contracts[@]}"; do
        echo -e "${YELLOW}Building $contract...${NC}"
        cd "$CONTRACTS_DIR/$contract"
        cargo build --release
        cd "$PROJECT_ROOT"
        echo -e "${GREEN}✓ $contract built${NC}"
    done
}

# Deploy FHIR integration contract
deploy_fhir() {
    echo -e "\n${YELLOW}Deploying FHIR Integration Contract...${NC}"
    
    cd "$CONTRACTS_DIR/fhir_integration"
    
    # Get WASM path
    WASM_FILE="target/wasm32-unknown-unknown/release/fhir_integration.wasm"
    
    if [ ! -f "$WASM_FILE" ]; then
        echo -e "${RED}WASM file not found: $WASM_FILE${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Uploading WASM to $NETWORK...${NC}"
    
    # Using soroban CLI
    FHIR_CONTRACT=$(soroban contract deploy \
        --wasm "$WASM_FILE" \
        --source-account "$ADMIN_ADDRESS" \
        --network "$NETWORK" \
        --sign-with-key-pair "$ADMIN_ADDRESS" \
        2>&1 | sed -n 's/.*Contract deployed at \([A-Z0-9]*\).*/\1/p' || echo "")
    
    if [ -z "$FHIR_CONTRACT" ]; then
        echo -e "${RED}Failed to deploy FHIR contract${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ FHIR contract deployed: $FHIR_CONTRACT${NC}"
    
    cd "$PROJECT_ROOT"
    echo "$FHIR_CONTRACT"
}

# Deploy EMR integration contract
deploy_emr() {
    local FHIR_CONTRACT=$1
    
    echo -e "\n${YELLOW}Deploying EMR Integration Contract...${NC}"
    
    cd "$CONTRACTS_DIR/emr_integration"
    
    WASM_FILE="target/wasm32-unknown-unknown/release/emr_integration.wasm"
    
    if [ ! -f "$WASM_FILE" ]; then
        echo -e "${RED}WASM file not found: $WASM_FILE${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Uploading WASM to $NETWORK...${NC}"
    
    EMR_CONTRACT=$(soroban contract deploy \
        --wasm "$WASM_FILE" \
        --source-account "$ADMIN_ADDRESS" \
        --network "$NETWORK" \
        --sign-with-key-pair "$ADMIN_ADDRESS" \
        2>&1 | sed -n 's/.*Contract deployed at \([A-Z0-9]*\).*/\1/p' || echo "")
    
    if [ -z "$EMR_CONTRACT" ]; then
        echo -e "${RED}Failed to deploy EMR contract${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ EMR contract deployed: $EMR_CONTRACT${NC}"
    
    cd "$PROJECT_ROOT"
    echo "$EMR_CONTRACT"
}

# Initialize contracts
initialize_contracts() {
    local FHIR_CONTRACT=$1
    local EMR_CONTRACT=$2
    local MEDICAL_RECORDS=$3
    
    echo -e "\n${YELLOW}Initializing healthcare contracts...${NC}"
    
    echo -e "${YELLOW}Initializing FHIR contract...${NC}"
    soroban contract invoke \
        --id "$FHIR_CONTRACT" \
        --source-account "$ADMIN_ADDRESS" \
        --network "$NETWORK" \
        --sign-with-key-pair "$ADMIN_ADDRESS" \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --medical_records_contract "$MEDICAL_RECORDS"
    
    echo -e "${GREEN}✓ FHIR contract initialized${NC}"
    
    echo -e "${YELLOW}Initializing EMR contract...${NC}"
    soroban contract invoke \
        --id "$EMR_CONTRACT" \
        --source-account "$ADMIN_ADDRESS" \
        --network "$NETWORK" \
        --sign-with-key-pair "$ADMIN_ADDRESS" \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --fhir_contract "$FHIR_CONTRACT"
    
    echo -e "${GREEN}✓ EMR contract initialized${NC}"
}

# Save deployment info
save_deployment_info() {
    local FHIR_CONTRACT=$1
    local EMR_CONTRACT=$2
    local DEPLOYMENT_FILE="$DEPLOYMENTS_DIR/healthcare_integration_${NETWORK}_$(date +%s).json"
    
    mkdir -p "$DEPLOYMENTS_DIR"
    
    cat > "$DEPLOYMENT_FILE" << EOF
{
  "network": "$NETWORK",
  "deployment_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "admin_address": "$ADMIN_ADDRESS",
  "contracts": {
    "fhir_integration": "$FHIR_CONTRACT",
    "emr_integration": "$EMR_CONTRACT"
  },
  "deployment_status": "completed",
  "notes": "Healthcare integration contracts deployed with FHIR HL7 standard compliance and EMR/EHR interoperability support"
}
EOF

    echo -e "\n${GREEN}Deployment info saved: $DEPLOYMENT_FILE${NC}"
    cat "$DEPLOYMENT_FILE"
}

# Main execution
main() {
    echo -e "${YELLOW}Network: $NETWORK${NC}"
    echo -e "${YELLOW}Admin: $ADMIN_ADDRESS${NC}"
    
    # Build
    build_contracts
    
    # Deploy
    FHIR_CONTRACT=$(deploy_fhir)
    EMR_CONTRACT=$(deploy_emr "$FHIR_CONTRACT")
    
    # Get medical records contract from previous deployment
    # In production, this would be read from a configuration file
    MEDICAL_RECORDS=$ADMIN_ADDRESS  # Placeholder
    
    # Initialize
    initialize_contracts "$FHIR_CONTRACT" "$EMR_CONTRACT" "$MEDICAL_RECORDS"
    
    # Save info
    save_deployment_info "$FHIR_CONTRACT" "$EMR_CONTRACT"
    
    echo -e "\n${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}Healthcare Integration Deployment Complete!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "\n${BLUE}Deployment Summary:${NC}"
    echo -e "  FHIR Contract: ${BLUE}$FHIR_CONTRACT${NC}"
    echo -e "  EMR Contract: ${BLUE}$EMR_CONTRACT${NC}"
    echo -e "  Network: ${BLUE}$NETWORK${NC}"
}

# Execute
main "$@"
