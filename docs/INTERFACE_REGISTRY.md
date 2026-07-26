# Contract Interface Registry

This document describes the contract interface registry and ABI compatibility testing layer for Uzima contracts.

## Overview

The contract interface registry tracks expected public interfaces for each contract, including function signatures, argument types, and versioning requirements. CI validates that contract changes don't introduce breaking interface changes without an explicit migration path.

## Registry Format

The registry is defined in `schemas/interface-registry/registry.json`:

```json
{
  "version": "1.0.0",
  "contracts": {
    "contract_name": {
      "version": "1.0.0",
      "description": "Contract description",
      "interfaces": {
        "function_name": {
          "description": "What the function does",
          "args": [
            { "name": "arg_name", "type": "Type", "required": true }
          ],
          "returns": "ReturnType",
          "state_mutation": true
        }
      }
    }
  }
}
```

## Validation

The validation script (`scripts/validate-interfaces.mjs`) checks:

1. **Interface existence**: All registered interfaces exist in the contract source
2. **Argument count**: Registered argument counts match the source
3. **Unregistered functions**: Public functions not in the registry (warnings)
4. **Contract existence**: Contract source files exist

### Usage

```bash
# Validate all contracts
node scripts/validate-interfaces.mjs

# Validate a single contract
node scripts/validate-interfaces.mjs medical_records
```

### Output

| File | Description |
|---|---|
| `reports/interface_violations.json` | Structured violation data |
| `reports/interface_report.md` | Human-readable markdown report |

## CI Integration

The CI workflow runs interface validation on every PR:

1. Validates all contracts against the registry
2. Uploads the report as an artifact
3. Comments on the PR with the violation summary
4. Fails if any breaking changes are detected

## Breaking Changes

A breaking change is any modification that would cause existing callers to fail:

- Removing a registered interface
- Changing argument count
- Changing argument types
- Changing return types
- Removing required arguments

### Handling Breaking Changes

When a breaking change is necessary:

1. Document the change in the PR description
2. Update the registry with the new interface
3. Add a migration guide in the PR
4. Consider deprecation period for old interfaces

## Adding a New Contract

1. Add the contract entry to `schemas/interface-registry/registry.json`
2. Define all public interfaces with argument types
3. Run `node scripts/validate-interfaces.mjs` to verify
4. Commit the registry with your contract changes

## Extending the Registry

The registry is designed to be extended:

- Add new contracts as they're developed
- Add detailed type information for stronger validation
- Add version constraints for cross-contract dependencies
- Add event definitions for complete API documentation

## Integration with SDK Generation

The registry can feed into SDK type generation:

- `npm run sdk:generate` reads the registry
- Generates TypeScript/JavaScript types for contract interfaces
- Validates that SDK types match the on-chain interface
