# Contract Template Compatibility Guide

## Overview

New contracts must follow the template structure defined in `contracts/contract_template/`. The `scripts/check_template_compat.sh` script validates that contracts conform to this template.

## Required Structure

Every contract must have:

| File | Purpose |
|------|---------|
| `Cargo.toml` | Package definition with `crate-type = ["cdylib"]` and `soroban-sdk` dependency |
| `src/lib.rs` | Main contract with `#![no_std]`, `#![forbid(alloc)]`, `#[contract]`, and `#[contractimpl]` |

## Recommended Structure

These files are recommended (enforced in strict mode):

| File | Purpose |
|------|---------|
| `src/errors.rs` | Error enum with `#[contracterror]` and `#[repr(u32)]` |
| `src/events.rs` | Event emission helper functions |
| `src/types.rs` | Shared type definitions |
| `src/test.rs` | Unit tests |

## Naming Conventions

| Convention | Example |
|------------|---------|
| Contract directory | `snake_case` (e.g., `my_contract`) |
| Cargo package name | `kebab-case` (e.g., `my-contract`) |
| Rust struct name | `PascalCase` (e.g., `MyContract`) |

## Running the Checker

```bash
# Check a single contract
./scripts/check_template_compat.sh my_contract

# Check all workspace contracts
./scripts/check_template_compat.sh --all

# Check all with strict mode (warnings become errors)
./scripts/check_template_compat.sh --all --strict

# Via npm
npm run template:check
npm run template:check:strict
```