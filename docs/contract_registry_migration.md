# Contract Registry Migration Guide

This guide explains how to migrate existing deployments to the new shared `contract_registry` for cross-contract discovery.

## Why migrate

The shared `contract_registry` centralizes contract address discovery, making deployments less brittle and removing the need for direct address wiring between dependent contracts.

## Deployment Steps

1. Deploy the registry contract first:

```bash
./scripts/deploy.sh contract_registry local
```

2. Deploy dependent contracts and save their IDs to `deployments/<network>_<contract>.json`.

3. Register deployed contracts with the registry:

```bash
REGISTRY_ID=$(jq -r '.contract_id' deployments/local_contract_registry.json)

soroban contract invoke --id "$REGISTRY_ID" --network local --source default -- register_contract "medical_records" "$MEDICAL_RECORDS_ID"
soroban contract invoke --id "$REGISTRY_ID" --network local --source default -- register_contract "identity_registry" "$IDENTITY_REGISTRY_ID"
```

4. Point supported contracts at the registry:

For contracts that now support registry discovery, call their `set_contract_registry` entrypoint:

```bash
soroban contract invoke --id "$MEDICAL_RECORDS_ID" --network local --source default -- set_contract_registry "$REGISTRY_ID"
```

5. Verify lookups:

```bash
soroban contract invoke --id "$REGISTRY_ID" --network local --source default -- get_contract "medical_records"
```

## Notes for legacy deployments

- Existing contracts that still store direct dependency addresses continue to operate normally.
- New contract versions should use the registry by default and rely on fallback direct storage only during migration.
- If you deploy a full environment with `deploy_environment.sh`, the script now deploys `contract_registry` first when present.
