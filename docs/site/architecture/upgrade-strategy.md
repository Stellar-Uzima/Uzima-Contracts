# Upgrade Strategy

See [docs/CONTRACT_UPGRADE_SAFETY.md](../../CONTRACT_UPGRADE_SAFETY.md) and [docs/VERSIONING_STRATEGY.md](../../VERSIONING_STRATEGY.md).

## Upgrade Mechanism

Contracts use the `upgradeability` contract and `upgrade_manager` for safe upgrades.

## Deprecation

Legacy entrypoints are tracked via the `deprecation_framework` contract, which emits warning events and provides migration guides.

## Version Pinning

The workspace pins exact SDK versions (`=21.7.7`) to ensure reproducible builds across all environments.
