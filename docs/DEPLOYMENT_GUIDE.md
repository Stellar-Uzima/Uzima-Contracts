# Uzima-Contracts Deployment Guide

This comprehensive guide covers deploying Uzima-Contracts smart contracts across different environments using the enhanced network configuration management system.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Environment Setup](#environment-setup)
4. [Deployment Workflows](#deployment-workflows)
5. [Network-Specific Deployments](#network-specific-deployments)
6. [Safety and Best Practices](#safety-and-best-practices)
7. [Troubleshooting](#troubleshooting)
8. [CI/CD Integration](#cicd-integration)

## Prerequisites

### Required Tools

- **Rust** (latest stable version)
- **Soroban CLI** (v21.7.7 or later)
- **Docker** (for local network)
- **Git** (for version control)

### Installation

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

### Project Setup

```bash
# Clone the repository
git clone https://github.com/Stellar-Uzima/Uzima-Contracts.git
cd Uzima-Contracts

# Install dependencies
make install-deps

# Validate setup
./scripts/validate_network_config.sh
```

## Quick Start

### Local Development Deployment

```bash
# Start local Stellar network
make start-local

# Deploy contract to local network
./scripts/deploy_enhanced.sh medical_records local

# Stop local network when done
make stop-local
```

### Testnet Deployment

```bash
# Deploy to testnet with simulation
./scripts/deploy_enhanced.sh medical_records testnet --simulation

# Deploy to testnet (real deployment)
./scripts/deploy_enhanced.sh medical_records testnet
```

## Environment Setup

### Local Development

1. **Start Local Network**
   ```bash
   make start-local
   ```

2. **Configure Local Identity**
   ```bash
   soroban config identity generate dev-local
   ```

3. **Deploy Contracts**
   ```bash
   ./scripts/deploy_enhanced.sh medical_records local --identity dev-local
   ```

### Testnet Environment

1. **Configure Testnet Identity**
   ```bash
   soroban config identity generate test-user
   ```

2. **Fund Account**
   ```bash
   soroban config identity fund test-user --network testnet
   ```

3. **Deploy Contracts**
   ```bash
   ./scripts/deploy_enhanced.sh medical_records testnet --identity test-user
   ```

### Production Environment

1. **Configure Production Identity**
   ```bash
   soroban config identity generate prod-user
   ```

2. **Validate Configuration**
   ```bash
   ./scripts/validate_network_config.sh --network mainnet
   ```

3. **Deploy with Safety Checks**
   ```bash
   ./scripts/deploy_enhanced.sh medical_records mainnet --identity prod-user --dry-run
   ./scripts/deploy_enhanced.sh medical_records mainnet --identity prod-user
   ```

## Deployment Workflows

### Development Workflow

```bash
# 1. Clean and build
make clean
make build-opt

# 2. Validate configuration
./scripts/validate_network_config.sh --network local

# 3. Deploy to local
./scripts/deploy_enhanced.sh medical_records local

# 4. Test contract interaction
./scripts/interact.sh medical_records local
```

### Testing Workflow

```bash
# 1. Validate testnet configuration
./scripts/validate_network_config.sh --network testnet

# 2. Simulate deployment
./scripts/deploy_enhanced.sh medical_records testnet --simulation

# 3. Deploy to testnet
./scripts/deploy_enhanced.sh medical_records testnet --identity test-user

# 4. Run integration tests
make test-integration
```

### Production Workflow

```bash
# 1. Comprehensive validation
./scripts/validate_network_config.sh --network mainnet --contract medical_records

# 2. Dry-run deployment
./scripts/deploy_enhanced.sh medical_records mainnet --identity prod-user --dry-run

# 3. Final deployment (with confirmation)
./scripts/deploy_enhanced.sh medical_records mainnet --identity prod-user

# 4. Verify deployment
./scripts/verify_deployment.sh medical_records mainnet
```

## Network-Specific Deployments

### Local Network

**Characteristics:**
- No real funds required
- Fast transactions
- Full control
- Ideal for development

**Deployment Commands:**
```bash
# Basic deployment
./scripts/deploy_enhanced.sh medical_records local

# With custom identity
./scripts/deploy_enhanced.sh medical_records local --identity dev

# Skip build (use existing WASM)
./scripts/deploy_enhanced.sh medical_records local --skip-build
```

### Testnet Network

**Characteristics:**
- Test funds available via friendbot
- Public test environment
- Slower than local
- Ideal for testing

**Deployment Commands:**
```bash
# With auto-funding
./scripts/deploy_enhanced.sh medical_records testnet

# With specific identity
./scripts/deploy_enhanced.sh medical_records testnet --identity alice

# With simulation first
./scripts/deploy_enhanced.sh medical_records testnet --simulation
```

### Futurenet Network

**Characteristics:**
- Experimental features
- Test funds available
- Unstable environment
- For testing new features

**Deployment Commands:**
```bash
# Deploy to futurenet
./scripts/deploy_enhanced.sh medical_records futurenet

# With fallback
./scripts/deploy_enhanced.sh medical_records futurenet --auto-fallback
```

### Mainnet Network

**Characteristics:**
- Real funds required
- Production environment
- Highest security
- Irreversible transactions

**Deployment Commands:**
```bash
# Safe deployment with dry-run
./scripts/deploy_enhanced.sh medical_records mainnet --dry-run --simulation

# Production deployment
./scripts/deploy_enhanced.sh medical_records mainnet --identity prod-user

# Forced deployment (bypass confirmation)
./scripts/deploy_enhanced.sh medical_records mainnet --force
```

## Safety and Best Practices

### Pre-Deployment Checklist

- [ ] Validate network configuration
- [ ] Test connectivity
- [ ] Verify identity setup
- [ ] Check contract build
- [ ] Run simulation (for mainnet)
- [ ] Confirm funding (for test networks)
- [ ] Review safety settings

### Safety Features

1. **Mainnet Confirmation**
   ```bash
   # Always requires explicit confirmation for mainnet
   ./scripts/deploy_enhanced.sh medical_records mainnet
   # Type 'CONFIRM' to proceed
   ```

2. **Dry-Run Mode**
   ```bash
   # Test without actual deployment
   ./scripts/deploy_enhanced.sh medical_records mainnet --dry-run
   ```

3. **Simulation Mode**
   ```bash
   # Simulate transaction execution
   ./scripts/deploy_enhanced.sh medical_records mainnet --simulation
   ```

4. **Auto-Fallback**
   ```bash
   # Use alternative network if preferred is unavailable
   ./scripts/deploy_enhanced.sh medical_records testnet --auto-fallback
   ```

### Best Practices

1. **Always Test First**
   ```bash
   # Test on local first
   ./scripts/deploy_enhanced.sh medical_records local
   
   # Then test on testnet
   ./scripts/deploy_enhanced.sh medical_records testnet --simulation
   ./scripts/deploy_enhanced.sh medical_records testnet
   
   # Finally deploy to mainnet
   ./scripts/deploy_enhanced.sh medical_records mainnet --dry-run
   ./scripts/deploy_enhanced.sh medical_records mainnet
   ```

2. **Use Descriptive Identities**
   ```bash
   # Use meaningful identity names
   soroban config identity generate medical-records-prod
   soroban config identity generate medical-records-test
   ```

3. **Save Deployment Information**
   ```bash
   # Deployment info is automatically saved to deployments/
   ls deployments/
   # medical_records_mainnet_2023-12-01.json
   ```

4. **Monitor Deployments**
   ```bash
   # Check deployment status
   ./scripts/monitor_deployments.sh
   ```

## Troubleshooting

### Common Issues

#### Network Connectivity Issues

**Problem:** Network not reachable
```bash
# Solution: Check network status
./scripts/network_manager.sh status

# Solution: Test connectivity
./scripts/network_manager.sh validate testnet

# Solution: Use fallback
./scripts/deploy_enhanced.sh medical_records testnet --auto-fallback
```

#### Identity Issues

**Problem:** Identity not found
```bash
# Solution: Create identity
soroban config identity generate my-identity

# Solution: List identities
soroban config identity list

# Solution: Check specific identity
soroban config identity show my-identity
```

#### Build Failures

**Problem:** Contract build fails
```bash
# Solution: Check contract
cargo check -p medical_records --target wasm32-unknown-unknown

# Solution: Clean and rebuild
cargo clean -p medical_records
cargo build -p medical_records --target wasm32-unknown-unknown --release

# Solution: Check dependencies
cargo update -p medical_records
```

#### Funding Issues

**Problem:** Account not funded
```bash
# Solution: Fund account manually
soroban config identity fund my-identity --network testnet

# Solution: Check balance
soroban config identity address my-identity
# Check balance on Stellar explorer
```

#### Deployment Failures

**Problem:** Deployment fails
```bash
# Solution: Check deployment prerequisites
./scripts/validate_network_config.sh --contract medical_records --network testnet

# Solution: Use debug mode
DEBUG=true ./scripts/deploy_enhanced.sh medical_records testnet --debug

# Solution: Check logs
tail -f ~/.soroban/soroban-rpc.log
```

### Debug Mode

Enable debug mode for detailed output:
```bash
# Enable debug output
DEBUG=true ./scripts/deploy_enhanced.sh medical_records testnet --debug

# Check all debug information
DEBUG=true ./scripts/deploy_enhanced.sh medical_records testnet --debug 2>&1 | tee deployment.log
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Deploy to Testnet

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
          
      - name: Install Soroban CLI
        run: cargo install --locked soroban-cli
        
      - name: Validate Configuration
        run: ./scripts/validate_network_config.sh --network testnet
        
      - name: Deploy Contract
        run: ./scripts/deploy_enhanced.sh medical_records testnet --simulation
        env:
          DRY_RUN: ${{ github.event_name == 'pull_request' }}
```

### Environment Variables for CI/CD

```yaml
env:
  SOROBAN_NETWORK: testnet
  SOROBAN_IDENTITY: ci-user
  DRY_RUN: true
  SIMULATION: true
  DEBUG: true
```

### Multi-Environment Deployment

```yaml
strategy:
  matrix:
    network: [local, testnet]
    include:
      - network: local
        dry_run: false
        simulation: false
      - network: testnet
        dry_run: false
        simulation: true
```

## Advanced Usage

### Batch Deployments

```bash
# Deploy multiple contracts
for contract in medical_records identity_registry payment_router; do
    ./scripts/deploy_enhanced.sh $contract testnet --simulation
done

# Deploy with different identities
./scripts/deploy_enhanced.sh medical_records testnet --identity medical-admin
./scripts/deploy_enhanced.sh identity_registry testnet --identity identity-admin
```

### Custom Network Configuration

```bash
# Add custom network to config/networks.toml
[networks.custom]
name = "Custom Network"
rpc-url = "https://custom-rpc.example.com"
network-passphrase = "Custom Network Passphrase"
environment = "testing"
requires-funding = false
safety-level = "medium"
confirmation-required = false

# Deploy to custom network
./scripts/deploy_enhanced.sh medical_records custom
```

### Contract Upgrades

```bash
# Deploy upgraded contract
./scripts/deploy_enhanced.sh medical_records_v2 mainnet --dry-run

# Verify upgrade
./scripts/verify_deployment.sh medical_records_v2 mainnet

# Migrate data (if needed)
./scripts/migrate_contract.sh medical_records medical_records_v2 mainnet
```

## Monitoring and Maintenance

### Deployment Monitoring

```bash
# Monitor all deployments
./scripts/monitor_deployments.sh

# Check specific contract
./scripts/monitor_deployments.sh --contract medical_records

# Check specific network
./scripts/monitor_deployments.sh --network mainnet
```

### Regular Maintenance

```bash
# Update network configurations
./scripts/network_manager.sh configure-all --force

# Validate all configurations
./scripts/validate_network_config.sh

# Clean up old deployments
./scripts/cleanup_deployments.sh --older-than 30d
```

## Support and Resources

### Documentation

- [Network Configuration Guide](./NETWORK_CONFIGURATION.md)
- [API Documentation](./API_REFERENCE.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)

### Community

- GitHub Issues: Report bugs and request features
- Discord Server: Community support and discussions
- Stellar Discord: Soroban-specific support

### Tools and Utilities

- Soroban CLI Documentation
- Stellar Explorer
- Stellar Laboratory

---

For additional help or questions, please refer to the project documentation or create an issue on GitHub.
