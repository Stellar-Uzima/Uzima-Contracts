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

## Upgrade Procedure

1. **Development**: Build the new contract WASM.
2. **Installation**: Deploy (install) the new WASM hash to the network.
3. **Proposal**: Call `propose_upgrade` on the `UpgradeManager`.
4. **Observation**: Wait for the timelock to expire.
5. **Approval**: Obtain necessary validator signatures.
6. **Execution**: Trigger `execute` on the `UpgradeManager`.
7. **Migration (Optional)**: If the data layout changed, the new version's `initialize` or a dedicated `migrate` function should be called.

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
