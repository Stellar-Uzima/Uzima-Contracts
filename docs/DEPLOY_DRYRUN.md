# Deployment Dry-Run & Plan Generation

Simulates contract deployment and generates JSON deployment plans for release candidates without actually deploying.

## Overview

The `deploy_dryrun.sh` script analyzes contracts, checks build status, validates dependencies, and produces a structured JSON deployment plan. This allows teams to review and approve deployment plans before execution.

## Quick Start

```bash
# Single contract dry-run
./scripts/deploy_dryrun.sh medical_records testnet

# All contracts dry-run
./scripts/deploy_dryrun.sh --all testnet

# Save plan to file
./scripts/deploy_dryrun.sh governor mainnet --output plan.json

# All contracts with output file
./scripts/deploy_dryrun.sh --all mainnet --output full-plan.json
```

## Output Format

The script outputs a JSON plan conforming to `deployments/plan-schema.json`:

### Single Contract Plan

```json
{
  "plan_version": "1.0.0",
  "generated_at": "2026-07-24T00:00:00Z",
  "dry_run": true,
  "network": {
    "name": "testnet",
    "passphrase": "Test SDF Network ; September 2015",
    "rpc_url": "https://soroban-testnet.stellar.org"
  },
  "contract": {
    "name": "medical_records",
    "wasm_path": "target/wasm32-unknown-unknown/release/medical_records.wasm",
    "wasm_size_bytes": 245760,
    "dependencies": ["common_error", "rbac"]
  },
  "build": { "status": "skipped", "check": "passed" },
  "deployment": { "estimated_gas": 50000000, "source_account": "<identity>", "init_required": true },
  "safety": { "status": "passed", "warnings": [] },
  "validation": { "contract_exists": true, "wasm_available": true, "dependencies_met": true },
  "estimated_cost_xlm": 0.5
}
```

### Multi-Contract Plan (--all)

```json
{
  "plan_version": "1.0.0",
  "dry_run": true,
  "network": { "name": "mainnet", "..." : "..." },
  "total_contracts": 11,
  "deployment_order": [
    { "order": 1, "name": "common_error", "wasm_available": true, "estimated_gas": 20000000 },
    { "order": 2, "name": "common_auth", "wasm_available": true, "estimated_gas": 20000000 },
    "..."
  ],
  "summary": {
    "total_estimated_gas": 450000000,
    "estimated_total_cost_xlm": 4.5,
    "builds_needed": 0,
    "wasm_available": 11,
    "wasm_missing": 0,
    "safety_warnings": ["mainnet_deployment_requires_manual_approval"]
  }
}
```

## Safety Checks

| Network | Safety Status | Description |
|---------|--------------|-------------|
| local | `passed` | No restrictions |
| testnet | `passed` | Test network, safe to deploy |
| futurenet | `passed` | Test network, safe to deploy |
| mainnet | `requires_approval` | Production deployment requires manual approval |

## Plan Schema

The plan format is defined in `deployments/plan-schema.json`. Key fields:

| Field | Type | Description |
|-------|------|-------------|
| `plan_version` | string | Plan format version (`1.0.0`) |
| `dry_run` | boolean | Always `true` for dry-run plans |
| `build.status` | enum | `skipped`, `required`, `completed` |
| `safety.status` | enum | `passed`, `requires_approval`, `blocked` |
| `estimated_cost_xlm` | number | Estimated cost in XLM |

## Integration with CI/CD

```yaml
# .github/workflows/deploy-preview.yml
- name: Generate deployment plan
  run: |
    ./scripts/deploy_dryrun.sh --all testnet --output plan.json
    cat plan.json

- name: Review plan
  run: |
    python3 -c "
    import json
    with open('plan.json') as f:
        plan = json.load(f)
    print(f'Contracts: {plan[\"total_contracts\"]}')
    print(f'Est. cost: {plan[\"summary\"][\"estimated_total_cost_xlm\"]} XLM')
    assert plan['dry_run'] == True
    "
```

## Usage with Release Process

```bash
# 1. Generate plan for release candidate
./scripts/deploy_dryrun.sh --all testnet --output rc-plan.json

# 2. Review and approve the plan
cat rc-plan.json | python3 -m json.tool

# 3. After approval, execute deployment
./scripts/deploy.sh medical_records testnet deployer
```
