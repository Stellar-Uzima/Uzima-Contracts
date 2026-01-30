#!/bin/bash

# deploy_environment.sh - Deploy contracts to a specific environment
# Usage: ./scripts/deploy_environment.sh <environment> [--contracts contract1,contract2] [--skip-tests]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Parse arguments
ENVIRONMENT=""
CONTRACTS=""
SKIP_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --contracts)
            CONTRACTS="$2"
            shift 2
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        *)
            if [ -z "$ENVIRONMENT" ]; then
                ENVIRONMENT="$1"
            else
                print_error "Unknown argument: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

if [ -z "$ENVIRONMENT" ]; then
    print_error "Usage: $0 <environment> [--contracts contract1,contract2] [--skip-tests]"
    print_error "Environments: local, testnet, futurenet, mainnet"
    exit 1
fi

# Map environment to network
case $ENVIRONMENT in
    "local")
        NETWORK="local"
        IDENTITY="default"
        ;;
    "testnet"|"test")
        NETWORK="testnet"
        IDENTITY="deployer-testnet"
        ;;
    "futurenet"|"staging")
        NETWORK="futurenet"
        IDENTITY="deployer-futurenet"
        ;;
    "mainnet"|"production"|"prod")
        NETWORK="mainnet"
        IDENTITY="deployer-mainnet"
        print_warning "⚠️  Deploying to MAINNET. Are you sure? (Ctrl+C to cancel)"
        sleep 5
        ;;
    *)
        print_error "Unknown environment: $ENVIRONMENT"
        exit 1
        ;;
esac

print_status "Deploying to environment: $ENVIRONMENT (network: $NETWORK)"

# Run tests unless skipped
if [ "$SKIP_TESTS" = false ]; then
    print_step "Running tests..."
    if ! cargo test --all; then
        print_error "Tests failed. Use --skip-tests to deploy anyway."
        exit 1
    fi
    print_status "All tests passed"
fi

# Build all contracts
print_step "Building contracts..."
if ! make build-opt; then
    print_error "Build failed"
    exit 1
fi

# Determine which contracts to deploy
if [ -n "$CONTRACTS" ]; then
    IFS=',' read -ra CONTRACT_ARRAY <<< "$CONTRACTS"
else
    # Deploy all contracts - use mapfile for safe array population
    mapfile -t CONTRACT_ARRAY < <(find contracts -maxdepth 1 -mindepth 1 -type d -exec basename {} \;)
fi

print_status "Deploying contracts: ${CONTRACT_ARRAY[*]}"

# Deploy each contract
FAILED_DEPLOYMENTS=()
SUCCESSFUL_DEPLOYMENTS=()

for contract in "${CONTRACT_ARRAY[@]}"; do
    print_step "Deploying $contract..."
    if ./scripts/deploy_with_rollback.sh "$contract" "$NETWORK" "$IDENTITY"; then
        SUCCESSFUL_DEPLOYMENTS+=("$contract")
        print_status "✓ $contract deployed successfully"
    else
        FAILED_DEPLOYMENTS+=("$contract")
        print_error "✗ $contract deployment failed"
    fi
done

# Summary
echo ""
print_step "Deployment Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
print_status "Environment: $ENVIRONMENT"
print_status "Network: $NETWORK"
echo ""

if [ ${#SUCCESSFUL_DEPLOYMENTS[@]} -gt 0 ]; then
    print_status "Successful deployments:"
    for contract in "${SUCCESSFUL_DEPLOYMENTS[@]}"; do
        echo "  ✓ $contract"
    done
fi

if [ ${#FAILED_DEPLOYMENTS[@]} -gt 0 ]; then
    print_error "Failed deployments:"
    for contract in "${FAILED_DEPLOYMENTS[@]}"; do
        echo "  ✗ $contract"
    done
    exit 1
fi

print_status "All deployments completed successfully! 🚀"

