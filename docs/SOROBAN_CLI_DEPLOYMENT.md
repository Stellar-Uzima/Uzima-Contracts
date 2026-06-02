# Soroban CLI Deployment Guide

This comprehensive guide covers all aspects of deploying Soroban smart contracts using the Soroban CLI across different networks.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Local Network Setup](#local-network-setup)
3. [Testnet Deployment](#testnet-deployment)
4. [Mainnet Deployment](#mainnet-deployment)
5. [Contract Initialization](#contract-initialization)
6. [Troubleshooting](#troubleshooting)
7. [Best Practices](#best-practices)

## Prerequisites

### Required Tools

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Soroban CLI
cargo install --locked soroban-cli

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Verify installation
soroban --version
```

### Environment Setup

```bash
# Create workspace directory
mkdir soroban-workspace
cd soroban-workspace

# Initialize Soroban configuration
soroban config network add --global local \
  --rpc-url http://localhost:8000/soroban/rpc \
  --network-passphrase "Standalone Network ; February 2017"

soroban config network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

soroban config network add --global mainnet \
  --rpc-url https://soroban-rpc.mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

## Local Network Setup

### Starting Local Network

```bash
# Start Stellar Quickstart with Soroban support
docker run --rm -it \
  -p 8000:8000 \
  stellar/quickstart:latest \
  --standalone \
  --enable-soroban-rpc

# Or use Stellar Soroban RPC container
docker run --rm -d \
  -p 8000:8000 \
  --name soroban-rpc \
  stellar/soroban-rpc:latest \
  --port 8000
```

### Creating Local Identities

```bash
# Generate development identity
soroban config identity generate dev-admin

# Generate multiple identities for testing
soroban config identity generate alice
soroban config identity generate bob
soroban config identity generate charlie

# List all identities
soroban config identity list

# Show identity details
soroban config identity show dev-admin
```

### Funding Local Accounts

```bash
# Fund accounts on local network (no real cost)
soroban keys fund dev-admin --network local
soroban keys fund alice --network local
soroban keys fund bob --network local
```

## Testnet Deployment

### Setting Up Testnet Identity

```bash
# Generate testnet identity
soroban config identity generate testnet-deployer

# Fund account using friendbot
soroban keys fund testnet-deployer --network testnet

# Verify funding
soroban account info --identity testnet-deployer --network testnet
```

### Building Contract

```bash
# Navigate to contract directory
cd path/to/your/contract

# Build for release
cargo build --target wasm32-unknown-unknown --release

# Optimize WASM file (optional but recommended)
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --output target/wasm32-unknown-unknown/release/your_contract_optimized.wasm
```

### Deploying to Testnet

```bash
# Deploy contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet-deployer \
  --network testnet

# Deploy with specific salt for deterministic address
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet-deployer \
  --network testnet \
  --salt "your-custom-salt"

# Deploy and save contract ID to file
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet-deployer \
  --network testnet \
  > contract_id.txt

CONTRACT_ID=$(cat contract_id.txt)
echo "Contract deployed with ID: $CONTRACT_ID"
```

### Verifying Testnet Deployment

```bash
# Check contract exists
soroban contract info --id $CONTRACT_ID --network testnet

# Check contract WASM hash
soroban contract info --id $CONTRACT_ID --network testnet | grep wasm_hash

# Verify on Stellar Explorer
echo "https://stellar.expert/explorer/testnet/contract/$CONTRACT_ID"
```

## Mainnet Deployment

### Security Preparations

```bash
# Create dedicated mainnet identity
soroban config identity generate mainnet-deployer

# NEVER fund with friendbot - use real funds
# Transfer funds to mainnet-deployer address
soroban config identity address mainnet-deployer

# Verify account balance before deployment
soroban account info --identity mainnet-deployer --network mainnet
```

### Pre-Deployment Checklist

```bash
# 1. Validate contract build
cargo check --target wasm32-unknown-unknown --release
cargo build --target wasm32-unknown-unknown --release

# 2. Test on local network first
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source dev-admin \
  --network local

# 3. Test on testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet-deployer \
  --network testnet

# 4. Verify mainnet connectivity
soroban account info --identity mainnet-deployer --network mainnet
```

### Mainnet Deployment

```bash
# Deploy to mainnet (requires confirmation)
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source mainnet-deployer \
  --network mainnet

# Deploy with dry-run simulation first
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source mainnet-deployer \
  --network mainnet \
  --dry-run

# Deploy with maximum fee for reliability
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source mainnet-deployer \
  --network mainnet \
  --fee 1000
```

### Post-Deployment Verification

```bash
# Save contract information
CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source mainnet-deployer \
  --network mainnet)

# Verify deployment
soroban contract info --id $CONTRACT_ID --network mainnet

# Save deployment details
cat > deployment_info.json <<EOF
{
  "contract_id": "$CONTRACT_ID",
  "network": "mainnet",
  "deployer": "$(soroban config identity address mainnet-deployer)",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "wasm_hash": "$(sha256sum target/wasm32-unknown-unknown/release/your_contract.wasm)"
}
EOF
```

## Contract Initialization

### Initializing with Constructor

```bash
# Initialize contract with constructor arguments
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  --function initialize \
  --arg-admin "$(soroban config identity address deployer)" \
  --arg-name "My Contract" \
  --arg-version "1.0.0"

# Initialize with complex arguments
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  --function initialize \
  --arg-config '{"max_supply": "1000000", "decimals": 18}' \
  --arg-allowed_tokens '["TOKEN1", "TOKEN2", "TOKEN3"]'
```

### Setting Up Access Control

```bash
# Grant admin permissions
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  --function grant_admin \
  --arg-address "$(soroban config identity address alice)"

# Set up role-based access
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  --function set_role \
  --arg-user "$(soroban config identity address bob)" \
  --arg-role "operator"
```

### Initial Configuration

```bash
# Configure contract parameters
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  --function configure \
  --arg-fee_rate "1000" \
  --arg-minimum_balance "10000000"

# Enable features
soroban contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  --function enable_feature \
  --arg-feature "advanced_features"
```

## Troubleshooting

### Common Deployment Issues

#### Network Connectivity

```bash
# Test network connectivity
curl -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
  https://soroban-testnet.stellar.org:443

# Check network status
soroban config network list
```

#### Identity Issues

```bash
# List identities
soroban config identity list

# Regenerate identity if needed
soroban config identity generate new-identity

# Check identity address
soroban config identity address testnet-deployer
```

#### Build Issues

```bash
# Clean build
cargo clean
cargo build --target wasm32-unknown-unknown --release

# Check WASM file
file target/wasm32-unknown-unknown/release/your_contract.wasm

# Optimize WASM
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --output target/wasm32-unknown-unknown/release/your_contract_optimized.wasm
```

#### Funding Issues

```bash
# Check account balance
soroban account info --identity testnet-deployer --network testnet

# Fund account (testnet only)
soroban keys fund testnet-deployer --network testnet

# Check friendbot status
curl https://friendbot.stellar.org?addr=$(soroban config identity address testnet-deployer)
```

### Error Messages and Solutions

#### "Transaction Failed (insufficient fee)"

```bash
# Increase fee
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet-deployer \
  --network testnet \
  --fee 500
```

#### "Contract already exists"

```bash
# Use different salt
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/your_contract.wasm \
  --source testnet-deployer \
  --network testnet \
  --salt "$(date +%s)"
```

#### "Network not found"

```bash
# Add network configuration
soroban config network add --global custom \
  --rpc-url https://custom-rpc.example.com \
  --network-passphrase "Custom Network"
```

## Best Practices

### Security Practices

1. **Use separate identities for different environments**
   ```bash
   soroban config identity generate mainnet-admin
   soroban config identity generate testnet-admin
   soroban config identity generate dev-admin
   ```

2. **Never share private keys**
   ```bash
   # Keep identities secure
   chmod 600 ~/.soroban/identities/*
   ```

3. **Use deterministic deployment addresses when possible**
   ```bash
   # Use fixed salt for predictable addresses
   soroban contract deploy --salt "production-salt-v1"
   ```

4. **Always test on testnet before mainnet**
   ```bash
   # Full testing workflow
   ./scripts/test_contract.sh local
   ./scripts/test_contract.sh testnet
   ./scripts/deploy_contract.sh mainnet
   ```

### Performance Optimization

1. **Optimize WASM files**
   ```bash
   soroban contract optimize \
     --wasm contract.wasm \
     --output contract_optimized.wasm
   ```

2. **Use appropriate fee levels**
   ```bash
   # Standard deployment
   soroban contract deploy --fee 100
   
   # High-priority deployment
   soroban contract deploy --fee 1000
   ```

3. **Batch operations when possible**
   ```bash
   # Deploy multiple contracts in sequence
   for contract in contract1 contract2 contract3; do
     soroban contract deploy --wasm $contract.wasm --source deployer --network testnet
   done
   ```

### Monitoring and Maintenance

1. **Save deployment information**
   ```bash
   # Create deployment log
   echo "$(date): Deployed $CONTRACT_ID to $NETWORK" >> deployments.log
   ```

2. **Monitor contract status**
   ```bash
   # Regular health checks
   soroban contract info --id $CONTRACT_ID --network mainnet
   ```

3. **Backup configurations**
   ```bash
   # Backup Soroban configuration
   cp -r ~/.soroban backup/soroban-$(date +%Y%m%d)
   ```

### Development Workflow

1. **Local Development**
   ```bash
   # Start local network
   docker run --rm -d -p 8000:8000 stellar/soroban-rpc:latest
   
   # Deploy and test locally
   soroban contract deploy --wasm contract.wasm --source dev-admin --network local
   ```

2. **Testnet Testing**
   ```bash
   # Deploy to testnet
   soroban contract deploy --wasm contract.wasm --source testnet-admin --network testnet
   
   # Run integration tests
   ./scripts/integration_test.sh $CONTRACT_ID testnet
   ```

3. **Production Deployment**
   ```bash
   # Final deployment with verification
   soroban contract deploy --wasm contract_optimized.wasm --source mainnet-admin --network mainnet
   soroban contract info --id $CONTRACT_ID --network mainnet
   ```

---

This guide provides a comprehensive foundation for deploying Soroban contracts. For more advanced topics, see the [Contract Interaction Guide](./SOROBAN_CLI_INTERACTION.md) and [Development Guide](./SOROBAN_CLI_DEVELOPMENT.md).
