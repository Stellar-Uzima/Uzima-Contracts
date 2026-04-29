# Environment Setup

## Network Configuration

The project supports three Stellar networks:

| Network | RPC URL | Use Case |
|---------|---------|---------|
| Local | `http://localhost:8000/soroban/rpc` | Development |
| Testnet | `https://soroban-testnet.stellar.org:443` | Testing |
| Futurenet | `https://rpc-futurenet.stellar.org:443` | Staging |

### Configure Networks

```bash
# Local
soroban config network add local \
  --rpc-url http://localhost:8000/soroban/rpc \
  --network-passphrase "Standalone Network ; February 2017"

# Testnet
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

### Generate Identity

```bash
soroban config identity generate default
```

## Environment Variables

Create a `.env` file (not committed to git):

```bash
SOROBAN_NETWORK=testnet
SOROBAN_ACCOUNT=default
CONTRACT_ID_MEDICAL_RECORDS=C...
CONTRACT_ID_HEALTHCARE_PAYMENT=C...
```

## SDK Version Pinning

The workspace uses exact version pinning to ensure reproducible builds:

```toml
# Cargo.toml
[workspace.dependencies]
soroban-sdk = { version = "=21.7.7" }
```

The `=` prefix pins to exactly `21.7.7`, preventing accidental upgrades.
