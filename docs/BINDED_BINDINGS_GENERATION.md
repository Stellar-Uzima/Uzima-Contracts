# Typed Bindings Generation

## Overview

The Uzima contract ecosystem generates type-safe bindings for mobile SDKs and
Python SDK from a central contract interface registry. This ensures consistency
across all SDK implementations.

## Registry Source

Bindings are generated from `schemas/interface-registry/registry.json`, which
is the single source of truth for all contract interfaces.

## Generation

Run the generation script:

```bash
./scripts/generate_bindings.sh              # generate all bindings
./scripts/generate_bindings.sh --python     # Python only
./scripts/generate_bindings.sh --typescript  # TypeScript only
./scripts/generate_bindings.sh --check       # verify bindings are current
```

## Generated Files

| Target | Output Path | Description |
|--------|-------------|-------------|
| Python | `mobile-sdk/python/uzima_sdk/contract_bindings.py` | Python dataclasses and enums |
| TypeScript | `mobile-sdk/core/src/generated/contract-bindings.ts` | TypeScript interfaces and enums |

## Regeneration

After modifying `registry.json` or contract interfaces, always regenerate:

1. Update contract source code
2. Run `node scripts/abi-compat.mjs` to update registry
3. Run `./scripts/generate_bindings.sh` to regenerate bindings
4. Run `./scripts/generate_bindings.sh --check` in CI to verify

## Adding New Contracts

To add bindings for a new contract:

1. Add the contract to `schemas/interface-registry/registry.json`
2. Define interfaces with args/returns
3. Run the generation script
4. The new types will appear in all SDK bindings

## CI Integration

Add to CI pipeline:

```yaml
- name: Verify bindings
  run: ./scripts/generate_bindings.sh --check
```