# Deployment Guide

See [docs/DEPLOYMENT_GUIDE.md](../../DEPLOYMENT_GUIDE.md) for the full deployment guide.

## Quick Deploy

```bash
# Local
make deploy-local

# Testnet
./scripts/deploy.sh medical_records testnet

# All contracts to an environment
./scripts/deploy_environment.sh testnet
```

## With Rollback Support

```bash
./scripts/deploy_with_rollback.sh medical_records testnet
```

## Deployment Artifacts

Deployment metadata is stored in `deployments/`:

```json
{
  "contract_name": "medical_records",
  "contract_id": "C...",
  "network": "testnet",
  "deployed_at": "2026-04-28T00:00:00Z"
}
```
