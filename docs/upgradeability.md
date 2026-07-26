# Uzima Smart Contract Upgradeability System

## Overview
The Uzima-Contracts repository implements a sophisticated upgradeability system designed for long-term maintainability, security, and zero-downtime updates. The system leverages Soroban's native `update_current_contract_wasm` capability, wrapped in a governance-controlled framework.

## Architecture

### 1. Transparent Upgrade Pattern
Contracts use the native Soroban upgrade mechanism. This preserves the contract address and all persistent storage, eliminating the need for complex proxy delegation (like EVM's DELEGATECALL) and the associated gas overhead.

### 2. UpgradeManager (Governance Layer)
The `UpgradeManager` contract acts as the central authority for all upgrades. It implements:
- **Proposal System**: Admins propose a new WASM hash.
- **Timelock**: A mandatory delay (default 24h) before execution to allow users to audit the new code.
- **Multi-Sig Validation**: Requires multiple validator signatures for high-stakes upgrades.
- **Auditable History**: Tracks every upgrade, version number, and description.

### 3. Immutable Storage Patterns
To prevent data corruption during upgrades, the system enforces:
- **Key Separation**: Critical configuration is stored in `Instance` storage, while user data is in `Persistent` storage.
- **Version Tracking**: Every contract stores its current version.
- **Cross-Version Compatibility**: New versions must increment the version number and handle migrations if necessary.
- **Deprecation Tracking**: Deprecated entrypoints can be registered during upgrades and monitored via emitted warning events.

## Upgrade Procedure

1. **Development**: Build the new contract WASM.
2. **Installation**: Deploy (install) the new WASM hash to the network.
3. **Plan**: Record the intended migration in `deployments/<network>/<contract>/plan.json`.
4. **Dry Run**: Execute `./scripts/migrate_contract.sh <contract> --network <network> --dry-run` locally and review the projected storage diff, gas, and emitted events before any live transaction.
5. **Proposal**: Call `propose_upgrade` on the `UpgradeManager`.
6. **Observation**: Wait for the timelock to expire.
7. **Approval**: Obtain necessary validator signatures.
8. **Execution**: Trigger `execute` on the `UpgradeManager`, or run `./scripts/migrate_contract.sh <contract> --network <network> --i-understand-this-is-live` for direct admin-managed upgrades after the dry-run has been approved internally.
9. **Migration (Optional)**: If the data layout changed, the new version's `initialize` or a dedicated `migrate` function should be called.
10. **Deprecation Registration (Optional)**: If old entrypoints remain temporarily supported, register them with the shared upgradeability deprecation registry.

## Planned Migration Dry Runs

Operators should keep each planned migration beside other deployment artifacts:

```text
deployments/<network>/<contract>/plan.json
```

The migration plan is the source of truth for both dry-run review and live execution. At minimum it must include:

- `new_wasm_hash`
- `version_bump`
- `expected_storage_migration_steps`
- `expected_gas`
- `expected_event_emissions`

The checked-in template at `deployments/testnet/medical_records/plan.json` shows the full shape used by `scripts/migrate_contract.sh`, including optional operator metadata such as `contract_id`, `admin_identity`, `upgrade_entrypoint`, and `caller_arg_name`.

### Dry-Run Workflow

```bash
./scripts/migrate_contract.sh medical_records --network testnet --dry-run
```

Dry-run mode performs no network submission. Instead it reads the plan file and prints:

- the projected storage state diff derived from `expected_storage_migration_steps`
- the projected gas budget from `expected_gas`
- the projected event emissions from `expected_event_emissions`

### Live Execution Guardrails

Live mode is intentionally harder to trigger:

```bash
./scripts/migrate_contract.sh medical_records \
  --network testnet \
  --identity testnet-admin \
  --i-understand-this-is-live
```

The script refuses to submit unless:

- `--i-understand-this-is-live` is present
- the operator types `LIVE` at the interactive prompt
- the plan includes the contract ID and upgrade metadata needed for the `upgrade` invocation

## Deprecation Warnings
The shared `upgradeability` crate now supports:

- tracking deprecated functions in contract storage
- registering deprecations as part of an upgrade
- emitting `Deprecated` events when legacy entrypoints are used

For the implementation pattern and migration expectations, see [docs/deprecation_migration.md](./deprecation_migration.md).

## Security Features
- **Rollback**: In case of a critical bug, the `UpgradeManager` can facilitate a rollback to the previous known-good WASM hash.
- **Freezing**: The admin can permanently freeze a contract, disabling future upgrades (immutability lock).
- **Required Auth**: All upgrade functions require strict cryptographic authentication.

## Testing
Comprehensive integration tests are located in `tests/upgrade_integration_test.rs`, covering:
- Successful upgrades.
- Failed upgrades (invalid version, not enough approvals).
- Timelock enforcement.
- Data persistence across versions.
