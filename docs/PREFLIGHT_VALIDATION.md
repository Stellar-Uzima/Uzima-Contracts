# Contract Preflight Validation

This document describes the preflight validation system for Uzima contract deployments.

## Overview

The preflight check validates deployment readiness before executing deployment scripts. It checks network configuration, contract dependencies, build artifacts, environment settings, and resource budgets.

## Script

### `scripts/preflight_check.sh`

Validates deployment readiness for contracts and networks.

```bash
# Validate for testnet
./scripts/preflight_check.sh --network testnet

# Validate specific contract for mainnet
./scripts/preflight_check.sh --network mainnet --contract medical_records

# Full validation with environment and identity
./scripts/preflight_check.sh --network testnet --environment staging --identity alice
```

**Options:**
- `--network <network>` - Target network (required: testnet, mainnet, futurenet, local)
- `--contract <name>` - Specific contract to validate
- `--identity <name>` - Identity to use (default: default)
- `--environment <env>` - Environment configuration

## Validation Checks

### Network Configuration
- Verifies `config/networks.toml` exists and is valid TOML
- Validates the target network has required fields (rpc-url, network-passphrase)
- Tests network connectivity

### Contract Validation
- Verifies contract directory exists
- Validates `Cargo.toml` is present
- Checks all path dependencies are resolvable
- Verifies WASM build artifact exists and is non-empty

### Environment Configuration
- Validates environment-specific configuration file
- Checks identity configuration via Soroban CLI

### Resource Budgets
- Checks if resource budgets are defined for the contract

### Signing Configuration
- Validates `deployments/signing-config.json` is valid

## Integration with Deploy Scripts

```bash
./scripts/preflight_check.sh --network testnet --contract medical_records
if [ $? -eq 0 ]; then
    ./scripts/deploy.sh medical_records testnet
fi
```

## Output

Reports are saved to `reports/preflight_report.json`.
