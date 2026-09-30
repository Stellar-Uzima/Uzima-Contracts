<!--
Required header fields (checked by scripts/check_adr_required.py):
  - the `# ADR-{NUMBER}: {TITLE}` heading must match the file name's number
  - `**Status:**` must start with Proposed, Accepted, Deprecated or Superseded
  - `**Date:**` must be an ISO date (YYYY-MM-DD)
Fill in Change Classification too; it is what the "ADR required for breaking
changes" rule keys off. Delete this comment block before merging.
-->
# ADR-{NUMBER}: {TITLE}

**Status:** Proposed | Accepted | Deprecated | Superseded by [ADR-{XXX}]
**Date:** {YYYY-MM-DD}
**Deciders:** {LIST OF DECISION MAKERS}

## Context

What is the issue that we're seeing that is motivating this decision or change?

## Decision

What is the change that we're proposing and/or doing?

## Change Classification

Fill this in for every contract change; it decides whether the ADR is required
by `scripts/check_adr_required.py` or merely recommended.

- **Breaking ABI change:** yes / no — if yes, list the exported functions removed or re-signed.
- **Storage-layout change:** yes / no — if yes, list the `DataKey` variants or `#[contracttype]` fields removed or renamed.
- **Migration required:** yes / no — link the migration script or steps if yes.
- **Affected contracts:** the contracts and callers whose behavior changes.

## Alternatives Considered

What other options were evaluated?

| Option | Pros | Cons |
|--------|------|------|
| {Alternative A} | ... | ... |
| {Alternative B} | ... | ... |

## Rationale

Why this decision was made. Reference:
- Existing patterns in the codebase
- Soroban SDK constraints (WASM size, resource limits)
- Healthcare compliance requirements (HIPAA, GDPR)
- Security implications

## Consequences

What becomes easier or more difficult to do because of this change?

### Positive
- ...

### Negative
- ...

### Risks
- ...

## Follow-Up Actions

- [ ] {Action item 1}
- [ ] {Action item 2}

## References

- [Related ADR](link)
- [GitHub Issue](link)
- [Soroban Documentation](https://soroban.stellar.org/docs)
