# Secrets & Identity Management

Per-network secrets and identity management tooling with safe defaults for Stellar Soroban deployments.

## Overview

The `manage_secrets.sh` script provides a structured approach to managing per-network secrets and cryptographic identities. It follows the existing script patterns in the repository (`check_error_codes.sh`, `deploy.sh`, `logger.sh`).

## Quick Start

```bash
# Initialize secrets for a network
./scripts/manage_secrets.sh init testnet

# List configured secrets
./scripts/manage_secrets.sh list testnet

# Generate a new identity
./scripts/manage_secrets.sh generate testnet custom-deployer

# Rotate an existing identity
./scripts/manage_secrets.sh rotate testnet admin

# Validate configuration
./scripts/manage_secrets.sh validate testnet

# Export env-safe configuration
./scripts/manage_secrets.sh export testnet
```

## Directory Structure

```
.secrets/
├── testnet/
│   ├── config.toml           # Network-specific secrets config
│   ├── .gitignore            # Prevents accidental secret commits
│   └── identities/
│       ├── admin/
│       │   ├── address.txt
│       │   └── metadata.toml
│       ├── deployer/
│       │   ├── address.txt
│       │   └── metadata.toml
│       └── approver/
│           ├── address.txt
│           └── metadata.toml
├── mainnet/
│   ├── config.toml
│   └── identities/
│       └── ...
└── local/
    └── ...
```

## Commands

### `init <network>`

Creates the secrets directory structure for a network with default identities (`admin`, `deployer`, `approver`) and safe defaults.

```bash
./scripts/manage_secrets.sh init testnet
./scripts/manage_secrets.sh init mainnet --force  # overwrite existing
```

### `generate <network> <name>`

Generates a new identity for a network. Supports ed25519 and sr25519 key types via `--key-type`.

```bash
./scripts/manage_secrets.sh generate testnet emergency-approver
./scripts/manage_secrets.sh generate mainnet backup-admin --key-type sr25519
```

### `list <network>`

Lists all configured identities and their addresses for a network.

### `rotate <network> <name>`

Rotates an identity's keys. Archives old keys to `archive/` subdirectory.

### `validate <network>`

Validates the secrets configuration for a network:
- Checks required identities exist
- Validates config.toml is present
- Warns about secret files in tracked directories
- Ensures `.gitignore` is configured

### `export <network>`

Exports environment variables for a network in a safe, non-secret format suitable for `.env` files.

## Configuration

Configuration entries are defined in `config/networks.toml` under the `[secrets.*]` sections:

| Key | Default | Description |
|-----|---------|-------------|
| `rotation_policy_days` | 30 | Days between mandatory key rotations |
| `max_secret_age_days` | 90 | Maximum age before secret is considered stale |
| `vault_backend` | `local` | Backend for secret storage (`local`, `vault`, `aws-sm`) |
| `max_requests_per_window` | 5 | Rate limit for identity operations |
| `window_seconds` | 3600 | Rate limit window duration |
| `cooldown_seconds` | 86400 | Cooldown between successive operations |

## Network-Specific Defaults

| Network | Encryption | Rotation (days) | Backend |
|---------|-----------|-----------------|---------|
| local | disabled | 30 | local |
| testnet | disabled | 30 | local |
| futurenet | disabled | 30 | local |
| mainnet | **enabled** | **14** | vault |

## Safety Features

1. **`.secrets/` is gitignored** — Prevents accidental secret commits
2. **Per-network isolation** — Secrets are directory-isolated per network
3. **Key rotation archival** — Old keys are archived, not deleted
4. **Rate limiting** — Identity operations are rate-limited per configuration
5. **Validation command** — Pre-flight checks before deployment
6. **Safe export** — `export` command outputs env vars without raw secrets

## Integration with Docker

The `docker-compose.yml` can reference secrets via the `.secrets/local/config.toml` path:

```yaml
services:
  contracts-builder:
    environment:
      - STELLAR_NETWORK=testnet
    volumes:
      - ./.secrets/testnet:/secrets:ro
```

## Integration with Deploy Scripts

```bash
# Source exported environment before deploying
eval "$(./scripts/manage_secrets.sh export testnet)"
./scripts/deploy.sh medical_records testnet deployer
```
