# ABI Drift Report Template

**Generated:** {{DATE}}
**Baseline:** {{BASELINE_REF}}
**Repository:** Stellar-Uzima/Uzima-Contracts

## Summary

| Metric | Count |
|--------|-------|
| Contracts in registry | {{CONTRACT_COUNT}} |
| Breaking changes | {{BREAKING_COUNT}} |
| Non-breaking changes | {{NON_BREAKING_COUNT}} |

## Breaking Changes

{{BREAKING_CHANGES_LIST}}

## Non-Breaking Changes

{{NON_BREAKING_CHANGES_LIST}}

## Downstream Impact

### Mobile SDK (React Native)
- Regenerate bindings: \./scripts/generate_bindings.sh --typescript\
- Update SDK version if breaking

### Python SDK
- Regenerate bindings: \./scripts/generate_bindings.sh --python\
- Update SDK version if breaking

### Other Consumers
- Check [CONTRACT_COMPATIBILITY.md](../docs/CONTRACT_COMPATIBILITY.md)
- Review [CHANGE_IMPACT_MATRIX.md](../docs/CHANGE_IMPACT_MATRIX.md)

## Remediation

If breaking changes are intentional:
1. Add \[approve-abi-change]\ to the PR body
2. Update downstream SDK versions
3. Notify consumers via CHANGELOG