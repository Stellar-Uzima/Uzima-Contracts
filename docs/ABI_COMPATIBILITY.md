# ABI Compatibility Guide

## Overview

The Uzima contract ecosystem maintains ABI compatibility across contract
versions to ensure downstream consumers (mobile SDKs, Python SDK, indexers)
can safely interact with upgraded contracts.

## ABI Drift Detection

### What is ABI Drift?

ABI drift occurs when a contract's public interface changes between versions.
This can break downstream consumers that depend on specific function signatures,
field names, or types.

### Detection Workflow

1. **Automated Checks**: CI runs \scripts/abi_compat_check.sh --ci\ on every PR
2. **Report Generation**: Human-readable reports generated in \eports/\
3. **Review Process**: Breaking changes require explicit approval

### Types of Changes

#### Breaking Changes (Require Approval)
- Contract interface removal
- Required field addition
- Type change of existing field
- Enum variant removal
- Function signature change

#### Non-Breaking Changes (Auto-Approved)
- New optional field
- New enum variant
- New interface addition
- Contract version bump

## Scripts

### abi_compat_check.sh

Main entry point for ABI compatibility checking:

\\\ash
./scripts/abi_compat_check.sh                    # full check
./scripts/abi_compat_check.sh --baseline v1.0.0  # compare against tag
./scripts/abi_compat_check.sh --report           # report only
./scripts/abi_compat_check.sh --ci               # CI mode
\\\

### abi-compat.mjs

Low-level ABI comparison tool:

\\\ash
node scripts/abi-compat.mjs                   # generate snapshots
node scripts/abi-compat.mjs --check           # check compatibility
node scripts/abi_compat.mjs --check --report reports/compat.txt
\\\

## Integration

### CI/CD Pipeline

\\\yaml
- name: ABI Compatibility Check
  run: ./scripts/abi_compat_check.sh --ci
\\\

### Pre-commit Hook

\\\ash
./scripts/abi_compat_check.sh --report
\\\

## Downstream Consumer Guide

### Mobile SDK (React Native)

1. After contract upgrade, regenerate bindings:
   \\\ash
   ./scripts/generate_bindings.sh --typescript
   \\\
2. Update SDK version in package.json
3. Run tests to verify compatibility

### Python SDK

1. After contract upgrade, regenerate bindings:
   \\\ash
   ./scripts/generate_bindings.sh --python
   \\\
2. Update version in setup.py/pyproject.toml
3. Run tests to verify compatibility

### Custom Integrations

1. Review the ABI drift report in \eports/\
2. Check [CHANGE_IMPACT_MATRIX.md](CHANGE_IMPACT_MATRIX.md)
3. Update your code to handle any breaking changes
4. Test against the new contract version

## Approval Process

To approve breaking ABI changes:

1. Review the generated report
2. Confirm changes are intentional and necessary
3. Add \[approve-abi-change]\ to the PR body
4. Ensure downstream SDKs are updated

## Resources

- [CONTRACT_COMPATIBILITY.md](CONTRACT_COMPATIBILITY.md)
- [CHANGE_IMPACT_MATRIX.md](CHANGE_IMPACT_MATRIX.md)
- [VERSIONING_STRATEGY.md](VERSIONING_STRATEGY.md)
- [API_REFERENCE.md](API_REFERENCE.md)