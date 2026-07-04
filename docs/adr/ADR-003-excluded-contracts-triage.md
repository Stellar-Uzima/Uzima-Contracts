# ADR-003: Excluded Contracts Triage

**Status:** Accepted
**Date:** 2025-06-29

## Context

The repository has accumulated contracts that are excluded from the active
workspace or not fully integrated into the standard build and test flow. Issue
#828, "Reintegrate 36 Excluded Contracts -- Fix, Test, and Add to Workspace",
identified the need to bring these contracts back under consistent maintenance.

Reintegrating every excluded contract in one unreviewed batch would make review
hard, risk hiding unrelated behavior changes, and make it unclear which
contracts are production-ready.

## Decision

Use an explicit triage process before reintegrating excluded contracts.

Each excluded contract should be classified into one of these buckets:

- `Reintegrate now`: builds, has tests, and fits the active architecture.
- `Fix first`: valuable contract, but blocked by compile errors, missing tests,
  or outdated interfaces.
- `Archive`: obsolete, duplicate, or superseded by another contract.
- `Needs design decision`: requires a separate ADR or issue before code changes.

Only contracts in `Reintegrate now` should be added back to the workspace
directly. Contracts in other buckets need focused follow-up issues or PRs.

## Consequences

The workspace stays reviewable and avoids a high-risk bulk reintegration.

Maintainers get a durable record of why each contract was restored, delayed, or
archived. This reduces repeated debate when future contributors encounter the
same excluded-contract list.

This process adds upfront documentation work, but it makes CI failures and
contract ownership clearer. Follow-up PRs should reference the triage bucket
they are resolving.

## References

- Related issue: #828
- `docs/GOVERNANCE_ARCHITECTURE.md`
- `docs/GOVERNANCE_REFACTORING_GUIDE.md`
