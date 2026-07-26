# Deterministic Contract IDs and Network Manifests

This document describes the strategy for deterministic contract ID generation
and the network manifest format used across local, testnet, and mainnet.

## Overview

Soroban contract IDs are derived from a deployer address and a salt value.
By fixing both inputs, the same contract address is always produced for a given
(deployer, contract_name, network) triple — enabling reproducible deployments
and infrastructure-as-code patterns.

## Salt Strategy

The salt for each contract is the SHA-256 of its canonical name:

```
salt = SHA256(contract_name_bytes)
```

For example, `medical_records` → `SHA256(b"medical_records")`.

This guarantees:
- The same contract name always produces the same contract ID on a given network
- Different contract names never collide
- Salts are human-readable and auditable

## Deployment Script

```bash
# Deploy with deterministic ID
./scripts/deploy.sh medical_records testnet --deterministic

# Verify the deployed contract ID matches the manifest
./scripts/verify_deployment.sh medical_records testnet --check-id
```

## Network Manifest Format

The `deployments/network-manifests.json` file tracks contract IDs per network:

```json
{
  "networks": {
    "testnet": {
      "contract_ids": {
        "medical_records": "C...<56-char contract ID>",
        "rbac": "C..."
      }
    }
  }
}
```

After a deterministic deployment, update the manifest:

```bash
./scripts/update_manifest.sh medical_records testnet <contract_id>
git add deployments/network-manifests.json && git commit -m "chore: update testnet contract ID for medical_records"
```

## CI Enforcement

CI validates that the deployed contract IDs match the manifest:

```bash
./scripts/validate_manifest.sh --network testnet
```

Fails if any live contract ID does not match the recorded manifest value.
