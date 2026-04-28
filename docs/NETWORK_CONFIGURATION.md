# Comprehensive Soroban Network Configuration Management

This document describes the enhanced network configuration management system implemented for Uzima-Contracts, providing robust, safe, and flexible network handling for Soroban smart contract deployment.

## Overview

The network configuration management system addresses the issue of manual network configuration being prone to errors across environments. It provides:

- **Environment-specific configurations** with validation checks
- **Auto-detection** of available networks
- **Fallback mechanisms** for network unavailability
- **Safety features** including mainnet confirmation prompts, transaction simulation, and dry-run mode
- **Network verification** and connectivity testing

## Architecture

### Core Components

1. **Network Configuration File** (`config/networks.toml`)
   - Centralized network definitions
   - Environment-specific settings
   - Safety configurations

2. **Network Manager Script** (`scripts/network_manager.sh`)
   - Network validation and configuration
   - Auto-detection and fallback
   - Safety checks

3. **Enhanced Deployment Script** (`scripts/deploy_enhanced.sh`)
   - Safe deployment with network management
   - Simulation and dry-run modes
   - Comprehensive error handling

4. **Validation Script** (`scripts/validate_network_config.sh`)
   - Configuration validation
   - Connectivity testing
   - Deployment readiness checks

## Network Configuration

### Supported Networks

The system supports four primary networks:

| Network | Environment | RPC URL | Safety Level | Funding Required |
|---------|-------------|---------|--------------|------------------|
| `local` | Development | `http://localhost:8000/soroban/rpc` | Low | No |
| `testnet` | Testing | `https://soroban-testnet.stellar.org` | Medium | Yes |
| `futurenet` | Testing | `https://rpc-futurenet.stellar.org` | Medium | Yes |
| `mainnet` | Production | `https://soroban-mainnet.stellar.org` | High | No |

### Configuration Structure

```toml
[networks.<network_name>]
name = "Human-readable name"
description = "Network description"
rpc-url = "RPC endpoint URL"
network-passphrase = "Stellar network passphrase"
horizon-url = "Horizon server URL"
friendbot-url = "Faucet URL (if available)"
environment = "development|testing|production"
requires-funding = true|false
gas-configuration = { max-instructions = 100_000_000, tx-resource-fee = 100 }
safety-level = "low|medium|high"
confirmation-required = true|false
```

### Network Groups

Networks are organized into groups for easier management:

- **Development**: `local`
- **Testing**: `testnet`, `futurenet`
- **Production**: `mainnet`

## Usage

### Basic Network Management

#### Configure a Network
```bash
# Configure a specific network
./scripts/network_manager.sh configure testnet

# Configure all networks
./scripts/network_manager.sh configure-all

# Force reconfiguration
FORCE=true ./scripts/network_manager.sh configure testnet
```

#### Check Network Status
```bash
# Show all network status
./scripts/network_manager.sh status

# Show specific network status
./scripts/network_manager.sh status testnet
```

#### Validate Network Configuration
```bash
# Validate specific network
./scripts/network_manager.sh validate testnet

# Auto-detect available networks
./scripts/network_manager.sh detect
```

### Enhanced Deployment

#### Basic Deployment
```bash
# Deploy to testnet
./scripts/deploy_enhanced.sh medical_records testnet

# Deploy with custom identity
./scripts/deploy_enhanced.sh medical_records testnet --identity alice
```

#### Safe Deployment Features
```bash
# Dry-run mode (no actual deployment)
./scripts/deploy_enhanced.sh medical_records mainnet --dry-run

# Simulation mode (test transaction without execution)
./scripts/deploy_enhanced.sh medical_records mainnet --simulation

# Auto-fallback if network unavailable
./scripts/deploy_enhanced.sh medical_records testnet --auto-fallback
```

#### Advanced Options
```bash
# Skip building (use existing WASM)
./scripts/deploy_enhanced.sh medical_records testnet --skip-build

# Force deployment (bypass some checks)
./scripts/deploy_enhanced.sh medical_records mainnet --force

# Debug mode with verbose output
./scripts/deploy_enhanced.sh medical_records testnet --debug
```

### Configuration Validation

#### Comprehensive Validation
```bash
# Validate all configurations
./scripts/validate_network_config.sh

# Validate specific network
./scripts/validate_network_config.sh --network testnet

# Validate deployment prerequisites
./scripts/validate_network_config.sh --contract medical_records --network testnet
```

## Safety Features

### Mainnet Protection

The system includes multiple layers of protection for mainnet operations:

1. **Confirmation Required**: Mainnet deployments require explicit confirmation
2. **Dry-run Mode**: Test deployments without executing transactions
3. **Simulation Mode**: Simulate transactions to check for errors
4. **Safety Checks**: Validate configurations before deployment

### Example Mainnet Deployment
```bash
# Safe mainnet deployment with all safety features
./scripts/deploy_enhanced.sh medical_records mainnet --dry-run --simulation

# After validation, proceed with actual deployment
./scripts/deploy_enhanced.sh medical_records mainnet
# Type 'CONFIRM' when prompted to proceed
```

### Environment Detection

The system automatically detects the current environment:

- **CI Environment**: Detected via `CI=true` environment variable
- **Production**: Detected via `NODE_ENV=production` in `.env` file
- **Testing**: Detected via `NODE_ENV=test` in `.env` file
- **Development**: Default environment

## Auto-Detection and Fallback

### Network Auto-Detection
```bash
# Automatically detect available networks
./scripts/network_manager.sh detect
```

The system tests connectivity to each network and reports available options.

### Fallback Mechanism
```bash
# Use fallback if preferred network unavailable
./scripts/deploy_enhanced.sh medical_records testnet --auto-fallback
```

Fallback order:
1. Try the requested network
2. Fall back to `local` if available
3. Fall back to `testnet` if available
4. Fail if no networks are available

## Configuration Validation

### Validation Checks

The system performs comprehensive validation:

1. **File Validation**: Check if configuration files exist and are readable
2. **Syntax Validation**: Validate TOML syntax
3. **Network Completeness**: Ensure all required fields are present
4. **Connectivity Testing**: Test network connectivity
5. **Soroban Configuration**: Verify Soroban CLI configuration
6. **Identity Validation**: Check identity configuration
7. **Deployment Prerequisites**: Validate contract build requirements

### Validation Report
```
========================================
VALIDATION REPORT
========================================
Total Tests: 15
Passed: 15
Failed: 0
Success Rate: 100%

🎉 All tests passed!
```

## Environment Variables

### Optional Variables

- `SOROBAN_RPC_URL`: Override RPC URL for current network
- `SOROBAN_NETWORK_PASSPHRASE`: Override network passphrase
- `DEBUG`: Enable debug output (`true`/`false`)
- `DRY_RUN`: Enable dry-run mode (`true`/`false`)
- `SIMULATION`: Enable simulation mode (`true`/`false`)
- `FORCE`: Force operations (`true`/`false`)

### Example Usage
```bash
# Enable debug output
DEBUG=true ./scripts/deploy_enhanced.sh medical_records testnet

# Dry-run mode
DRY_RUN=true ./scripts/deploy_enhanced.sh medical_records mainnet

# Override RPC URL
SOROBAN_RPC_URL=http://localhost:8001 ./scripts/deploy_enhanced.sh medical_records local
```

## Troubleshooting

### Common Issues

#### Network Not Reachable
```bash
# Check network status
./scripts/network_manager.sh status

# Test connectivity
./scripts/network_manager.sh validate testnet

# Try fallback
./scripts/deploy_enhanced.sh medical_records testnet --auto-fallback
```

#### Configuration Errors
```bash
# Validate configuration
./scripts/validate_network_config.sh

# Check TOML syntax
python3 -c "import tomllib; print(tomllib.load(open('config/networks.toml', 'rb')))"
```

#### Identity Issues
```bash
# Check identity status
soroban config identity show

# Generate new identity
soroban config identity generate my-identity
```

#### Build Failures
```bash
# Check contract build
cargo check -p medical_records --target wasm32-unknown-unknown

# Clean and rebuild
cargo clean -p medical_records
cargo build -p medical_records --target wasm32-unknown-unknown --release
```

### Debug Mode

Enable debug mode for detailed output:
```bash
DEBUG=true ./scripts/deploy_enhanced.sh medical_records testnet --debug
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
- name: Validate Network Configuration
  run: ./scripts/validate_network_config.sh --network testnet

- name: Deploy to Testnet
  run: ./scripts/deploy_enhanced.sh medical_records testnet --auto-fallback
  env:
    DRY_RUN: false
    SIMULATION: true
```

### Environment-Specific Deployments

```bash
# Development
./scripts/deploy_enhanced.sh medical_records local

# Testing
./scripts/deploy_enhanced.sh medical_records testnet --simulation

# Production (with safety checks)
./scripts/deploy_enhanced.sh medical_records mainnet --dry-run
```

## Best Practices

1. **Always Use Enhanced Scripts**: Use `deploy_enhanced.sh` instead of manual deployment
2. **Test Before Production**: Always use simulation mode before mainnet deployment
3. **Validate Configurations**: Run validation scripts before deployment
4. **Use Auto-Fallback**: Enable auto-fallback for better reliability
5. **Monitor Network Status**: Check network status before critical deployments
6. **Keep Configurations Updated**: Regularly update network configurations
7. **Use Environment Variables**: Leverage environment variables for CI/CD integration

## Migration from Old System

### Before (Old System)
```bash
# Manual network configuration
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Manual deployment
soroban contract deploy --wasm contract.wasm --source alice --network testnet
```

### After (New System)
```bash
# Automatic network configuration and deployment
./scripts/deploy_enhanced.sh medical_records testnet --identity alice
```

### Benefits of Migration

1. **Reduced Errors**: Automated configuration eliminates manual errors
2. **Better Safety**: Built-in safety checks prevent accidental mainnet deployments
3. **Improved Reliability**: Auto-detection and fallback mechanisms
4. **Enhanced Debugging**: Comprehensive logging and error reporting
5. **CI/CD Integration**: Better support for automated deployments

## Contributing

When contributing to the network configuration system:

1. **Test All Networks**: Ensure changes work with all supported networks
2. **Validate Configurations**: Run validation scripts after changes
3. **Update Documentation**: Keep documentation up to date
4. **Test Safety Features**: Verify safety features work correctly
5. **Check Backward Compatibility**: Ensure existing workflows continue to work

## Support

For issues or questions about the network configuration system:

1. Check the validation report for configuration issues
2. Review debug output for detailed error information
3. Consult the troubleshooting section
4. Check GitHub issues for known problems
5. Create new issues with detailed error reports and configuration details
