# contract_template

Canonical scaffold for all Uzima Soroban smart contracts.

## Purpose

This template defines the minimum required structure for every new contract:
`#![no_std]`, `#[contract]` + `#[contractimpl]`, an `initialize` function,
`require_auth()` usage, a `#[contracterror]` enum, and event emission.

## Generating a New Contract

```bash
# Simple copy-based scaffold:
./scripts/scaffold-contract.sh <your_contract_name>

# Category-aware scaffold:
./scripts/generate_contract.sh <your_contract_name> <category>

# Verify the scaffold:
./scripts/smoke-test-scaffold.sh <your_contract_name>
```

## Checking Template Compatibility

```bash
# Check all workspace contracts:
./scripts/check_template_compat.sh

# Check a specific contract:
./scripts/check_template_compat.sh <contract_name>

# List every check performed:
./scripts/check_template_compat.sh --list-checks
```

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Main contract: `#[contract]` struct, `#[contractimpl]` block |
| `src/errors.rs` | `#[contracterror]` enum following Uzima conventions |
| `src/events.rs` | Event-emission helpers |
| `src/types.rs` | Shared `#[contracttype]` structs |
| `src/test.rs` | Unit tests using `soroban-sdk/testutils` |
| `Cargo.toml` | Workspace-inherited metadata, `cdylib` crate type |

## Entry Points

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(env, admin)` | None (deployer becomes admin) | One-time init |
| `transfer_admin(env, new_admin)` | Current admin | Transfer admin rights |
| `update_data(env, caller, data)` | Admin | Update stored data |
| `get_admin(env)` | None | Return current admin |
| `get_data(env)` | None | Return stored data |
